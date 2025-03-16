use std::process::Command;
use std::path::PathBuf;
use std::fs;

use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use env_logger::Env;
use log::{info, warn, error};
use serde::{Deserialize, Serialize};
use names::Generator;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Commit and push changes
    Commit {
        /// Custom commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Files to add (default: all)
        #[arg(short, long)]
        files: Option<Vec<String>>,

        /// Use conventional commit format
        #[arg(short, long)]
        conventional: bool,
    },
    /// Branch operations
    Branch {
        #[command(subcommand)]
        cmd: BranchCommands,
    },
    /// Initialize configuration
    Init,
    /// Show status
    Status,
}

#[derive(Subcommand)]
enum BranchCommands {
    /// Create a new branch
    Create { name: String },
    /// Switch to a branch
    Switch { name: String },
    /// Delete a branch
    Delete { name: String },
}

#[derive(Serialize, Deserialize)]
struct Config {
    default_remote: String,
    commit_template: String,
    auto_pull: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_remote: String::from("origin"),
            commit_template: String::from("feat: {}"),
            auto_pull: true,
        }
    }
}

struct GitOps {
    config: Config,
    dry_run: bool,
}

impl GitOps {
    fn new(config: Config, dry_run: bool) -> Self {
        Self { config, dry_run }
    }

    fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .map_err(|e| anyhow!("Failed to get current branch: {}", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn check_git_repo(&self) -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn has_changes(&self) -> Result<bool> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| anyhow!("Failed to check git status: {}", e))?;

        Ok(!output.stdout.is_empty())
    }

    fn pull(&self) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would pull changes");
            return Ok(());
        }

        let output = Command::new("git")
            .args(["pull"])
            .output()
            .map_err(|e| anyhow!("Failed to pull changes: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Pull failed: {}", err_msg);
            return Err(anyhow!("Pull failed: {}", err_msg));
        }

        Ok(())
    }

    fn add_files(&self, files: &[String]) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would add files: {:?}", files);
            return Ok(());
        }

        let output = Command::new("git")
            .arg("add")
            .args(files)
            .output()
            .map_err(|e| anyhow!("Failed to add files: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Add failed: {}", err_msg);
            return Err(anyhow!("Add failed: {}", err_msg));
        }

        Ok(())
    }

    fn commit(&self, message: &str) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would commit with message: {}", message);
            return Ok(());
        }

        let output = Command::new("git")
            .args(["commit", "-m", message])
            .output()
            .map_err(|e| anyhow!("Failed to commit: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Commit failed: {}", err_msg);
            return Err(anyhow!("Commit failed: {}", err_msg));
        }

        Ok(())
    }

    fn push(&self, branch: &str) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would push to {}", branch);
            return Ok(());
        }

        let output = Command::new("git")
            .args(["push", &self.config.default_remote, branch])
            .output()
            .map_err(|e| anyhow!("Failed to push: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Push failed: {}", err_msg);
            return Err(anyhow!("Push failed: {}", err_msg));
        }

        Ok(())
    }

    fn create_branch(&self, name: &str) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would create branch: {}", name);
            return Ok(());
        }

        let output = Command::new("git")
            .args(["checkout", "-b", name])
            .output()
            .map_err(|e| anyhow!("Failed to create branch: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Branch creation failed: {}", err_msg);
            return Err(anyhow!("Branch creation failed: {}", err_msg));
        }

        Ok(())
    }

    fn switch_branch(&self, name: &str) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would switch to branch: {}", name);
            return Ok(());
        }

        let output = Command::new("git")
            .args(["checkout", name])
            .output()
            .map_err(|e| anyhow!("Failed to switch branch: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Branch switch failed: {}", err_msg);
            return Err(anyhow!("Branch switch failed: {}", err_msg));
        }

        Ok(())
    }

    fn delete_branch(&self, name: &str) -> Result<()> {
        if self.dry_run {
            info!("[DRY RUN] Would delete branch: {}", name);
            return Ok(());
        }

        let output = Command::new("git")
            .args(["branch", "-d", name])
            .output()
            .map_err(|e| anyhow!("Failed to delete branch: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            error!("Branch deletion failed: {}", err_msg);
            return Err(anyhow!("Branch deletion failed: {}", err_msg));
        }

        Ok(())
    }
}

fn load_config() -> Result<Config> {
    let config_path = PathBuf::from("git-automate.toml");
    if !config_path.exists() {
        return Ok(Config::default());
    }

    let config_str = fs::read_to_string(config_path)
        .map_err(|e| anyhow!("Failed to read config file: {}", e))?;
    let config = toml::from_str(&config_str)
        .map_err(|e| anyhow!("Failed to parse config file: {}", e))?;

    Ok(config)
}

fn generate_commit_message(template: &str, conventional: bool) -> String {
    let mut generator = Generator::default();
    let name = generator.next().unwrap();
    
    if conventional {
        format!("feat: {}", name)
    } else {
        name
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_env(Env::default().default_filter_or("info"))
            .init();
    }

    let config = load_config()?;
    let git_ops = GitOps::new(config, cli.dry_run);

    if !git_ops.check_git_repo() {
        error!("Not in a git repository");
        return Err(anyhow::anyhow!("Not in a git repository"));
    }

    match &cli.command {
        Commands::Commit { message, files, conventional } => {
            let files = files.clone().unwrap_or_else(|| vec![String::from(".")]);

            
            if git_ops.config.auto_pull {
                git_ops.pull()?;
            }

            git_ops.add_files(&files)?;

            if !git_ops.has_changes()? {
                warn!("No changes to commit");
                return Ok(());
            }

            let commit_msg = message.clone()
                .unwrap_or_else(|| generate_commit_message(&git_ops.config.commit_template, *conventional));
            
            git_ops.commit(&commit_msg)?;

            let current_branch = git_ops.get_current_branch()?;
            git_ops.push(&current_branch)?;

            info!("Successfully committed and pushed changes");
        }
        Commands::Branch { cmd } => {
            match cmd {
                BranchCommands::Create { name } => git_ops.create_branch(name)?,
                BranchCommands::Switch { name } => git_ops.switch_branch(name)?,
                BranchCommands::Delete { name } => git_ops.delete_branch(name)?,
            }
        }
        Commands::Init => {
            let config = Config::default();
            let toml = toml::to_string_pretty(&config)?;
            fs::write("git-automate.toml", toml)?;
            info!("Initialized configuration file");
        }
        Commands::Status => {
            let current_branch = git_ops.get_current_branch()?;
            let has_changes = git_ops.has_changes()?;
            
            println!("Current branch: {}", current_branch);
            println!("Has uncommitted changes: {}", has_changes);
        }
    }

    Ok(())
}

