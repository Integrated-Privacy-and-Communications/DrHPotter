//! Fake shell environment

mod commands;
mod filesystem;

use std::path::PathBuf;

use commands::execute_command;
use filesystem::FakeFilesystem;

/// Fake shell that emulates a Linux shell environment
pub struct FakeShell {
    filesystem: FakeFilesystem,
    current_dir: PathBuf,
    env_vars: std::collections::HashMap<String, String>,
}

impl FakeShell {
    /// Create a new fake shell
    pub fn new() -> Self {
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("USER".to_string(), "root".to_string());
        env_vars.insert("HOME".to_string(), "/root".to_string());
        env_vars.insert("PATH".to_string(), "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string());
        env_vars.insert("SHELL".to_string(), "/bin/bash".to_string());

        Self {
            filesystem: FakeFilesystem::new(),
            current_dir: PathBuf::from("/root"),
            env_vars,
        }
    }

    /// Execute a command in the fake shell
    pub async fn execute(&mut self, command: &str) -> String {
        let command = command.trim();

        if command.is_empty() {
            return String::new();
        }

        // Parse command (handle basic shell syntax)
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        let cmd = parts[0];
        let args = &parts[1..];

        // Execute the command
        execute_command(
            cmd,
            args,
            &mut self.filesystem,
            &mut self.current_dir,
            &self.env_vars,
        ).await
    }

    /// Get current working directory
    pub fn pwd(&self) -> &PathBuf {
        &self.current_dir
    }
}

impl Default for FakeShell {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shell_creation() {
        let shell = FakeShell::new();
        assert_eq!(shell.pwd(), &PathBuf::from("/root"));
    }

    #[tokio::test]
    async fn test_pwd_command() {
        let mut shell = FakeShell::new();
        let output = shell.execute("pwd").await;
        assert_eq!(output.trim(), "/root");
    }

    #[tokio::test]
    async fn test_whoami_command() {
        let mut shell = FakeShell::new();
        let output = shell.execute("whoami").await;
        assert_eq!(output.trim(), "root");
    }
}
