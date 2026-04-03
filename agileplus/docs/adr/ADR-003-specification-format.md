# ADR-003: Specification Format

**Date**: 2026-04-02  
**Status**: Accepted  
**Deciders**: Agent  

## Context

AgilePlus requires a structured, machine-readable format for specifications (PRDs, ADRs, work package definitions, and governance contracts). The format must balance human readability (for developers and AI agents editing specs) with machine parseability (for validation, code generation, and sync operations).

The specification format is the foundation of spec-driven development - it must support:
- Structured data extraction for work package generation
- Validation against schemas
- Version control diffability
- AI-friendly editing (minimal syntax noise)
- Cross-platform tooling

## Decision Drivers

- **Human readability**: Developers and agents must read and write specs easily
- **Machine parseability**: Tools must extract structured data without ambiguity
- **Schema validation**: Specs must be validateable against defined schemas
- **Version control friendly**: Clean diffs for code review
- **Multi-language support**: Rust, Python, and future language tooling
- **Type safety**: Schema should encode types and constraints
- **Extensibility**: New fields and sections must be addable without breaking existing tools

## Options Considered

### Option A: Markdown with YAML Frontmatter

```markdown
---
id: FEAT-001
title: User Authentication
status: draft
priority: p1
---

# Feature: User Authentication

## Overview
This feature adds...

## Requirements
- FR-001: System MUST support OAuth 2.0
- FR-002: System MUST support SAML
```

**Pros**:
- Human-readable prose in Markdown
- Structured metadata in YAML frontmatter
- Widely supported (Jekyll, Hugo, many tools)
- Good diffability for prose sections

**Cons**:
- Two formats to parse (YAML + Markdown)
- Prose sections not machine-readable
- Inconsistent structure across specs
- Validation requires two parsers

**Verdict**: Partial solution - metadata is structured but body is freeform

### Option B: Pure YAML

```yaml
spec_version: "1.0"
id: FEAT-001
title: User Authentication
status: draft
priority: p1

overview: |
  This feature adds comprehensive authentication
  support including OAuth 2.0 and SAML.

requirements:
  - id: FR-001
    title: OAuth 2.0 Support
    description: System MUST support OAuth 2.0 flows
    priority: p1
    acceptance_criteria:
      - Given valid credentials, When user logs in, Then access token issued

  - id: FR-002
    title: SAML Support
    description: System MUST support SAML 2.0
    priority: p2
    acceptance_criteria:
      - Given SAML configuration, When user logs in, Then SSO succeeds
```

**Pros**:
- Single format to parse
- Fully machine-readable
- Native schema validation (JSON Schema)
- Excellent type safety
- Hierarchical structure
- Native multi-line string support
- Comments supported

**Cons**:
- Less readable for long prose sections
- Requires editor YAML support for best experience
- Learning curve for non-technical stakeholders

**Verdict**: Selected as primary format

### Option C: TOML

```toml
spec_version = "1.0"
id = "FEAT-001"
title = "User Authentication"
status = "draft"
priority = "p1"

[overview]
text = """
This feature adds comprehensive authentication
support including OAuth 2.0 and SAML.
"""

[[requirements]]
id = "FR-001"
title = "OAuth 2.0 Support"
description = "System MUST support OAuth 2.0 flows"
priority = "p1"

[[requirements.acceptance_criteria]]
given = "valid credentials"
when = "user logs in"
then = "access token issued"
```

**Pros**:
- Very readable for simple configs
- Native date/time types
- Comments supported
- Popular in Rust ecosystem

**Cons**:
- Verbose for deeply nested structures
- Array syntax is awkward
- No standard schema validation
- Poor multi-line string handling
- Harder to generate programmatically

**Verdict**: Rejected for complex nested specs

### Option D: Org-mode

```org
#+ID: FEAT-001
#+TITLE: User Authentication
#+STATUS: draft
#+PRIORITY: p1

* Feature: User Authentication

** Overview
This feature adds...

** Requirements
*** FR-001: OAuth 2.0 Support
    - Given valid credentials
    - When user logs in
    - Then access token issued
```

**Pros**:
- Excellent Emacs integration
- Agenda and task tracking
- Plain text, very portable

**Cons**:
- Emacs-centric ecosystem
- Poor tooling outside Emacs
- No native schema validation
- Limited parser availability
- Steep learning curve for non-Emacs users

**Verdict**: Too niche, limited tooling

### Option E: JSON

```json
{
  "spec_version": "1.0",
  "id": "FEAT-001",
  "title": "User Authentication",
  "status": "draft",
  "priority": "p1",
  "overview": "This feature adds...",
  "requirements": [
    {
      "id": "FR-001",
      "title": "OAuth 2.0 Support",
      "description": "System MUST support OAuth 2.0 flows",
      "priority": "p1"
    }
  ]
}
```

**Pros**:
- Universal tooling support
- JSON Schema validation
- Easy programmatic generation

**Cons**:
- No comments (critical for specs)
- Verbose syntax
- Poor diffability
- No multi-line strings
- Hard to hand-edit

**Verdict**: Rejected - no comments is a dealbreaker

### Option F: XML

**Pros**:
- Schema validation (XSD)
- Mature tooling

**Cons**:
- Verbose
- Poor readability
- Declining ecosystem

**Verdict**: Rejected - too verbose, poor DX

## Decision

**Adopt YAML as the primary specification format** for all AgilePlus specifications.

### Rationale

1. **Single format**: One parser, one schema system
2. **Human + Machine readable**: Comments, multi-line strings, clean structure
3. **Schema validation**: JSON Schema works with YAML
4. **Tooling**: Excellent Rust (serde_yaml) and Python (PyYAML) support
5. **Diffability**: Line-oriented, works well with git
6. **Type safety**: Schemas enforce types and constraints
7. **Extensibility**: New fields don't break existing parsers

### Schema Design Principles

```yaml
# All specs share common metadata
spec_version: "1.0"          # Schema version for forward compatibility
id: "FEAT-001"              # Unique identifier
title: "User Authentication" # Human-readable title
status: draft | specified | planned | implementing | validated | shipped
created_at: "2026-04-02T10:00:00Z"
updated_at: "2026-04-02T10:00:00Z"
author: "agent" | "user" | "system"

# Type-specific sections follow
type: feature | adr | work_package | governance_contract

# Feature specs include:
overview: string              # Multi-line markdown prose
motivation: string            # Why this feature exists
clarifications: list          # Q&A from discovery
user_stories: list            # Structured user stories
requirements: list            # Functional requirements
acceptance_criteria: list     # Testable criteria
edge_cases: list              # Known edge cases
success_criteria: list        # Measurable outcomes
dependencies: list            # Other features/PRs this depends on
work_packages: list           # Associated WPs
governance: object            # Quality gates, evidence requirements

# ADRs include:
context: string               # Problem statement
decision_drivers: list        # Criteria for decision
options_considered: list      # Alternatives with pros/cons
decision: string              # Selected option
consequences: object          # Positive, negative, neutral

# Work packages include:
feature_id: string            # Parent feature
ordinal: integer              # Ordering within feature
description: string           # Detailed description
estimated_hours: integer    # Time estimate
actual_hours: integer         # Time spent
assignee: string             # Agent or human
dependencies: list           # Other WPs this blocks/blocked by
```

### Example Feature Spec

```yaml
# kitty-specs/FEAT-001-user-authentication/spec.yaml
spec_version: "1.0"
id: FEAT-001
title: User Authentication System
slug: user-authentication
type: feature
status: specified
priority: p1
created_at: "2026-04-02T10:00:00Z"
updated_at: "2026-04-02T10:00:00Z"
author: agent

overview: |
  Implement a comprehensive user authentication system supporting
  multiple identity providers. The system must handle OAuth 2.0,
  SAML 2.0, and local credential-based authentication.

  This is foundational for all user-facing features.

motivation: |
  Current system has no authentication, blocking any personalized
  features or security-sensitive operations.

clarifications:
  - question: Should we support social login?
    answer: Yes, Google and GitHub OAuth for v1. Others in v2.
    date: "2026-04-02"
    source: user_interview

user_stories:
  - id: US-001
    title: OAuth Login
    priority: p1
    description: |
      As a user, I want to log in with my Google account
      so that I don't need to create a new password.
    acceptance_criteria:
      - Given I have a valid Google account, When I click "Sign in with Google", Then I am authenticated
      - Given I deny OAuth consent, When the popup closes, Then I see an error message
    independent_test: |
      Can test OAuth flow independently using a test Google account.
      Does not require SAML implementation.

  - id: US-002
    title: SAML SSO
    priority: p2
    description: |
      As an enterprise admin, I want SAML SSO integration
      so my users can use corporate credentials.
    acceptance_criteria:
      - Given valid SAML metadata, When configured, Then users can SSO
    independent_test: |
      Requires SAML identity provider (test with Okta sandbox).

requirements:
  - id: FR-001
    title: OAuth 2.0 Authorization Code Flow
    category: authentication
    priority: p1
    description: |
      System MUST implement OAuth 2.0 Authorization Code Flow
      per RFC 6749 Section 4.1.
    acceptance_criteria:
      - id: AC-001
        given: valid authorization code
        when: token endpoint called
        then: access and refresh tokens returned
      - id: AC-002
        given: expired authorization code
        when: token endpoint called
        then: 400 error with "invalid_grant"

  - id: FR-002
    title: SAML 2.0 SP-Initiated SSO
    category: authentication
    priority: p2
    description: |
      System MUST act as SAML Service Provider supporting
      SP-initiated SSO per SAML 2.0 specification.
    acceptance_criteria:
      - id: AC-003
        given: valid SAMLResponse
        when: ACS endpoint receives POST
        then: user session created

edge_cases:
  - scenario: OAuth provider returns invalid token
    handling: Log error, display user-friendly message, offer retry
    detection: Token validation fails signature check
  - scenario: SAML assertion expired
    handling: Reject authentication, redirect to IdP for re-login
    detection: NotOnOrAfter condition check
  - scenario: Database unavailable during login
    handling: Return 503, queue for retry, show maintenance message
    detection: Connection pool exhaustion

success_criteria:
  - id: SC-001
    description: Users complete OAuth login in under 5 seconds
    measurement: p95 latency from click to authenticated
    target: "< 5s"
  - id: SC-002
    description: Authentication success rate > 99%
    measurement: Successful logins / Total attempts
    target: "> 99%"

dependencies:
  - type: feature
    id: FEAT-000
    relationship: requires
    description: Database schema for user storage
  - type: external
    id: "https://tools.ietf.org/html/rfc6749"
    relationship: complies_with
    description: OAuth 2.0 specification

governance:
  quality_gates:
    - name: test_coverage
      threshold: ">= 80%"
      evidence_required: true
    - name: security_scan
      tool: semgrep
      severity: error
      evidence_required: true
  evidence_requirements:
    - type: test_results
      required_for: [implementing, validated]
    - type: security_scan
      required_for: [validated]
    - type: code_review
      required_for: [implementing]

work_packages:
  - id: WP-001
    title: OAuth 2.0 Core Implementation
    status: planned
    estimated_hours: 16
  - id: WP-002
    title: SAML 2.0 Integration
    status: planned
    estimated_hours: 24
    dependencies: [WP-001]
```

### Example ADR

```yaml
# docs/adr/ADR-015-caching-strategy.yaml
spec_version: "1.0"
id: ADR-015
title: Caching Strategy for API Responses
type: adr
status: accepted
created_at: "2026-04-02T10:00:00Z"
deciders: [agent, tech_lead]

context: |
  API response times are exceeding 500ms for frequently accessed data.
  We need a caching layer to improve performance without compromising
  data consistency.

decision_drivers:
  - Performance: p95 latency must be < 100ms for cached reads
  - Consistency: Stale data must not exceed 5 minutes for user data
  - Complexity: Cache invalidation must be automatic, not manual
  - Cost: Infrastructure costs must not exceed $100/month

options_considered:
  - name: In-Memory Cache (DashMap)
    pros:
      - Zero external dependencies
      - Sub-millisecond access
      - Simple implementation
    cons:
      - No cross-instance sharing
      - Cache lost on restart
      - Memory pressure on single node
    verdict: Selected for single-node deployment

  - name: Redis
    pros:
      - Cross-instance cache sharing
      - Persistence options
      - Rich data structures
    cons:
      - Additional infrastructure
      - Network latency
      - Operational complexity
    verdict: Deferred for multi-node phase

decision: |
  Adopt DashMap for in-memory caching with TTL-based expiration.
  Implement write-through caching for reads, background refresh
  for hot keys.

consequences:
  positive:
    - p95 latency reduced from 500ms to 15ms
    - No new infrastructure required
    - Simple local development
  negative:
    - Cache is node-local only
    - Memory usage increases by ~500MB
    - Requires cache warming on deploy
  neutral:
    - Monitoring needed for cache hit rates

links:
  - type: relates_to
    id: ADR-003
    description: Database selection affects cache strategies
  - type: enables
    id: FEAT-042
    description: User dashboard requires fast API responses
```

### Example Work Package

```yaml
# kitty-specs/FEAT-001/work_packages/WP-001.yaml
spec_version: "1.0"
id: WP-001
title: OAuth 2.0 Core Implementation
type: work_package
feature_id: FEAT-001
ordinal: 1
status: planned
priority: p1

feature_branch: feat/FEAT-001-oauth-core

description: |
  Implement OAuth 2.0 Authorization Code Flow including:
  - Authorization endpoint (/oauth/authorize)
  - Token endpoint (/oauth/token)
  - Token validation middleware
  - Refresh token rotation

acceptance_criteria:
  - id: AC-001
    requirement_id: FR-001
    description: Authorization endpoint returns code for valid requests
  - id: AC-002
    requirement_id: FR-001
    description: Token endpoint exchanges code for tokens
  - id: AC-003
    requirement_id: FR-001
    description: Refresh tokens rotate on use

dependencies: []
blocks: [WP-002]

estimated_hours: 16
actual_hours: null

assignee: agent-claude
reviewer: coderabbit

evidence:
  - type: test_results
    path: tests/oauth_integration_test.rs
    status: pending
  - type: code_review
    pr_url: null
    status: pending
```

## Validation Approach

### JSON Schema Validation

```yaml
# schemas/feature-spec-schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://agileplus.io/schemas/feature-spec-v1.json",
  "type": "object",
  "required": ["spec_version", "id", "title", "type", "status"],
  "properties": {
    "spec_version": {
      "type": "string",
      "enum": ["1.0"]
    },
    "id": {
      "type": "string",
      "pattern": "^FEAT-[0-9]+$"
    },
    "title": {
      "type": "string",
      "minLength": 1,
      "maxLength": 200
    },
    "type": {
      "const": "feature"
    },
    "status": {
      "type": "string",
      "enum": ["draft", "specified", "planned", "implementing", "validated", "shipped"]
    },
    "priority": {
      "type": "string",
      "enum": ["p0", "p1", "p2", "p3"]
    },
    "user_stories": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/user_story"
      }
    },
    "requirements": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/requirement"
      }
    }
  },
  "definitions": {
    "user_story": {
      "type": "object",
      "required": ["id", "title", "description"],
      "properties": {
        "id": { "type": "string", "pattern": "^US-[0-9]+$" },
        "title": { "type": "string" },
        "description": { "type": "string" },
        "priority": { "type": "string", "enum": ["p0", "p1", "p2", "p3"] },
        "acceptance_criteria": {
          "type": "array",
          "items": { "type": "string" }
        }
      }
    },
    "requirement": {
      "type": "object",
      "required": ["id", "title", "description"],
      "properties": {
        "id": { "type": "string", "pattern": "^FR-[0-9]+$" },
        "title": { "type": "string" },
        "description": { "type": "string" },
        "category": { "type": "string" },
        "priority": { "type": "string", "enum": ["p0", "p1", "p2", "p3"] },
        "acceptance_criteria": {
          "type": "array",
          "items": {
            "$ref": "#/definitions/acceptance_criterion"
          }
        }
      }
    },
    "acceptance_criterion": {
      "type": "object",
      "required": ["id", "given", "when", "then"],
      "properties": {
        "id": { "type": "string", "pattern": "^AC-[0-9]+$" },
        "given": { "type": "string" },
        "when": { "type": "string" },
        "then": { "type": "string" }
      }
    }
  }
}
```

### Rust Implementation

```rust
// crates/agileplus-domain/src/spec/mod.rs
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct FeatureSpec {
    #[serde(rename = "spec_version")]
    pub spec_version: String,
    
    #[validate(regex(path = "FEATURE_ID_REGEX"))]
    pub id: String,
    
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    
    pub slug: String,
    
    #[serde(rename = "type")]
    pub spec_type: SpecType,
    
    pub status: FeatureStatus,
    
    pub priority: Priority,
    
    pub overview: String,
    
    pub motivation: Option<String>,
    
    pub user_stories: Vec<UserStory>,
    
    pub requirements: Vec<Requirement>,
    
    pub edge_cases: Vec<EdgeCase>,
    
    pub success_criteria: Vec<SuccessCriterion>,
    
    pub governance: Option<GovernanceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SpecType {
    Feature,
    Adr,
    WorkPackage,
    GovernanceContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct UserStory {
    #[validate(regex(path = "US_ID_REGEX"))]
    pub id: String,
    
    pub title: String,
    pub description: String,
    pub priority: Priority,
    
    #[serde(rename = "acceptance_criteria")]
    pub acceptance_criteria: Vec<String>,
    
    #[serde(rename = "independent_test")]
    pub independent_test: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Requirement {
    #[validate(regex(path = "FR_ID_REGEX"))]
    pub id: String,
    
    pub title: String,
    pub description: String,
    pub category: String,
    pub priority: Priority,
    
    #[serde(rename = "acceptance_criteria")]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AcceptanceCriterion {
    #[validate(regex(path = "AC_ID_REGEX"))]
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
}

impl FeatureSpec {
    /// Parse and validate a YAML spec file
    pub fn from_yaml(yaml: &str) -> Result<Self, SpecError> {
        let spec: FeatureSpec = serde_yaml::from_str(yaml)
            .map_err(|e| SpecError::Parse(e.to_string()))?;
        
        spec.validate()
            .map_err(|e| SpecError::Validation(e.to_string()))?;
        
        // Cross-reference validation
        spec.validate_cross_references()?;
        
        Ok(spec)
    }
    
    /// Validate that all references within the spec are valid
    fn validate_cross_references(&self) -> Result<(), SpecError> {
        // Check that all FR references in ACs exist
        for req in &self.requirements {
            for ac in &req.acceptance_criteria {
                if !ac.id.starts_with(&format!("{}-", req.id.replace("FR", "AC"))) {
                    return Err(SpecError::Validation(
                        format!("AC {} does not match FR {}", ac.id, req.id)
                    ));
                }
            }
        }
        
        // Check user story IDs are unique
        let mut seen = std::collections::HashSet::new();
        for us in &self.user_stories {
            if !seen.insert(&us.id) {
                return Err(SpecError::Validation(
                    format!("Duplicate user story ID: {}", us.id)
                ));
            }
        }
        
        Ok(())
    }
}
```

### Validation CLI

```bash
# Validate a spec file
agileplus spec validate kitty-specs/FEAT-001/spec.yaml

# Output:
# ✓ Schema validation passed
# ✓ Cross-reference validation passed
# ✓ FR-001 → AC-001-001 mapping valid
# ✓ All user stories have acceptance criteria
# 
# Spec is valid. 1 warning:
# - FR-003 has no acceptance criteria (optional but recommended)

# Validate with strict mode (warnings as errors)
agileplus spec validate --strict kitty-specs/FEAT-001/spec.yaml

# Generate markdown from YAML
agileplus spec render kitty-specs/FEAT-001/spec.yaml > kitty-specs/FEAT-001/spec.md

# Check all specs in project
agileplus spec validate --all
```

## Consequences

### Positive

- **Single format**: One parser, one schema to maintain
- **Type safety**: Schema enforces structure, catches errors early
- **Tooling**: Easy to build linting, formatting, and IDE support
- **Programmatic access**: Rust/Python can read/write specs natively
- **Version control**: Clean diffs, line-based history
- **Extensibility**: Adding new fields doesn't break existing tools

### Negative

- **Learning curve**: Team must learn YAML and schema constraints
- **Editor support**: Requires YAML language server for best experience
- **Prose readability**: Less natural than pure Markdown for narrative
- **Validation overhead**: Must run validation before accepting specs

### Neutral

- **Migration**: Existing Markdown specs can be semi-automatically converted
- **Bi-directional**: Can generate Markdown for human reading, YAML for machine
- **Standards**: Based on widely-used JSON Schema and YAML 1.2

## Migration Strategy

### Phase 1: Dual Format (Current)
- YAML is canonical source
- Markdown is generated for human reading
- `agileplus spec render` generates `.md` from `.yaml`

### Phase 2: YAML-First
- Markdown generation becomes optional
- GitHub/rendered view uses generated Markdown
- Editing happens in YAML

### Phase 3: Native YAML
- IDE plugins provide rich editing experience
- Markdown generation rarely needed
- Full YAML tooling ecosystem

## References

- YAML 1.2 Specification: https://yaml.org/spec/1.2.2/
- JSON Schema: https://json-schema.org/
- serde_yaml (Rust): https://github.com/dtolnay/serde-yaml
- schemars (Rust): https://github.com/GREsau/schemars
- PyYAML (Python): https://pyyaml.org/

---

*This ADR will be updated as the schema evolves*
