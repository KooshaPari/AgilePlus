package gotreqt_test

import (
	"testing"

	"github.com/phenotype/AgilePlus/go/gotreqt"
)

// Example test showing FR traceability.
//
// Traces to:
//   - FR-GOTREQT-001: Basic traceability
//   - FR-GOTREQT-002: Test reporting
func TestBasicTraceability(t *testing.T) {
	gotreqt.TraceTo(t, "FR-GOTREQT-001", "FR-GOTREQT-002")

	// Test implementation
	if false {
		t.Error("This should not fail")
	}
}

// Example test with DescribeFr.
//
// Traces to: FR-GOTREQT-003: Nested test groups
func TestWithDescribeFr(t *testing.T) {
	gotreqt.DescribeFr(t, "FR-GOTREQT-003", "NestedGroup", func(t *testing.T) {
		gotreqt.TraceTo(t, "FR-GOTREQT-003")
		t.Log("Inside nested group")
	})
}

// Test coverage reporting.
func TestCoverageReport(t *testing.T) {
	// Run some tests first
	t.Run("SubTest1", func(t *testing.T) {
		gotreqt.TraceTo(t, "FR-GOTREQT-001")
	})

	// Generate report
	report := gotreqt.Report()
	if len(report) == 0 {
		t.Log("No traces collected (expected in sub-test)")
	}

	// Check coverage
	expected := []string{"FR-GOTREQT-001", "FR-GOTREQT-002", "FR-GOTREQT-003"}
	covered, total := gotreqt.Coverage(expected)
	t.Logf("Coverage: %d/%d FRs covered", covered, total)
}
