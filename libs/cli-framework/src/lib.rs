//! CLI argument parsing framework for AgilePlus
//!
//! Provides a trait-based CLI application framework with command registration.

pub mod error;

pub use error::{CliError, Result};

use std::collections::HashMap;
use std::fmt;

/// Argument definition for CLI commands.
#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub takes_value: bool,
}

impl Arg {
    /// Create a new argument with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            required: false,
            default_value: None,
            takes_value: true,
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Mark the argument as required.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set a default value.
    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.name)?;
        if self.required {
            write!(f, " (required)")?;
        }
        if let Some(ref default) = self.default_value {
            write!(f, " = {}", default)?;
        }
        Ok(())
    }
}

/// Command handler function type.
#[allow(clippy::non_send_fields_in_send_ty)]
pub type CommandHandler = Box<dyn Fn(&mut CommandContext) -> Result<()> + Send + Sync>;

/// Context for command execution.
#[derive(Debug, Default)]
pub struct CommandContext {
    args: HashMap<String, String>,
}

impl CommandContext {
    /// Create a new command context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get an argument value.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.args.get(name).map(|s| s.as_str())
    }

    /// Set an argument value.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.args.insert(name.into(), value.into());
    }

    /// Check if an argument is provided.
    pub fn contains(&self, name: &str) -> bool {
        self.args.contains_key(name)
    }
}

/// Command definition.
pub struct Command {
    pub name: String,
    pub description: String,
    pub args: Vec<Arg>,
    pub handler: Option<CommandHandler>,
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("args", &self.args)
            .finish()
    }
}

impl Command {
    /// Create a new command with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            args: Vec::new(),
            handler: None,
        }
    }

    /// Set the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add an argument.
    pub fn arg(mut self, arg: Arg) -> Self {
        self.args.push(arg);
        self
    }

    /// Set the handler.
    pub fn handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut CommandContext) -> Result<()> + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }
}

/// CLI application trait.
pub trait CliApp: Send + Sync {
    /// Get the application name.
    fn name(&self) -> &str;

    /// Get the application version.
    fn version(&self) -> &str;

    /// Get all registered commands.
    fn commands(&self) -> Vec<&Command>;

    /// Get a command by name.
    fn get_command(&self, name: &str) -> Option<&Command>;

    /// Execute a command by name with context.
    fn execute(&self, name: &str, ctx: &mut CommandContext) -> Result<()>;
}

/// CLI application builder.
#[derive(Debug, Default)]
pub struct CliAppBuilder {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    commands: Vec<Command>,
}

impl CliAppBuilder {
    /// Create a new CLI application builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the application name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the application version.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the application description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add a command.
    pub fn command(mut self, cmd: Command) -> Self {
        self.commands.push(cmd);
        self
    }

    /// Build the CLI application.
    pub fn build(self) -> CliAppImpl {
        CliAppImpl {
            name: self.name.unwrap_or_else(|| "app".to_string()),
            version: self.version.unwrap_or_else(|| "0.1.0".to_string()),
            description: self.description.unwrap_or_default(),
            commands: self.commands,
        }
    }
}

/// CLI application implementation.
#[derive(Debug)]
pub struct CliAppImpl {
    name: String,
    version: String,
    #[allow(dead_code)]
    description: String,
    commands: Vec<Command>,
}

impl CliApp for CliAppImpl {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn commands(&self) -> Vec<&Command> {
        self.commands.iter().collect()
    }

    fn get_command(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name == name)
    }

    fn execute(&self, name: &str, ctx: &mut CommandContext) -> Result<()> {
        let cmd = self
            .get_command(name)
            .ok_or_else(|| CliError::CommandNotFound(name.to_string()))?;

        let handler = cmd
            .handler
            .as_ref()
            .ok_or_else(|| CliError::ExecutionFailed("command has no handler".to_string()))?;

        handler(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_creation() {
        let arg = Arg::new("port")
            .description("The port number")
            .required(true)
            .default("8080");
        assert_eq!(arg.name, "port");
        assert!(arg.required);
        assert_eq!(arg.default_value, Some("8080".to_string()));
    }

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("serve")
            .description("Start the server")
            .arg(Arg::new("port").default("8080"))
            .handler(|_| Ok(()));
        assert_eq!(cmd.name, "serve");
        assert!(cmd.handler.is_some());
    }

    #[test]
    fn test_command_context() {
        let mut ctx = CommandContext::new();
        ctx.set("name", "Alice");
        ctx.set("age", "30");
        assert_eq!(ctx.get("name"), Some("Alice"));
        assert_eq!(ctx.get("age"), Some("30"));
        assert!(ctx.contains("name"));
        assert!(!ctx.contains("email"));
    }

    #[test]
    fn test_cli_app_builder() {
        let app = CliAppBuilder::new()
            .name("testapp")
            .version("1.0.0")
            .description("A test application")
            .command(Command::new("hello").handler(|_| Ok(())))
            .build();

        assert_eq!(app.name(), "testapp");
        assert_eq!(app.version(), "1.0.0");
        assert_eq!(app.commands().len(), 1);
    }

    #[test]
    fn test_command_not_found() {
        let app = CliAppBuilder::new().name("testapp").build();

        let mut ctx = CommandContext::new();
        let result = app.execute("nonexistent", &mut ctx);
        assert!(matches!(result, Err(CliError::CommandNotFound(_))));
    }
}
