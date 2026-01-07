# Research: Documentation Structure

**Feature**: 003-docs-setup
**Date**: 2026-01-06

## Overview

This research covers best practices for Rust project documentation, specifically CONTRIBUTING.md structure, architecture documentation, and docs/ organization.

---

## Decision 1: CONTRIBUTING.md Location

**Decision**: Place CONTRIBUTING.md at repository root

**Rationale**: GitHub automatically links to CONTRIBUTING.md in the repository root when users open issues or pull requests. This is the standard convention for open-source projects.

**Alternatives considered**:
- docs/CONTRIBUTING.md - Less discoverable, GitHub won't auto-link
- .github/CONTRIBUTING.md - Valid but less visible to casual browsers

---

## Decision 2: CONTRIBUTING.md Structure

**Decision**: Follow standard open-source contribution guide structure

**Rationale**: Well-established patterns make it easier for contributors who have worked on other projects.

**Structure**:
1. Code of Conduct reference (if applicable)
2. Getting Started (prerequisites, setup)
3. Development Workflow (branch naming, commits, PRs)
4. Code Standards (formatting, linting, testing)
5. Pull Request Process
6. Issue Reporting

**Alternatives considered**:
- Minimal single-page guide - Too sparse for a protocol implementation
- Multi-file contribution docs - Overkill for current project stage

---

## Decision 3: Commit Message Convention

**Decision**: Use Conventional Commits format

**Rationale**:
- Enables automated changelog generation
- Provides semantic meaning to commits
- Industry standard for Rust projects

**Format**:
```
type(scope): description

[optional body]

[optional footer]
```

**Types**: feat, fix, docs, style, refactor, test, chore, ci

**Alternatives considered**:
- Free-form messages - Less structured, harder to parse
- Angular commit format - Conventional Commits is the evolved standard

---

## Decision 4: ARCHITECTURE.md Location

**Decision**: Place ARCHITECTURE.md in docs/ directory

**Rationale**:
- Detailed technical documentation belongs in docs/
- Keeps repository root clean
- Allows for related architecture files (diagrams, ADRs) in same location

**Alternatives considered**:
- Repository root - Clutters root with technical docs
- docs/architecture/ARCHITECTURE.md - Unnecessary nesting

---

## Decision 5: Architecture Diagram Format

**Decision**: Use Mermaid diagrams embedded in Markdown

**Rationale**:
- GitHub renders Mermaid natively (no external tools needed)
- Version-controlled as text (diffable)
- Easy to update as architecture evolves
- No external image hosting required

**Diagram types**:
- Crate dependency diagram (flowchart)
- Layer diagram showing component boundaries

**Alternatives considered**:
- PNG/SVG images - Requires external tooling, not easily editable
- PlantUML - Less native GitHub support
- ASCII art - Limited expressiveness for complex relationships

---

## Decision 6: docs/ Directory Structure

**Decision**: Use category-based subdirectories

**Structure**:
```
docs/
├── README.md           # Index/navigation
├── ARCHITECTURE.md     # Crate structure and boundaries
├── architecture/       # Future: detailed arch docs, ADRs
├── guides/             # Future: how-to guides
└── reference/          # Future: API reference, config reference
```

**Rationale**:
- Prepares for documentation growth
- Common pattern in Rust ecosystem (tokio, axum, etc.)
- Clear categorization for different doc types

**Alternatives considered**:
- Flat docs/ - Doesn't scale as docs grow
- Topic-based (per-crate) - Better for API docs, not user-facing docs

---

## Decision 7: Crate Diagram Content

**Decision**: Document all planned crates from TODO.md with status indicators

**Crates to document**:
| Crate | Status | Description |
|-------|--------|-------------|
| cauce-core | Planned | Protocol types, schemas, validation |
| cauce-client-sdk | Planned | Client library for connecting to Hub |
| cauce-server-sdk | Planned | Server library for building Hubs |
| cauce-hub | Planned | Reference Hub implementation |
| cauce-cli | Planned | Hub management CLI |
| cauce-agent-cli | Planned | Interactive agent REPL |
| cauce-adapter-echo | Planned | Testing adapter |
| cauce-adapter-cli | Planned | Stdin/stdout adapter |

**Rationale**: Shows full vision while indicating current implementation status

**Alternatives considered**:
- Only implemented crates - Loses architectural context
- Separate roadmap document - Fragments information

---

## Decision 8: Coverage Requirement Documentation

**Decision**: Prominently document 95% coverage requirement in CONTRIBUTING.md

**Rationale**: Constitution Principle XI mandates 95% coverage. Contributors need to understand this upfront to avoid rejected PRs.

**Location**: Development Workflow section with link to CI configuration

**Alternatives considered**:
- Separate testing guide - Fragments critical information
- Only in CI config - Not discoverable for new contributors

---

## Decision 9: Branch Naming Convention

**Decision**: Document SpecKit pattern (###-feature-name)

**Rationale**: Already established by project tooling. Consistency with existing branches (001-repo-setup, 002-ci-cd-pipeline).

**Format**: `NNN-short-description` where NNN is zero-padded feature number

**Alternatives considered**:
- GitHub flow (feature/xxx) - Doesn't match existing pattern
- Conventional branch names - Would conflict with SpecKit tooling

---

## Decision 10: PR Process Documentation

**Decision**: Document squash-merge workflow with PR template reference

**Rationale**:
- Squash merge keeps history clean
- PR template ensures consistent descriptions
- Matches established workflow from previous PRs

**Steps to document**:
1. Create feature branch (via SpecKit or manually)
2. Implement with atomic commits
3. Push and create PR
4. Wait for CI (all checks must pass)
5. Squash merge when approved

**Alternatives considered**:
- Merge commits - Creates noisy history
- Rebase and merge - More complex for contributors

---

## Summary

All documentation decisions align with:
- Rust ecosystem conventions
- GitHub platform capabilities (native Mermaid rendering)
- Project-specific requirements (95% coverage, SpecKit workflow)
- Constitution principles (Spec-First, TDD)

No NEEDS CLARIFICATION items remain.
