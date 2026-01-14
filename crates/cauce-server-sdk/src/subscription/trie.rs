//! Topic trie for efficient wildcard pattern matching.
//!
//! The trie supports Cauce Protocol wildcard patterns:
//! - `*` matches exactly one segment
//! - `**` matches zero or more segments (must be last segment)

use std::collections::HashMap;

/// A trie node for topic pattern matching.
#[derive(Debug, Default)]
struct TrieNode {
    /// Children nodes keyed by segment
    children: HashMap<String, TrieNode>,
    /// Subscription IDs that have patterns ending at this node
    subscriptions: Vec<String>,
    /// Whether this node has a `*` wildcard child
    single_wildcard: Option<Box<TrieNode>>,
    /// Subscription IDs that have `**` at this node (match rest of path)
    multi_wildcard_subscriptions: Vec<String>,
}

/// A trie for efficient topic pattern matching.
///
/// Supports the Cauce Protocol wildcard patterns:
/// - `*` matches exactly one segment
/// - `**` matches zero or more segments
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::subscription::TopicTrie;
///
/// let mut trie = TopicTrie::new();
/// trie.insert("signal.email.*", "sub_1");
/// trie.insert("signal.**", "sub_2");
///
/// // Get matching subscriptions for a topic
/// let matches = trie.get_matches("signal.email.received");
/// assert!(matches.contains(&"sub_1".to_string()));
/// assert!(matches.contains(&"sub_2".to_string()));
/// ```
#[derive(Debug, Default)]
pub struct TopicTrie {
    root: TrieNode,
}

impl TopicTrie {
    /// Creates a new empty topic trie.
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    /// Inserts a topic pattern with a subscription ID.
    ///
    /// The pattern can contain:
    /// - Literal segments: `email`, `signal`
    /// - Single wildcard: `*` matches one segment
    /// - Multi wildcard: `**` matches zero or more segments (must be last)
    pub fn insert(&mut self, pattern: &str, subscription_id: &str) {
        let segments: Vec<&str> = pattern.split('.').collect();
        Self::insert_into_node(&mut self.root, &segments, subscription_id);
    }

    fn insert_into_node(node: &mut TrieNode, segments: &[&str], subscription_id: &str) {
        if segments.is_empty() {
            node.subscriptions.push(subscription_id.to_string());
            return;
        }

        let segment = segments[0];
        let rest = &segments[1..];

        if segment == "**" {
            // Multi-wildcard must be last - store here
            node.multi_wildcard_subscriptions
                .push(subscription_id.to_string());
        } else if segment == "*" {
            // Single wildcard
            let wildcard_node = node.single_wildcard.get_or_insert_with(Box::default);
            Self::insert_into_node(wildcard_node.as_mut(), rest, subscription_id);
        } else {
            // Literal segment
            let child = node.children.entry(segment.to_string()).or_default();
            Self::insert_into_node(child, rest, subscription_id);
        }
    }

    /// Removes a subscription ID from a pattern.
    pub fn remove(&mut self, pattern: &str, subscription_id: &str) {
        let segments: Vec<&str> = pattern.split('.').collect();
        Self::remove_from_node(&mut self.root, &segments, subscription_id);
    }

    fn remove_from_node(node: &mut TrieNode, segments: &[&str], subscription_id: &str) {
        if segments.is_empty() {
            node.subscriptions.retain(|s| s != subscription_id);
            return;
        }

        let segment = segments[0];
        let rest = &segments[1..];

        if segment == "**" {
            node.multi_wildcard_subscriptions
                .retain(|s| s != subscription_id);
        } else if segment == "*" {
            if let Some(ref mut wildcard_node) = node.single_wildcard {
                Self::remove_from_node(wildcard_node.as_mut(), rest, subscription_id);
            }
        } else if let Some(child) = node.children.get_mut(segment) {
            Self::remove_from_node(child, rest, subscription_id);
        }
    }

    /// Gets all subscription IDs that match a concrete topic.
    ///
    /// The topic should not contain wildcards - it's the actual
    /// topic being published to.
    pub fn get_matches(&self, topic: &str) -> Vec<String> {
        let segments: Vec<&str> = topic.split('.').collect();
        let mut matches = Vec::new();
        Self::collect_matches(&self.root, &segments, &mut matches);
        matches
    }

    fn collect_matches(node: &TrieNode, segments: &[&str], matches: &mut Vec<String>) {
        // Multi-wildcard matches anything from this point
        matches.extend(node.multi_wildcard_subscriptions.iter().cloned());

        if segments.is_empty() {
            // End of topic - collect exact matches
            matches.extend(node.subscriptions.iter().cloned());
            return;
        }

        let segment = segments[0];
        let rest = &segments[1..];

        // Check single wildcard (matches this segment)
        if let Some(ref wildcard_node) = node.single_wildcard {
            Self::collect_matches(wildcard_node.as_ref(), rest, matches);
        }

        // Check literal match
        if let Some(child) = node.children.get(segment) {
            Self::collect_matches(child, rest, matches);
        }
    }

    /// Checks if a pattern matches a concrete topic.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use cauce_server_sdk::subscription::TopicTrie;
    ///
    /// assert!(TopicTrie::pattern_matches("signal.*", "signal.email"));
    /// assert!(TopicTrie::pattern_matches("signal.**", "signal.email.received"));
    /// assert!(!TopicTrie::pattern_matches("signal.email", "signal.slack"));
    /// ```
    pub fn pattern_matches(pattern: &str, topic: &str) -> bool {
        let pattern_segments: Vec<&str> = pattern.split('.').collect();
        let topic_segments: Vec<&str> = topic.split('.').collect();
        Self::segments_match(&pattern_segments, &topic_segments)
    }

    fn segments_match(pattern: &[&str], topic: &[&str]) -> bool {
        if pattern.is_empty() {
            return topic.is_empty();
        }

        let p = pattern[0];

        if p == "**" {
            // ** matches zero or more segments, must be last
            return true;
        }

        if topic.is_empty() {
            return false;
        }

        let t = topic[0];

        if p == "*" || p == t {
            Self::segments_match(&pattern[1..], &topic[1..])
        } else {
            false
        }
    }

    /// Validates a topic pattern.
    ///
    /// Returns an error message if the pattern is invalid.
    pub fn validate_pattern(pattern: &str) -> Result<(), String> {
        if pattern.is_empty() {
            return Err("pattern cannot be empty".to_string());
        }

        let segments: Vec<&str> = pattern.split('.').collect();

        for (i, segment) in segments.iter().enumerate() {
            if segment.is_empty() {
                return Err("pattern cannot have empty segments".to_string());
            }

            if *segment == "**" && i != segments.len() - 1 {
                return Err("** wildcard must be the last segment".to_string());
            }

            // Check for invalid characters (basic validation)
            if segment.contains(|c: char| !c.is_alphanumeric() && c != '_' && c != '-' && c != '*')
            {
                return Err(format!("invalid character in segment: {}", segment));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_exact_match() {
        let mut trie = TopicTrie::new();
        trie.insert("signal.email.received", "sub_1");

        let matches = trie.get_matches("signal.email.received");
        assert_eq!(matches, vec!["sub_1"]);

        let no_matches = trie.get_matches("signal.email.sent");
        assert!(no_matches.is_empty());
    }

    #[test]
    fn test_trie_single_wildcard() {
        let mut trie = TopicTrie::new();
        trie.insert("signal.*.received", "sub_1");

        let matches = trie.get_matches("signal.email.received");
        assert_eq!(matches, vec!["sub_1"]);

        let matches = trie.get_matches("signal.slack.received");
        assert_eq!(matches, vec!["sub_1"]);

        let no_matches = trie.get_matches("signal.email.sent");
        assert!(no_matches.is_empty());
    }

    #[test]
    fn test_trie_multi_wildcard() {
        let mut trie = TopicTrie::new();
        trie.insert("signal.**", "sub_1");

        let matches = trie.get_matches("signal.email");
        assert_eq!(matches, vec!["sub_1"]);

        let matches = trie.get_matches("signal.email.received");
        assert_eq!(matches, vec!["sub_1"]);

        let matches = trie.get_matches("signal.email.thread.reply");
        assert_eq!(matches, vec!["sub_1"]);

        // Multi-wildcard matches zero segments too
        let matches = trie.get_matches("signal");
        assert_eq!(matches, vec!["sub_1"]);
    }

    #[test]
    fn test_trie_multiple_subscriptions() {
        let mut trie = TopicTrie::new();
        trie.insert("signal.email.*", "sub_1");
        trie.insert("signal.**", "sub_2");
        trie.insert("signal.email.received", "sub_3");

        let matches = trie.get_matches("signal.email.received");
        assert!(matches.contains(&"sub_1".to_string()));
        assert!(matches.contains(&"sub_2".to_string()));
        assert!(matches.contains(&"sub_3".to_string()));
    }

    #[test]
    fn test_trie_remove() {
        let mut trie = TopicTrie::new();
        trie.insert("signal.email.*", "sub_1");
        trie.insert("signal.email.*", "sub_2");

        trie.remove("signal.email.*", "sub_1");

        let matches = trie.get_matches("signal.email.received");
        assert!(!matches.contains(&"sub_1".to_string()));
        assert!(matches.contains(&"sub_2".to_string()));
    }

    #[test]
    fn test_pattern_matches() {
        assert!(TopicTrie::pattern_matches("signal.email", "signal.email"));
        assert!(!TopicTrie::pattern_matches("signal.email", "signal.slack"));

        assert!(TopicTrie::pattern_matches("signal.*", "signal.email"));
        assert!(TopicTrie::pattern_matches("signal.*", "signal.slack"));
        assert!(!TopicTrie::pattern_matches("signal.*", "signal.email.thread"));

        assert!(TopicTrie::pattern_matches("signal.**", "signal"));
        assert!(TopicTrie::pattern_matches("signal.**", "signal.email"));
        assert!(TopicTrie::pattern_matches("signal.**", "signal.email.thread"));
        assert!(!TopicTrie::pattern_matches("signal.**", "other.topic"));

        assert!(TopicTrie::pattern_matches("*.email.*", "signal.email.received"));
        assert!(!TopicTrie::pattern_matches("*.email.*", "signal.slack.received"));
    }

    #[test]
    fn test_validate_pattern() {
        assert!(TopicTrie::validate_pattern("signal.email").is_ok());
        assert!(TopicTrie::validate_pattern("signal.*").is_ok());
        assert!(TopicTrie::validate_pattern("signal.**").is_ok());
        assert!(TopicTrie::validate_pattern("signal.*.received").is_ok());

        assert!(TopicTrie::validate_pattern("").is_err());
        assert!(TopicTrie::validate_pattern("signal..email").is_err());
        assert!(TopicTrie::validate_pattern("signal.**.email").is_err()); // ** must be last
    }
}
