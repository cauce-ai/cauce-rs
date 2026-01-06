# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[###-feature-name]`  
**Created**: [DATE]  
**Status**: Draft  
**Input**: User description: "$ARGUMENTS"

## User Scenarios & Testing *(mandatory)*

<!--
  IMPORTANT: User stories should be PRIORITIZED as user journeys ordered by importance.
  Each user story/journey must be INDEPENDENTLY TESTABLE - meaning if you implement just ONE of them,
  you should still have a viable MVP (Minimum Viable Product) that delivers value.
  
  Assign priorities (P1, P2, P3, etc.) to each story, where P1 is the most critical.
  Think of each story as a standalone slice of functionality that can be:
  - Developed independently
  - Tested independently
  - Deployed independently
  - Demonstrated to users independently
-->

### User Story 1 - [Brief Title] (Priority: P1)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently - e.g., "Can be fully tested by [specific action] and delivers [specific value]"]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 2 - [Brief Title] (Priority: P2)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

### User Story 3 - [Brief Title] (Priority: P3)

[Describe this user journey in plain language]

**Why this priority**: [Explain the value and why it has this priority level]

**Independent Test**: [Describe how this can be tested independently]

**Acceptance Scenarios**:

1. **Given** [initial state], **When** [action], **Then** [expected outcome]

---

[Add more user stories as needed, each with an assigned priority]

### Edge Cases

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right edge cases.
-->

- What happens when [boundary condition]?
- How does system handle [error scenario]?

## Requirements *(mandatory)*

<!--
  ACTION REQUIRED: The content in this section represents placeholders.
  Fill them out with the right functional requirements.
-->

### Functional Requirements

- **FR-001**: System MUST [specific capability, e.g., "allow users to create accounts"]
- **FR-002**: System MUST [specific capability, e.g., "validate email addresses"]  
- **FR-003**: Users MUST be able to [key interaction, e.g., "reset their password"]
- **FR-004**: System MUST [data requirement, e.g., "persist user preferences"]
- **FR-005**: System MUST [behavior, e.g., "log all security events"]

*Example of marking unclear requirements:*

- **FR-006**: System MUST authenticate users via [NEEDS CLARIFICATION: auth method not specified - email/password, SSO, OAuth?]
- **FR-007**: System MUST retain user data for [NEEDS CLARIFICATION: retention period not specified]

### Key Entities *(include if feature involves data)*

- **[Entity 1]**: [What it represents, key attributes without implementation]
- **[Entity 2]**: [What it represents, relationships to other entities]

## Success Criteria *(mandatory)*

<!--
  ACTION REQUIRED: Define measurable success criteria.
  These must be technology-agnostic and measurable.
-->

### Measurable Outcomes

- **SC-001**: [Measurable metric, e.g., "Users can complete account creation in under 2 minutes"]
- **SC-002**: [Measurable metric, e.g., "System handles 1000 concurrent users without degradation"]
- **SC-003**: [User satisfaction metric, e.g., "90% of users successfully complete primary task on first attempt"]
- **SC-004**: [Business metric, e.g., "Reduce support tickets related to [X] by 50%"]

## Protocol Impact *(Cauce-specific)*

<!--
  ACTION REQUIRED: Assess how this feature affects the Cauce Protocol.
  Mark sections N/A if not applicable to this feature.
-->

### Schema Impact

<!--
  Per Constitution Principle II: Schema-Driven Contracts
  Core schemas are REQUIRED; payload schemas are extensible.
-->

| Schema | Change Type | Description |
|--------|-------------|-------------|
| `signal.schema.json` | [New/Modified/None] | [Description or N/A] |
| `action.schema.json` | [New/Modified/None] | [Description or N/A] |
| `jsonrpc.schema.json` | [New/Modified/None] | [Description or N/A] |
| `errors.schema.json` | [New/Modified/None] | [Description or N/A] |
| `methods/*.schema.json` | [New/Modified/None] | [Description or N/A] |
| `payloads/*.schema.json` | [New/Modified/None] | [Description or N/A] |

### Component Interactions

<!--
  Per Constitution Principle VI: Component Separation
  Map responsibilities to the correct component. No boundary violations.
-->

| Component | Responsibility in This Feature | NOT Responsible For |
|-----------|-------------------------------|---------------------|
| **Adapter** | [What adapters do for this feature] | [What they must NOT do] |
| **Hub** | [What hub does for this feature] | [What it must NOT do] |
| **Agent** | [What agents do for this feature] | [What they must NOT do] |

### Transport Considerations

<!--
  Per Constitution Principle IV: Transport Agnostic
  Message semantics MUST NOT change based on transport.
-->

| Transport | Supported | Notes |
|-----------|-----------|-------|
| WebSocket | [Yes/No/N/A] | [Any transport-specific considerations] |
| Server-Sent Events | [Yes/No/N/A] | [Any transport-specific considerations] |
| HTTP Polling | [Yes/No/N/A] | [Any transport-specific considerations] |
| Webhooks | [Yes/No/N/A] | [Any transport-specific considerations] |

**Semantic consistency**: [Confirm message meaning is identical across all supported transports]

### Wire Protocol

<!--
  Per Constitution Principle V: Interoperability
  JSON-RPC 2.0 compliance is REQUIRED.
-->

- **New methods**: [List any new JSON-RPC methods introduced, or "None"]
- **Modified methods**: [List any changed JSON-RPC methods, or "None"]
- **A2A impact**: [How this affects A2A endpoint, or "None"]
- **MCP impact**: [How this affects MCP interface, or "None"]

### Version Impact

<!--
  Per Constitution Principle IX: Semantic Versioning
-->

- **Change type**: [MAJOR (breaking) / MINOR (backward compatible) / PATCH (bug fix)]
- **Rationale**: [Why this version bump is appropriate]
