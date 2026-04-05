---
id: FR-AGILE-005
title: HeliosApp Completion
status: draft
priority: P1
created: 2026-03-05
category: application
owner: phenotype-org
source: kitty-specs/005-heliosapp-completion
---

# FR-AGILE-005: HeliosApp Completion

## Description

Complete the HeliosApp (helMo) iOS/macOS application for spec-driven development on mobile devices, enabling field developers to capture requirements, track progress, and sync with AgilePlus.

## Objectives

- Achieve App Store release readiness
- Implement core spec capture workflows
- Enable offline-first operation with sync
- Support voice-to-spec transcription
- Provide photo/video evidence capture

## User Stories

### US-1: Field Developer Captures Requirements (P0)
**Given** a developer at a customer site without laptop,  
**When** they open HeliosApp and describe requirements via voice,  
**Then** the app transcribes voice to structured FR spec with photo evidence.

### US-2: Offline Work with Sync (P1)
**Given** a developer working in an area with no connectivity,  
**When** they create specs and plans offline,  
**Then** all changes sync automatically when connectivity is restored without conflicts.

### US-3: Evidence Capture in Field (P1)
**Given** a developer observing a bug in production,  
**When** they capture photo/video evidence via HeliosApp,  
**Then** the evidence is attached to the FR spec with geolocation and timestamp.

### US-4: Push Notification for Blockers (P2)
**Given** a work package the developer is assigned to,  
**When** a blocker is identified by another team member,  
**Then** the developer receives a push notification with blocker details and suggested resolution.

## Acceptance Criteria

- [ ] iOS app with core AgilePlus workflows
- [ ] macOS app with full feature parity
- [ ] Offline operation with background sync
- [ ] Voice-to-spec transcription
- [ ] Photo/video evidence attachment
- [ ] Biometric authentication
- [ ] Push notifications for blockers

## Work Packages

| WP | Title | Status |
|----|-------|--------|
| WP-001 | iOS App Core | planned |
| WP-002 | macOS App | planned |
| WP-003 | Offline Sync | planned |
| WP-004 | Voice Transcription | planned |
| WP-005 | Evidence Capture | planned |

## Dependencies

- FR-AGILE-003 (Platform, sync)
- SwiftUI, Combine
- iOS 17+, macOS 14+

## Traceability

- Test Framework: XCTest
- Coverage Target: ≥75%

## Notes

Original: `kitty-specs/005-heliosapp-completion/`
Repository: helMo
