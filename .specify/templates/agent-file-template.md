# [PROJECT NAME] Development Guidelines

Auto-generated from all feature plans. Last updated: [DATE]

## Active Technologies

[EXTRACTED FROM ALL PLAN.MD FILES]

## Project Structure

```text
[ACTUAL STRUCTURE FROM PLANS]
```

## Commands

[ONLY COMMANDS FOR ACTIVE TECHNOLOGIES]

## Code Style

[LANGUAGE-SPECIFIC, ONLY FOR LANGUAGES IN USE]

## Recent Changes

[LAST 3 FEATURES AND WHAT THEY ADDED]

## Protocol Principles

**Cauce Protocol**: See `.specify/memory/constitution.md`

Key development requirements:
- **TDD Required**: Tests before code, 95% coverage (Principle XI)
- **Spec-First**: Behavior defined in spec before implementation (Principle I)
- **Schema-Driven**: JSON Schemas for all protocol messages (Principle II)
- **Component Separation**: Adapter/Hub/Agent boundaries (Principle VI)

Important constraints:
- All PRs must pass 95% coverage threshold
- Schema changes require spec update first
- Adapters: translation only, no AI logic
- Hub: routing only, no business logic
- Agents: AI decisions, no protocol details

Full constitution: `.specify/memory/constitution.md`

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
