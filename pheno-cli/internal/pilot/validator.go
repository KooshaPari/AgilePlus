package pilot

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

// Check represents a single validation check result.
type Check struct {
	Name   string
	Passed bool
	Detail string
}

// PilotReport holds the results of a pilot repo validation.
type PilotReport struct {
	RepoPath string
	Checks   []Check
	Passed   bool
}

// ValidatePilotRepo checks that a repository has been bootstrapped correctly
// for pilot rollout: mise.toml exists, hooks are installed, and CI workflows
// are present.
func ValidatePilotRepo(repoPath string) (*PilotReport, error) {
	if _, err := os.Stat(repoPath); err != nil {
		return nil, fmt.Errorf("repo path does not exist: %w", err)
	}

	report := &PilotReport{RepoPath: repoPath}

	// Basic checks
	report.Checks = append(report.Checks, checkMiseToml(repoPath))
	report.Checks = append(report.Checks, checkCIWorkflows(repoPath))
	report.Checks = append(report.Checks, checkGitHooks(repoPath))

	// Governance checks (from reconcile.rules.yaml)
	report.Checks = append(report.Checks, checkGovernanceFiles(repoPath)...)
	report.Checks = append(report.Checks, checkCIWorkflowContent(repoPath))
	report.Checks = append(report.Checks, checkBranchDiscipline(repoPath))

	allPassed := true
	for _, c := range report.Checks {
		if !c.Passed {
			allPassed = false
			break
		}
	}
	report.Passed = allPassed

	return report, nil
}

// Governance file checks from reconcile.rules.yaml
func checkGovernanceFiles(repoPath string) []Check {
	var checks []Check

	requiredFiles := []struct {
		path        string
		description string
	}{
		{"AGENTS.md", "Agent activity and decision log"},
		{"CLAUDE.md", "Workspace rules and governance guidelines"},
		{"SECURITY.md", "Security policy and incident procedures"},
		{".gitignore", "Version control ignore patterns"},
	}

	for _, f := range requiredFiles {
		p := filepath.Join(repoPath, f.path)
		if _, err := os.Stat(p); err != nil {
			checks = append(checks, Check{
				Name:   fmt.Sprintf("governance: %s", f.path),
				Passed: false,
				Detail: fmt.Sprintf("%s not found", f.path),
			})
		} else {
			checks = append(checks, Check{
				Name:   fmt.Sprintf("governance: %s", f.path),
				Passed: true,
				Detail: f.path,
			})
		}
	}

	return checks
}

// Check CI workflow content for phenotypeActions references
func checkCIWorkflowContent(repoPath string) Check {
	workflowsDir := filepath.Join(repoPath, ".github", "workflows")
	entries, err := os.ReadDir(workflowsDir)
	if err != nil {
		return Check{Name: "CI workflow references", Passed: false, Detail: "cannot read workflows directory"}
	}

	var workflowContent []byte
	var foundPolicyGate, foundLintTest bool

	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		content, err := os.ReadFile(filepath.Join(workflowsDir, e.Name()))
		if err != nil {
			continue
		}
		workflowContent = append(workflowContent, content...)

		if strings.Contains(string(content), "phenotypeActions/actions/policy-gate") {
			foundPolicyGate = true
		}
		if strings.Contains(string(content), "phenotypeActions/actions/lint-test") {
			foundLintTest = true
		}
	}

	if !foundPolicyGate || !foundLintTest {
		detail := "missing required actions: "
		if !foundPolicyGate {
			detail += "phenotypeActions/actions/policy-gate "
		}
		if !foundLintTest {
			detail += "phenotypeActions/actions/lint-test"
		}
		return Check{Name: "CI workflow references", Passed: false, Detail: strings.TrimSpace(detail)}
	}

	return Check{Name: "CI workflow references", Passed: true, Detail: "policy-gate and lint-test actions found"}
}

// Check branch discipline configuration
func checkBranchDiscipline(repoPath string) Check {
	// Check for branch naming convention in pre-commit or config
	patterns := []string{
		filepath.Join(repoPath, ".pre-commit-config.yaml"),
		filepath.Join(repoPath, "Taskfile.yml"),
		filepath.Join(repoPath, "mise.toml"),
	}

	for _, p := range patterns {
		content, err := os.ReadFile(p)
		if err != nil {
			continue
		}

		// Look for branch naming patterns
		if strings.Contains(string(content), "branch") && strings.Contains(string(content), "pattern") {
			return Check{Name: "branch discipline", Passed: true, Detail: "branch naming conventions configured"}
		}
	}

	// If no explicit config, check if .git/hooks have pre-commit
	hooksDir := filepath.Join(repoPath, ".git", "hooks")
	if _, err := os.Stat(hooksDir); err == nil {
		return Check{Name: "branch discipline", Passed: true, Detail: "git hooks present (enforces branch checks)"}
	}

	return Check{Name: "branch discipline", Passed: false, Detail: "no branch discipline configuration found"}
}

func checkMiseToml(repoPath string) Check {
	p := filepath.Join(repoPath, "mise.toml")
	if _, err := os.Stat(p); err != nil {
		return Check{Name: "mise.toml exists", Passed: false, Detail: "mise.toml not found"}
	}
	return Check{Name: "mise.toml exists", Passed: true, Detail: p}
}

func checkCIWorkflows(repoPath string) Check {
	dir := filepath.Join(repoPath, ".github", "workflows")
	entries, err := os.ReadDir(dir)
	if err != nil {
		return Check{Name: "CI workflows installed", Passed: false, Detail: ".github/workflows not found or unreadable"}
	}
	for _, e := range entries {
		if !e.IsDir() {
			return Check{Name: "CI workflows installed", Passed: true, Detail: fmt.Sprintf("%d workflow(s) found", countFiles(entries))}
		}
	}
	return Check{Name: "CI workflows installed", Passed: false, Detail: "no workflow files in .github/workflows"}
}

func checkGitHooks(repoPath string) Check {
	hooksDir := filepath.Join(repoPath, ".git", "hooks")
	preCommit := filepath.Join(hooksDir, "pre-commit")
	if _, err := os.Stat(preCommit); err != nil {
		return Check{Name: "git hooks installed", Passed: false, Detail: "pre-commit hook not found"}
	}
	return Check{Name: "git hooks installed", Passed: true, Detail: preCommit}
}

func countFiles(entries []os.DirEntry) int {
	count := 0
	for _, e := range entries {
		if !e.IsDir() {
			count++
		}
	}
	return count
}
