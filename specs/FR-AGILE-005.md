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
