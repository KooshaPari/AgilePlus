package plugin

import (
	"context"
	"sync"
)

type Metadata struct {
	Name        string
	Version     string
	Description string
	Author      string
	Tags        []string
}

type TemplateFile struct {
	Path    string
	Content string
}

type Plugin interface {
	Metadata() Metadata
	Init(ctx context.Context, cfg map[string]interface{}) error
	Templates() []TemplateFile
	Execute(ctx context.Context, args []string) error
}

type Registry struct {
	mu      sync.RWMutex
	plugins map[string]Plugin
}

var globalRegistry = NewRegistry()

func Global() *Registry { return globalRegistry }

func NewRegistry() *Registry {
	return &Registry{plugins: make(map[string]Plugin)}
}

func (r *Registry) Register(p Plugin) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	m := p.Metadata()
	if _, ok := r.plugins[m.Name]; ok {
		return nil
	}
	r.plugins[m.Name] = p
	return nil
}

func (r *Registry) List() []Plugin {
	r.mu.RLock()
	defer r.mu.RUnlock()
	out := make([]Plugin, 0, len(r.plugins))
	for _, p := range r.plugins {
		out = append(out, p)
	}
	return out
}

func (r *Registry) Get(name string) (Plugin, bool) {
	r.mu.RLock()
	defer r.mu.RUnlock()
	p, ok := r.plugins[name]
	return p, ok
}

func init() {
	globalRegistry.Register(&SAST{})
	globalRegistry.Register(&Dependabot{})
	globalRegistry.Register(&Coverage{})
	globalRegistry.Register(&Codeowners{})
	globalRegistry.Register(&IssueTemplates{})
	globalRegistry.Register(&PRTemplates{})
	globalRegistry.Register(&QualityGate{})
	globalRegistry.Register(&PreCommit{})
	globalRegistry.Register(&EditorConfig{})
	globalRegistry.Register(&SecurityPolicy{})
	globalRegistry.Register(&Trivy{})
	globalRegistry.Register(&LicenseCompliance{})
}

type SAST struct{}

func (p *SAST) Metadata() Metadata {
	return Metadata{Name: "sast", Version: "1.0.0", Description: "Static Application Security Testing", Author: "Phenotype Org"}
}
func (p *SAST) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *SAST) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/workflows/sast.yml", Content: sastYML}}
}
func (p *SAST) Execute(_ context.Context, _ []string) error { return nil }

const sastYML = `name: SAST
on:
  pull_request:
  push:
    branches: [main]
jobs:
  semgrep:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: returntocorp/semgrep-action@v1
`

type Dependabot struct{}

func (p *Dependabot) Metadata() Metadata {
	return Metadata{Name: "dependabot", Version: "1.0.0", Description: "Automated dependency updates", Author: "Phenotype Org"}
}
func (p *Dependabot) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *Dependabot) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/dependabot.yml", Content: dependabotYML}}
}
func (p *Dependabot) Execute(_ context.Context, _ []string) error { return nil }

const dependabotYML = `version: 2
updates:
  - package-ecosystem: gomod
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
`

type Coverage struct{}

func (p *Coverage) Metadata() Metadata {
	return Metadata{Name: "coverage", Version: "1.0.0", Description: "Code coverage tracking", Author: "Phenotype Org"}
}
func (p *Coverage) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *Coverage) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/workflows/coverage.yml", Content: coverageYML}}
}
func (p *Coverage) Execute(_ context.Context, _ []string) error { return nil }

const coverageYML = `name: Coverage
on:
  push:
    branches: [main]
  pull_request:
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: go test ./... -coverprofile=coverage.out
      - uses: codecov/codecov-action@v4
`

type Codeowners struct{}

func (p *Codeowners) Metadata() Metadata {
	return Metadata{Name: "codeowners", Version: "1.0.0", Description: "GitHub CODEOWNERS", Author: "Phenotype Org"}
}
func (p *Codeowners) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *Codeowners) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/CODEOWNERS", Content: codeownersContent}}
}
func (p *Codeowners) Execute(_ context.Context, _ []string) error { return nil }

const codeownersContent = "* @phenotype/owners\n"

type IssueTemplates struct{}

func (p *IssueTemplates) Metadata() Metadata {
	return Metadata{Name: "issue-templates", Version: "1.0.0", Description: "GitHub issue templates", Author: "Phenotype Org"}
}
func (p *IssueTemplates) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *IssueTemplates) Templates() []TemplateFile {
	return []TemplateFile{
		{Path: ".github/ISSUE_TEMPLATE/bug_report.yml", Content: bugYML},
		{Path: ".github/ISSUE_TEMPLATE/feature_request.yml", Content: featureYML},
	}
}
func (p *IssueTemplates) Execute(_ context.Context, _ []string) error { return nil }

const bugYML = `name: Bug Report
description: Create a report to help us improve
labels: [bug]
body:
  - type: markdown
    attributes:
      value: |
        ## Bug Description
        [Describe the bug]
`

const featureYML = `name: Feature Request
description: Suggest an idea for this project
labels: [enhancement]
body:
  - type: markdown
    attributes:
      value: |
        ## Feature Description
        [Describe the feature]
`

type PRTemplates struct{}

func (p *PRTemplates) Metadata() Metadata {
	return Metadata{Name: "pr-templates", Version: "1.0.0", Description: "Pull request templates", Author: "Phenotype Org"}
}
func (p *PRTemplates) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *PRTemplates) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/PULL_REQUEST_TEMPLATE.md", Content: prYML}}
}
func (p *PRTemplates) Execute(_ context.Context, _ []string) error { return nil }

const prYML = `## Description
[Describe your changes]

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated
`

type QualityGate struct{}

func (p *QualityGate) Metadata() Metadata {
	return Metadata{Name: "quality-gate", Version: "1.0.0", Description: "Quality gate workflow", Author: "Phenotype Org"}
}
func (p *QualityGate) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *QualityGate) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/workflows/quality-gate.yml", Content: qualityYML}}
}
func (p *QualityGate) Execute(_ context.Context, _ []string) error { return nil }

const qualityYML = `name: Quality Gate
on:
  pull_request:
  push:
    branches: [main]
jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run quality checks
        run: echo Running quality checks...
`

type PreCommit struct{}

func (p *PreCommit) Metadata() Metadata {
	return Metadata{Name: "pre-commit", Version: "1.0.0", Description: "Pre-commit hooks", Author: "Phenotype Org"}
}
func (p *PreCommit) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *PreCommit) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".pre-commit-config.yaml", Content: precommitYML}}
}
func (p *PreCommit) Execute(_ context.Context, _ []string) error { return nil }

const precommitYML = `repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
`

type EditorConfig struct{}

func (p *EditorConfig) Metadata() Metadata {
	return Metadata{Name: "editorconfig", Version: "1.0.0", Description: "Editor configuration", Author: "Phenotype Org"}
}
func (p *EditorConfig) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *EditorConfig) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".editorconfig", Content: editorconfigYML}}
}
func (p *EditorConfig) Execute(_ context.Context, _ []string) error { return nil }

const editorconfigYML = `root = true

[*]
indent_style = space
indent_size = 2
end_of_line = lf
charset = utf-8
trim_trailing_whitespace = true
insert_final_newline = true

[*.go]
indent_size = 4

[*.rs]
indent_size = 4
`

type SecurityPolicy struct{}

func (p *SecurityPolicy) Metadata() Metadata {
	return Metadata{Name: "security-policy", Version: "1.0.0", Description: "Security policy", Author: "Phenotype Org"}
}
func (p *SecurityPolicy) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *SecurityPolicy) Templates() []TemplateFile {
	return []TemplateFile{{Path: "SECURITY.md", Content: securityYML}}
}
func (p *SecurityPolicy) Execute(_ context.Context, _ []string) error { return nil }

const securityYML = `# Security Policy

## Reporting Vulnerabilities
Email security@phenotype.dev

## Supported Versions
| Version | Supported |
| --- | --- |
| 1.x | yes |
`

type Trivy struct{}

func (p *Trivy) Metadata() Metadata {
	return Metadata{Name: "trivy", Version: "1.0.0", Description: "Container scanning", Author: "Phenotype Org"}
}
func (p *Trivy) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *Trivy) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/workflows/trivy.yml", Content: trivyYML}}
}
func (p *Trivy) Execute(_ context.Context, _ []string) error { return nil }

const trivyYML = `name: Trivy
on:
  schedule:
    cron: 0 6 * * *
  push:
    branches: [main]
jobs:
  trivy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: aquasecurity/trivy-action@master
`

type LicenseCompliance struct{}

func (p *LicenseCompliance) Metadata() Metadata {
	return Metadata{Name: "license-compliance", Version: "1.0.0", Description: "License compliance", Author: "Phenotype Org"}
}
func (p *LicenseCompliance) Init(_ context.Context, _ map[string]interface{}) error { return nil }
func (p *LicenseCompliance) Templates() []TemplateFile {
	return []TemplateFile{{Path: ".github/workflows/license.yml", Content: licenseYML}}
}
func (p *LicenseCompliance) Execute(_ context.Context, _ []string) error { return nil }

const licenseYML = `name: License Check
on: [pull_request]
jobs:
  license:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check licenses
        run: echo License check placeholder
`
