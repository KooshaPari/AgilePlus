package manifest

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/pelletier/go-toml/v2"
)

// RepoConfig represents the configuration for a single repository in the org manifest.
type RepoConfig struct {
	Name        string `toml:"name"`
	Language    string `toml:"language"`
	Registry    string `toml:"registry"`
	RiskProfile string `toml:"risk_profile"`
	Private     bool   `toml:"private"`
	Skip        bool   `toml:"skip"`
}

// OrgManifest holds the organization-wide repository manifest.
type OrgManifest struct {
	Repos []RepoConfig `toml:"repos"`
}

// LoadManifest reads and parses a repos.toml file at the given path.
func LoadManifest(path string) (*OrgManifest, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read manifest %s: %w", path, err)
	}

	var m OrgManifest
	if err := toml.Unmarshal(data, &m); err != nil {
		return nil, fmt.Errorf("failed to parse manifest %s: %w", path, err)
	}

	return &m, nil
}

// manifestIndicators maps manifest filenames to their likely language.
var manifestIndicators = map[string]string{
	"package.json":     "typescript",
	"Cargo.toml":       "rust",
	"go.mod":           "go",
	"pyproject.toml":   "python",
	"setup.py":         "python",
	"requirements.txt": "python",
}

// registryForLanguage returns the default registry for a language.
func registryForLanguage(lang string) string {
	switch lang {
	case "go":
		return "go_proxy"
	case "rust":
		return "crates.io"
	case "python":
		return "pypi"
	case "typescript":
		return "npm"
	default:
		return "unknown"
	}
}

// GenerateManifest auto-detects repositories by scanning reposDir for manifest files.
// It treats EVERY subdirectory as a potential repo.
func GenerateManifest(reposDir string) (*OrgManifest, error) {
	entries, err := os.ReadDir(reposDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read repos directory %s: %w", reposDir, err)
	}

	var repos []RepoConfig
	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		name := entry.Name()
		// Skip hidden dirs (except .github, .claude, .devcontainer)
		if strings.HasPrefix(name, ".") && name != ".github" && name != ".devcontainer" && name != ".claude" && name != ".vscode" {
			continue
		}
		repoPath := filepath.Join(reposDir, name)
		lang := detectLanguage(repoPath)
		if lang == "" {
			lang = "docs"
		}
		repos = append(repos, RepoConfig{
			Name:        name,
			Language:    lang,
			Registry:    registryForLanguage(lang),
			RiskProfile: "low",
		})
	}

	return &OrgManifest{Repos: repos}, nil
}

// detectLanguage returns the primary language of a repo by checking for manifest files, .git, or source files.
func detectLanguage(repoPath string) string {
	// Check .git first
	if _, err := os.Stat(filepath.Join(repoPath, ".git")); err == nil {
		// It's a git repo - find manifest or source
		if lang := detectFromManifest(repoPath); lang != "" {
			return lang
		}
		return detectFromSource(repoPath)
	}
	// Not a git repo - check manifest
	return detectFromManifest(repoPath)
}

// detectFromManifest checks for package manifest files.
func detectFromManifest(repoPath string) string {
	manifests := []string{
		"go.mod", "Cargo.toml", "package.json", "pyproject.toml",
		"setup.py", "requirements.txt", "setup.cfg", "Pipfile", "pyproject.toml",
		"*.cabal", "mix.exs", "stack.yaml", "pom.xml", "build.gradle", "build.sbt",
		"Makefile", "CMakeLists.txt", "Dockerfile", "Dockerfile.dev", "Dockerfile.prod",
		"docker-compose.yml", "docker-compose.yaml",
	}
	for _, m := range manifests {
		if strings.HasPrefix(m, "*") {
			continue
		}
		if _, err := os.Stat(filepath.Join(repoPath, m)); err == nil {
			if lang, ok := manifestIndicators[m]; ok {
				return lang
			}
		}
	}
	return ""
}

// detectFromSource detects language from source file extensions.
func detectFromSource(repoPath string) string {
	exts := []string{".go", ".rs", ".ts", ".tsx", ".js", ".jsx", ".py", ".java", ".kt", ".cs", ".cpp", ".c", ".h", ".hpp", ".rb", ".php", ".swift", ".scala", ".ex", ".exs", ".erl", ".hs", ".ml", ".clj", ".zig", ".lua", ".r", ".nims"}
	entries, err := os.ReadDir(repoPath)
	if err != nil {
		return ""
	}
	for _, entry := range entries {
		if entry.IsDir() {
			name := entry.Name()
			if name == "node_modules" || name == "target" || name == ".git" || name == "dist" || name == "build" || name == "__pycache__" || name == ".venv" {
				continue
			}
			if lang := detectFromSource(filepath.Join(repoPath, name)); lang != "" {
				return lang
			}
		} else {
			name := entry.Name()
			for _, ext := range exts {
				if strings.HasSuffix(name, ext) {
					return "code"
				}
			}
		}
	}
	return ""
}
