package cmd

import (
	"fmt"
	"os"

	"github.com/KooshaPari/pheno-cli/internal/plugin"
	"github.com/spf13/cobra"
)

var pluginCmd = &cobra.Command{
	Use:   "plugin",
	Short: "Manage pheno plugins",
	Long:  `Manage pheno plugins for governance template generation.`,
}

var pluginListCmd = &cobra.Command{
	Use:   "list",
	Short: "List installed plugins",
	Run: func(cmd *cobra.Command, args []string) {
		plugins := plugin.Global().List()
		if len(plugins) == 0 {
			fmt.Println("No plugins installed")
			return
		}
		fmt.Println("Installed plugins:")
		for _, p := range plugins {
			m := p.Metadata()
			fmt.Printf("  %-20s %-6s %s\n", m.Name, m.Version, m.Description)
		}
	},
}

var pluginRunCmd = &cobra.Command{
	Use:   "run [plugin] [path]",
	Short: "Run a plugin",
	Args:  cobra.RangeArgs(1, 2),
	Run: func(cmd *cobra.Command, args []string) {
		name := args[0]
		p, ok := plugin.Global().Get(name)
		if !ok {
			fmt.Fprintf(os.Stderr, "Plugin not found: %s\n", name)
			os.Exit(1)
		}

		templates := p.Templates()
		fmt.Printf("Running plugin %s (%d templates)\n", name, len(templates))
		for _, t := range templates {
			fmt.Printf("  - %s\n", t.Path)
		}
	},
}

func init() {
	pluginCmd.AddCommand(pluginListCmd)
	pluginCmd.AddCommand(pluginRunCmd)
	rootCmd.AddCommand(pluginCmd)
}
