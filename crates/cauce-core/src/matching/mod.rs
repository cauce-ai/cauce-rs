//! Topic pattern matching for the Cauce Protocol.
//!
//! This module provides utilities for matching topics against subscription patterns:
//!
//! - [`TopicMatcher`] - Static methods for topic pattern matching
//! - [`topic_matches`] - Convenience function for single pattern matching
//!
//! ## Wildcard Support
//!
//! - `*` - Matches exactly one segment
//! - `**` - Matches one or more segments
//!
//! ## Examples
//!
//! ```
//! use cauce_core::matching::{TopicMatcher, topic_matches};
//!
//! // Single segment wildcard
//! assert!(topic_matches("signal.email", "signal.*"));
//! assert!(!topic_matches("signal.email.received", "signal.*"));
//!
//! // Multi-segment wildcard
//! assert!(topic_matches("signal.email.received", "signal.**"));
//! assert!(topic_matches("signal.email.inbox.unread", "signal.**"));
//! ```

/// A utility struct for topic pattern matching.
///
/// `TopicMatcher` provides static methods for matching topics against patterns
/// with wildcard support.
pub struct TopicMatcher;

impl TopicMatcher {
    /// Checks if a topic matches a pattern.
    ///
    /// # Wildcards
    ///
    /// - `*` matches exactly one segment
    /// - `**` matches one or more segments
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to check
    /// * `pattern` - The pattern to match against
    ///
    /// # Returns
    ///
    /// `true` if the topic matches the pattern, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::matching::TopicMatcher;
    ///
    /// assert!(TopicMatcher::matches("signal.email", "signal.*"));
    /// assert!(!TopicMatcher::matches("signal.email.sent", "signal.*"));
    /// assert!(TopicMatcher::matches("signal.email.sent", "signal.**"));
    /// ```
    pub fn matches(topic: &str, pattern: &str) -> bool {
        let topic_segments: Vec<&str> = topic.split('.').collect();
        let pattern_segments: Vec<&str> = pattern.split('.').collect();
        matches_segments(&topic_segments, &pattern_segments)
    }

    /// Checks if a topic matches any of the given patterns.
    ///
    /// # Arguments
    ///
    /// * `topic` - The topic to check
    /// * `patterns` - A slice of patterns to match against
    ///
    /// # Returns
    ///
    /// `true` if the topic matches at least one pattern, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::matching::TopicMatcher;
    ///
    /// let patterns = &["signal.email.*", "signal.slack.**"];
    /// assert!(TopicMatcher::matches_any("signal.email.sent", patterns));
    /// assert!(TopicMatcher::matches_any("signal.slack.dm.new", patterns));
    /// assert!(!TopicMatcher::matches_any("action.send", patterns));
    /// ```
    pub fn matches_any(topic: &str, patterns: &[&str]) -> bool {
        patterns.iter().any(|pattern| Self::matches(topic, pattern))
    }
}

/// Convenience function for topic pattern matching.
///
/// This is equivalent to [`TopicMatcher::matches`].
///
/// # Example
///
/// ```
/// use cauce_core::matching::topic_matches;
///
/// assert!(topic_matches("signal.email.received", "signal.**"));
/// ```
pub fn topic_matches(topic: &str, pattern: &str) -> bool {
    TopicMatcher::matches(topic, pattern)
}

/// Recursively matches topic segments against pattern segments.
///
/// Uses a context to track whether `**` has consumed at least one segment.
fn matches_segments(topic: &[&str], pattern: &[&str]) -> bool {
    matches_segments_inner(topic, pattern, false)
}

/// Inner recursive function with tracking for `**` consumption.
///
/// `star_star_consumed` tracks whether the current `**` has consumed at least one segment.
fn matches_segments_inner(topic: &[&str], pattern: &[&str], star_star_consumed: bool) -> bool {
    match (topic.first(), pattern.first()) {
        // Both empty - match
        (None, None) => true,

        // Topic has more segments, pattern is empty - no match
        (Some(_), None) => false,

        // Topic is empty, pattern has `**` left
        (None, Some(&"**")) => {
            // ** matches 1+ segments. If we've consumed at least one, we can move past **.
            if star_star_consumed {
                matches_segments_inner(&[], &pattern[1..], false)
            } else {
                // ** hasn't consumed anything yet, can't match empty topic
                false
            }
        }

        // Topic is empty, pattern has non-** segment - no match
        (None, Some(_)) => false,

        // Pattern has `**` - try matching 1 or more segments
        (Some(seg), Some(&"**")) => {
            // Skip empty segments (from splitting empty string)
            if seg.is_empty() {
                return matches_segments_inner(&topic[1..], pattern, star_star_consumed);
            }

            if pattern.len() == 1 {
                // ** is the last pattern segment, matches all remaining topic segments
                // As long as there's at least one non-empty segment
                true
            } else {
                // Option 1: ** consumes this segment (mark as consumed)
                // Option 2: ** has consumed enough, try matching rest of pattern
                matches_segments_inner(&topic[1..], pattern, true)
                    || (star_star_consumed && matches_segments_inner(topic, &pattern[1..], false))
            }
        }

        // Pattern has `*` - matches exactly one segment
        (Some(seg), Some(&"*")) => {
            // Skip empty segments
            if seg.is_empty() {
                return matches_segments_inner(&topic[1..], pattern, false);
            }
            matches_segments_inner(&topic[1..], &pattern[1..], false)
        }

        // Exact match required
        (Some(t), Some(p)) if *t == *p => matches_segments_inner(&topic[1..], &pattern[1..], false),

        // Skip empty topic segments
        (Some(&""), Some(_)) => matches_segments_inner(&topic[1..], pattern, false),

        // No match
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Exact matching tests =====

    #[test]
    fn test_exact_match() {
        assert!(topic_matches("signal.email", "signal.email"));
    }

    #[test]
    fn test_exact_match_single_segment() {
        assert!(topic_matches("signal", "signal"));
    }

    #[test]
    fn test_exact_match_many_segments() {
        assert!(topic_matches(
            "signal.email.inbox.unread",
            "signal.email.inbox.unread"
        ));
    }

    #[test]
    fn test_exact_no_match_different() {
        assert!(!topic_matches("signal.email", "signal.slack"));
    }

    #[test]
    fn test_exact_no_match_length() {
        assert!(!topic_matches("signal.email", "signal.email.received"));
        assert!(!topic_matches("signal.email.received", "signal.email"));
    }

    // ===== Single-segment wildcard (*) tests =====

    #[test]
    fn test_single_wildcard_matches_one() {
        assert!(topic_matches("signal.email", "signal.*"));
    }

    #[test]
    fn test_single_wildcard_no_match_two() {
        assert!(!topic_matches("signal.email.received", "signal.*"));
    }

    #[test]
    fn test_single_wildcard_middle() {
        assert!(topic_matches("signal.email.received", "signal.*.received"));
    }

    #[test]
    fn test_single_wildcard_start() {
        assert!(topic_matches("email.received", "*.received"));
    }

    #[test]
    fn test_single_wildcard_no_match_empty() {
        // Pattern "signal.*" requires exactly 2 segments
        assert!(!topic_matches("signal", "signal.*"));
    }

    #[test]
    fn test_multiple_single_wildcards() {
        assert!(topic_matches("signal.email.received", "*.*.*"));
        assert!(!topic_matches("signal.email", "*.*.*"));
    }

    // ===== Multi-segment wildcard (**) tests =====

    #[test]
    fn test_multi_wildcard_matches_one() {
        assert!(topic_matches("signal.email", "signal.**"));
    }

    #[test]
    fn test_multi_wildcard_matches_many() {
        assert!(topic_matches("signal.email.received", "signal.**"));
        assert!(topic_matches("signal.email.inbox.unread", "signal.**"));
    }

    #[test]
    fn test_multi_wildcard_end_pattern() {
        assert!(topic_matches("signal.email.received", "**.received"));
    }

    #[test]
    fn test_multi_wildcard_middle() {
        assert!(topic_matches(
            "signal.email.inbox.received",
            "signal.**.received"
        ));
    }

    #[test]
    fn test_multi_wildcard_requires_at_least_one() {
        // ** matches 1+ segments, not 0
        // "signal.**" should NOT match "signal" alone
        // But per the spec, ** matches "one or more segments"
        // Let's verify: "signal.**" pattern on "signal" topic
        assert!(!topic_matches("signal", "signal.**"));
    }

    #[test]
    fn test_multi_wildcard_alone() {
        // Just "**" should match any topic with at least one segment
        assert!(topic_matches("signal", "**"));
        assert!(topic_matches("signal.email", "**"));
        assert!(topic_matches("signal.email.received", "**"));
    }

    // ===== TopicMatcher::matches_any tests =====

    #[test]
    fn test_matches_any_first_pattern() {
        let patterns = &["signal.*", "action.*"];
        assert!(topic_matches("signal.email", "signal.*"));
        assert!(TopicMatcher::matches_any("signal.email", patterns));
    }

    #[test]
    fn test_matches_any_second_pattern() {
        let patterns = &["signal.*", "action.*"];
        assert!(TopicMatcher::matches_any("action.send", patterns));
    }

    #[test]
    fn test_matches_any_no_match() {
        let patterns = &["signal.*", "action.*"];
        assert!(!TopicMatcher::matches_any("other.topic", patterns));
    }

    #[test]
    fn test_matches_any_empty_patterns() {
        let patterns: &[&str] = &[];
        assert!(!TopicMatcher::matches_any("signal.email", patterns));
    }

    // ===== Edge case tests =====

    #[test]
    fn test_empty_topic() {
        assert!(!topic_matches("", "signal.*"));
        assert!(!topic_matches("", "**"));
    }

    #[test]
    fn test_empty_pattern() {
        assert!(!topic_matches("signal", ""));
    }

    #[test]
    fn test_both_empty() {
        // Empty topic split by '.' gives [""], empty pattern gives [""]
        // This is an edge case - let's verify behavior
        assert!(topic_matches("", ""));
    }

    #[test]
    fn test_complex_pattern() {
        assert!(topic_matches(
            "signal.email.inbox.unread.important",
            "signal.**.important"
        ));
        assert!(topic_matches(
            "signal.email.important",
            "signal.**.important"
        ));
        assert!(!topic_matches("signal.important", "signal.**.important")); // ** needs 1+ segments
    }

    #[test]
    fn test_consecutive_wildcards() {
        // "**.**" should match 2+ segments
        assert!(topic_matches("signal.email", "**.**"));
        assert!(topic_matches("signal.email.received", "**.**"));
    }
}
