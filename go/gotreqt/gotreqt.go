// Package gotreqt provides traceability support for Go tests.
//
// Use this package to mark tests as tracing to Feature Requirements (FRs).
//
// Example:
//
//	func TestFeature(t *testing.T) {
//		gotreqt.TraceTo(t, "FR-EXAMPLE-001")
//		// test code
//	}
//
//	func TestFeatureWithMultipleFRs(t *testing.T) {
//		gotreqt.TraceTo(t, "FR-EXAMPLE-002", "FR-EXAMPLE-003")
//		// test code
//	}
package gotreqt

import (
	"fmt"
	"os"
	"strings"
	"testing"
)

// TraceTo marks a test as tracing to one or more Feature Requirement IDs.
//
// Usage:
//
//	func TestSomething(t *testing.T) {
//		gotreqt.TraceTo(t, "FR-EXAMPLE-001")
//		// ... test code
//	}
//
// Multiple FRs can be specified:
//
//	gotreqt.TraceTo(t, "FR-EXAMPLE-001", "FR-EXAMPLE-002")
func TraceTo(t *testing.T, frIDs ...string) {
	t.Helper()

	// Validate FR ID format
	for _, frID := range frIDs {
		if !isValidFRID(frID) {
			t.Errorf("Invalid FR ID format: %s (expected FR-XXXX-NNN)", frID)
			continue
		}
	}

	// Log trace information if verbose
	if testing.Verbose() {
		fmt.Printf("  [TRACE] %s traces to: %s\n", t.Name(), strings.Join(frIDs, ", "))
	}

	// Store in environment for collection during CI
	key := fmt.Sprintf("TRACE_%s", t.Name())
	value := strings.Join(frIDs, ",")
	os.Setenv(key, value)
}

// DescribeFr creates a test sub-group for a specific FR.
//
// Usage:
//
//	func TestFeatureGroup(t *testing.T) {
//		gotreqt.DescribeFr(t, "FR-EXAMPLE-001", "Feature Description", func(t *testing.T) {
//			// test code
//		})
//	}
func DescribeFr(t *testing.T, frID string, description string, fn func(*testing.T)) {
	t.Helper()
	t.Run(fmt.Sprintf("%s_%s", frID, description), fn)
}

// isValidFRID checks if the FR ID matches the expected format.
func isValidFRID(frID string) bool {
	// Format: FR-XXXX-NNN or FR-XXXX-NNN-YYY
	parts := strings.Split(frID, "-")
	if len(parts) < 3 {
		return false
	}
	if parts[0] != "FR" {
		return false
	}
	// Project code should be uppercase letters/digits
	for _, c := range parts[1] {
		if !((c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9')) {
			return false
		}
	}
	return true
}

// Report generates a traceability report for collected FR data.
//
// This is typically called from TestMain or a dedicated test.
func Report() map[string][]string {
	result := make(map[string][]string)

	for _, e := range os.Environ() {
		if strings.HasPrefix(e, "TRACE_") {
			parts := strings.SplitN(e, "=", 2)
			if len(parts) == 2 {
				testName := strings.TrimPrefix(parts[0], "TRACE_")
				frIDs := strings.Split(parts[1], ",")
				result[testName] = frIDs
			}
		}
	}

	return result
}

// Coverage calculates coverage metrics from a test run.
//
// Returns the number of unique FRs covered and total FRs expected.
func Coverage(expectedFRs []string) (covered int, total int) {
	total = len(expectedFRs)

	report := Report()
	coveredFRs := make(map[string]bool)

	for _, frIDs := range report {
		for _, frID := range frIDs {
			coveredFRs[frID] = true
		}
	}

	for _, fr := range expectedFRs {
		if coveredFRs[fr] {
			covered++
		}
	}

	return covered, total
}
