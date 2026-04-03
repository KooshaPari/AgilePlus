package pilot

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"gopkg.in/yaml.v3"
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

// ReconcileRules represents the full reconcile.rules.yaml structure.
type ReconcileRules struct {
	Version            string                  `yaml:"version"`
	Modes             map[string]ModeConfig  `yaml:"modes"`
	Ownership         OwnershipConfig          `yaml:"ownership"`
	Governance        GovernanceConfig        `yaml:"governance"`
	CIWorkflows       CIWorkflowsConfig      `yaml:"ci_workflows"`
	Logging           LoggingConfig           `yaml:"logging"`
	CodeQuality       CodeQualityConfig      `yaml:"code_quality"`
	BranchDiscipline  BranchDisciplineConfig `yaml:"branch_discipline"`
	CommitMessages    CommitMessagesConfig   `yaml:"commit_messages"`
	DirectoryStructure DirectoryConfig        `yaml:"directory_structure"`
	Dependencies      DependenciesConfig     `yaml:"dependencies"`
	Documentation     DocumentationConfig    `yaml:"documentation"`
	Enforcement       EnforcementConfig     `yaml:"enforcement"`
}

type ModeConfig struct {
	CreateMissing         bool   `yaml:"create_missing"`
	UpdateTemplateOwned   bool   `yaml:"update_template_owned_if_unchanged"`
	CreateConflictSidecar bool   `yaml:"create_conflict_sidecar"`
	SidecarSuffix         string `yaml:"sidecar_suffix"`
	ReplaceTemplateOwned  bool   `yaml:"replace_template_owned"`
	ReportOnly           bool   `yaml:"report_only"`
}

type OwnershipConfig struct {
	TemplateOwned []string `yaml:"template_owned"`
	Protected     []string `yaml:"protected"`
}

type GovernanceConfig struct {
	RequiredFiles []RequiredFile `yaml:"required_files"`
}

type RequiredFile struct {
	Path        string `yaml:"path"`
	Template    string `yaml:"template,omitempty"`
	Check       string `yaml:"check"`
	Description string `yaml:"description"`
}

type CIWorkflowsConfig struct {
	Required []CIWorkflow `yaml:"required"`
	Optional []CIWorkflow `yaml:"optional"`
}

type CIWorkflow struct {
	Name        string   `yaml:"name"`
	Source      string   `yaml:"source"`
	Check       string   `yaml:"check"`
	Trigger     string   `yaml:"trigger"`
	Description string   `yaml:"description"`
	Branches    []string `yaml:"branches,omitempty"`
}

type LoggingConfig struct {
	Schema LoggingSchema `yaml:"schema"`
}

type LoggingSchema struct {
	RequiredFields   []string          `yaml:"required_fields"`
	Format          string            `yaml:"format"`
	TimestampFormat string            `yaml:"timestamp_format"`
	LevelValues     []string          `yaml:"level_values"`
	Example         map[string]string `yaml:"example"`
}

type CodeQualityConfig struct {
	MaxFunctionLines int              `yaml:"max_function_lines"`
	MinCoverage      int              `yaml:"min_coverage"`
	LintZeroErrors   bool             `yaml:"lint_zero_errors"`
	LintMaxWarnings  int              `yaml:"lint_max_warnings"`
	Standards        LanguageStandards `yaml:"standards"`
}

type LanguageStandards struct {
	Go         LanguageStandard `yaml:"go"`
	Python     LanguageStandard `yaml:"python"`
	TypeScript LanguageStandard `yaml:"typescript"`
	Rust       LanguageStandard `yaml:"rust"`
}

type LanguageStandard struct {
	MinCoverage   int      `yaml:"min_coverage"`
	Linters       []string `yaml:"linters"`
	Format        string   `yaml:"format,omitempty"`
	TestFramework string   `yaml:"test_framework,omitempty"`
	Clippy        string   `yaml:"clippy,omitempty"`
}

type BranchDisciplineConfig struct {
	Canonical           []string `yaml:"canonical"`
	ProtectedBranches   []string `yaml:"protected_branches"`
	RequirePRReviews    int      `yaml:"require_pr_reviews"`
	RequireStatusChecks bool     `yaml:"require_status_checks"`
	EnforceAdmin        bool     `yaml:"enforce_admin"`
	DismissStaleReviews bool     `yaml:"dismiss_stale_reviews"`
}

type CommitMessagesConfig struct {
	Format           string   `yaml:"format"`
	Types            []string `yaml:"types"`
	MaxSubjectLength int      `yaml:"max_subject_length"`
	RequireBody      bool     `yaml:"require_body"`
	BodyWrapLength   int      `yaml:"body_wrap_length"`
	FooterReferences []string `yaml:"footer_references"`
}

type DirectoryConfig struct {
	RequiredDirs  []string `yaml:"required_dirs"`
	StandardFiles []string `yaml:"standard_files"`
}

type DependenciesConfig struct {
	VulnerabilityCheck bool `yaml:"vulnerability_check"`
	OutdatedCheck      bool `yaml:"outdated_check"`
	MaxOutdatedAgeDays int  `yaml:"max_outdated_age_days"`
	RequireLockFiles   bool `yaml:"require_lock_files"`
}

type DocumentationConfig struct {
	Readme           ReadmeConfig    `yaml:"readme"`
	APIDocs          DocRequirement `yaml:"api_docs"`
	ArchitectureDocs DocRequirement `yaml:"architecture_docs"`
}

type ReadmeConfig struct {
	Required    bool     `yaml:"required"`
	MinSections []string `yaml:"min_sections"`
}

type DocRequirement struct {
	Required     bool   `yaml:"required"`
	WhenApplies string `yaml:"when_applicable"`
}

type EnforcementConfig struct {
	Governance    string `yaml:"governance"`
	CodeQuality   string `yaml:"code_quality"`
	Documentation string `yaml:"documentation"`
	Security      string `yaml:"security"`
	Dependencies  string `yaml:"dependencies"`
}

// ValidatePilotRepo checks that a repository has been bootstrapped correctly
// for pilot rollout: mise.toml exists, hooks are installed, and CI workflows
// are present.
func ValidatePilotRepo(repoPath string) (*PilotReport, error) {
	return ValidatePilotRepoWithRules(repoPath, "")
}

// ValidatePilotRepoWithRules validates using reconcile.rules.yaml if provided.
func ValidatePilotRepoWithRules(repoPath string, rulesDir string) (*PilotReport, error) {
	if _, err := os.Stat(repoPath); err != nil {
		return nil, fmt.Errorf("repo path does not exist: %w", err)
	}

	report := &PilotReport{RepoPath: repoPath}

	// Load reconcile rules if provided
	var rules *ReconcileRules
	if rulesDir != "" {
		rulesPath := filepath.Join(rulesDir, "reconcile.rules.yaml")
		if data, err := os.ReadFile(rulesPath); err == nil {
			var r ReconcileRules
			if err := yaml.Unmarshal(data, &r); err == nil {
				rules = &r
			}
		}
	}

	// Basic checks
	report.Checks = append(report.Checks, checkMiseToml(repoPath))
	report.Checks = append(report.Checks, checkCIWorkflows(repoPath))
	report.Checks = append(report.Checks, checkGitHooks(repoPath))

	// Governance checks (from reconcile.rules.yaml)
	report.Checks = append(report.Checks, checkGovernanceFiles(repoPath, rules)...)
	report.Checks = append(report.Checks, checkCIWorkflowContent(repoPath, rules))
	report.Checks = append(report.Checks, checkBranchDiscipline(repoPath, rules))
	report.Checks = append(report.Checks, checkCommitMessageStandards(repoPath, rules))
	report.Checks = append(report.Checks, checkDirectoryStructure(repoPath, rules))
	report.Checks = append(report.Checks, checkDependencyManagement(repoPath, rules))
	report.Checks = append(report.Checks, checkDocumentation(repoPath, rules))

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
func checkGovernanceFiles(repoPath string, rules *ReconcileRules) []Check {
	var checks []Check

	// Default required governance files
	requiredFiles := []struct {
		path        string
		description string
	}{
		{"AGENTS.md", "Agent activity and decision log"},
		{"CLAUDE.md", "Workspace rules and governance guidelines"},
		{"SECURITY.md", "Security policy and incident procedures"},
		{".gitignore", "Version control ignore patterns"},
	}

	// If rules are provided, use those instead
	if rules != nil && len(rules.Governance.RequiredFiles) > 0 {
		requiredFiles = nil
		for _, f := range rules.Governance.RequiredFiles {
			requiredFiles = append(requiredFiles, struct {
				path        string
				description string
			}{f.Path, f.Description})
		}
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
func checkCIWorkflowContent(repoPath string, rules *ReconcileRules) Check {
	workflowsDir := filepath.Join(repoPath, ".github", "workflows")
	entries, err := os.ReadDir(workflowsDir)
	if err != nil {
		return Check{Name: "CI workflow references", Passed: false, Detail: "cannot read workflows directory"}
	}

	// Default required actions
	requiredActions := map[string]bool{
		"phenotypeActions/actions/policy-gate": false,
		"phenotypeActions/actions/lint-test":   false,
	}

	// If rules provided, use those
	if rules != nil {
		requiredActions = nil
		for _, wf := range rules.CIWorkflows.Required {
			requiredActions[wf.Source] = false
		}
	}

	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		content, err := os.ReadFile(filepath.Join(workflowsDir, e.Name()))
		if err != nil {
			continue
		}

		for action := range requiredActions {
			if strings.Contains(string(content), action) {
				requiredActions[action] = true
			}
		}
	}

	var missing []string
	for action, found := range requiredActions {
		if !found {
			missing = append(missing, action)
		}
	}

	if len(missing) > 0 {
		return Check{Name: "CI workflow references", Passed: false, Detail: fmt.Sprintf("missing: %s", strings.Join(missing, ", "))}
	}

	return Check{Name: "CI workflow references", Passed: true, Detail: "all required actions found"}
}

// Check branch discipline configuration
func checkBranchDiscipline(repoPath string, rules *ReconcileRules) Check {
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

	// If rules provided, check for canonical branches
	if rules != nil && len(rules.BranchDiscipline.Canonical) > 0 {
		for _, canonical := range rules.BranchDiscipline.Canonical {
			gitDir := filepath.Join(repoPath, ".git")
			headFile := filepath.Join(gitDir, "HEAD")
			content, err := os.ReadFile(headFile)
			if err == nil && strings.Contains(string(content), "ref: refs/heads/"+canonical) {
				return Check{Name: "branch discipline", Passed: true, Detail: fmt.Sprintf("canonical branch '%s' found", canonical)}
			}
		}
	}

	// If no explicit config, check if .git/hooks have pre-commit
	hooksDir := filepath.Join(repoPath, ".git", "hooks")
	if _, err := os.Stat(hooksDir); err == nil {
		return Check{Name: "branch discipline", Passed: true, Detail: "git hooks present (enforces branch checks)"}
	}

	return Check{Name: "branch discipline", Passed: false, Detail: "no branch discipline configuration found"}
}

// Check commit message format standards
func checkCommitMessageStandards(repoPath string, rules *ReconcileRules) Check {
	format := "conventional commits"

	if rules != nil && rules.CommitMessages.Format != "" {
		format = rules.CommitMessages.Format
	}

	// Check for commitlint config or similar
	commitlintPaths := []string{
		filepath.Join(repoPath, ".commitlintrc.json"),
		filepath.Join(repoPath, ".commitlintrc.yaml"),
		filepath.Join(repoPath, ".commitlintrc.yml"),
		filepath.Join(repoPath, "commitlint.config.js"),
	}

	for _, p := range commitlintPaths {
		if _, err := os.Stat(p); err == nil {
			return Check{Name: "commit message format", Passed: true, Detail: fmt.Sprintf("%s configured", format)}
		}
	}

	// Check pre-commit for commit validation
	precommitPath := filepath.Join(repoPath, ".pre-commit-config.yaml")
	if content, err := os.ReadFile(precommitPath); err == nil {
		if strings.Contains(string(content), "commit") && strings.Contains(string(content), "lint") {
			return Check{Name: "commit message format", Passed: true, Detail: "commit linting configured"}
		}
	}

	return Check{Name: "commit message format", Passed: false, Detail: fmt.Sprintf("%s not enforced", format)}
}

// Check directory structure compliance
func checkDirectoryStructure(repoPath string, rules *ReconcileRules) Check {
	var requiredDirs []string
	var standardFiles []string

	if rules != nil {
		requiredDirs = rules.DirectoryStructure.RequiredDirs
		standardFiles = rules.DirectoryStructure.StandardFiles
	}

	// Default: check for src directory
	if len(requiredDirs) == 0 {
		srcDir := filepath.Join(repoPath, "src")
		if _, err := os.Stat(srcDir); err == nil {
			return Check{Name: "directory structure", Passed: true, Detail: "src/ directory found"}
		}
	}

	var missing []string
	for _, dir := range requiredDirs {
		p := filepath.Join(repoPath, dir)
		if _, err := os.Stat(p); err != nil {
			missing = append(missing, dir)
		}
	}

	for _, file := range standardFiles {
		p := filepath.Join(repoPath, file)
		if _, err := os.Stat(p); err != nil {
			missing = append(missing, file)
		}
	}

	if len(missing) > 0 {
		return Check{Name: "directory structure", Passed: false, Detail: fmt.Sprintf("missing: %s", strings.Join(missing, ", "))}
	}

	if len(requiredDirs) > 0 || len(standardFiles) > 0 {
		return Check{Name: "directory structure", Passed: true, Detail: "all required directories and files present"}
	}

	return Check{Name: "directory structure", Passed: true, Detail: "no specific structure requirements"}
}

// Check dependency management (lock files)
func checkDependencyManagement(repoPath string, rules *ReconcileRules) Check {
	requireLock := true
	if rules != nil {
		requireLock = rules.Dependencies.RequireLockFiles
	}

	if !requireLock {
		return Check{Name: "dependency lock files", Passed: true, Detail: "lock files not required"}
	}

	lockFiles := []string{
		"go.sum",           // Go
		"Cargo.lock",        // Rust
		"package-lock.json", // npm
		"yarn.lock",        // yarn
		"Pipfile.lock",     // pipenv
		"poetry.lock",      // poetry
		"requirements.lock", // pip
		"uv.lock",          // uv
	}

	var found []string
	for _, lockFile := range lockFiles {
		p := filepath.Join(repoPath, lockFile)
		if _, err := os.Stat(p); err == nil {
			found = append(found, lockFile)
		}
	}

	if len(found) > 0 {
		return Check{Name: "dependency lock files", Passed: true, Detail: fmt.Sprintf("found: %s", strings.Join(found, ", "))}
	}

	return Check{Name: "dependency lock files", Passed: false, Detail: "no lock files found (recommended for reproducibility)"}
}

// Check documentation requirements
func checkDocumentation(repoPath string, rules *ReconcileRules) Check {
	requireReadme := true
	if rules != nil {
		requireReadme = rules.Documentation.Readme.Required
	}

	if !requireReadme {
		return Check{Name: "documentation", Passed: true, Detail: "README not required"}
	}

	readmePaths := []string{
		filepath.Join(repoPath, "README.md"),
		filepath.Join(repoPath, "README.txt"),
		filepath.Join(repoPath, "README"),
	}

	for _, p := range readmePaths {
		if _, err := os.Stat(p); err == nil {
			// Check for minimum sections if rules provided
			if rules != nil && len(rules.Documentation.Readme.MinSections) > 0 {
				content, _ := os.ReadFile(p)
				var missing []string
				for _, section := range rules.Documentation.Readme.MinSections {
					if !strings.Contains(strings.ToLower(string(content)), strings.ToLower(section)) {
						missing = append(missing, section)
					}
				}
				if len(missing) > 0 {
					return Check{Name: "documentation", Passed: false, Detail: fmt.Sprintf("README missing sections: %s", strings.Join(missing, ", "))}
				}
			}
			return Check{Name: "documentation", Passed: true, Detail: "README found"}
		}
	}

	return Check{Name: "documentation", Passed: false, Detail: "README not found"}
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
