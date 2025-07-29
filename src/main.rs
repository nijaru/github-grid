use chrono::{Local, NaiveDate, Datelike};
use clap::{Parser, Subcommand};
use git2::{Repository, Signature};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::fs;
use std::env;

mod patterns;
mod git_ops;
mod github;
mod error;

use patterns::{Pattern, CommitInfo, RealisticPattern, SteadyPattern, SporadicPattern, ContractorPattern, CasualPattern, ActivePattern, MaintainerPattern, HyperactivePattern, ExtremePattern, PatternConfig, IntensityLevel, ConfigurablePattern};
use git_ops::*;
use github::GitHubClient;
use error::{GitHubGridError, Result};

#[derive(Parser)]
#[command(name = "github-grid")]
#[command(about = "Generate realistic Git commit patterns for GitHub contribution graphs")]
struct Cli {
    /// Target repository path
    #[arg(short, long)]
    repo: Option<PathBuf>,
    
    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    start: Option<String>,
    
    /// End date (YYYY-MM-DD)
    #[arg(long)]
    end: Option<String>,
    
    /// Target total commits for the year (overrides pattern)
    #[arg(long)]
    target_total: Option<u32>,
    
    /// Pattern to use
    #[arg(short, long, default_value = "realistic")]
    pattern: String,
    
    /// Show preview without committing
    #[arg(long)]
    dry_run: bool,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show available patterns
    Patterns,
    /// Preview commits for date range
    Preview {
        #[arg(long)]
        start: String,
        #[arg(long)]
        end: String,
        #[arg(short, long, default_value = "realistic")]
        pattern: String,
    },
    /// Initialize or reset a private GitHub repo for commit patterns
    Init {
        /// Repository name (defaults to username-grid)
        #[arg(short, long)]
        name: Option<String>,
        /// Force recreate if repo exists
        #[arg(long)]
        force: bool,
        /// Local directory to clone to (defaults to ~/github/repo-name)
        #[arg(long)]
        local_dir: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Patterns) => {
            show_patterns();
            return Ok(());
        }
        Some(Commands::Preview { start, end, pattern }) => {
            let start_date = NaiveDate::parse_from_str(&start, "%Y-%m-%d")?;
            let end_date = NaiveDate::parse_from_str(&end, "%Y-%m-%d")?;
            preview_pattern(&pattern, start_date, end_date)?;
            return Ok(());
        }
        Some(Commands::Init { name, force, local_dir }) => {
            init_github_repo(name, force, local_dir)?;
            return Ok(());
        }
        None => {}
    }
    
    // Use default path if not specified
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let repo_path = match cli.repo {
        Some(path) => path,
        None => {
            // Get username dynamically for default path
            let github = GitHubClient::new()?;
            let username = github.username();
            PathBuf::from(format!("{}/github/{}-grid", home_dir, username))
        }
    };
    
    let repo = Repository::open(&repo_path)?;
    let mut git_ops = GitOperations::new(repo);
    
    let (start_date, end_date) = determine_date_range(&mut git_ops, cli.start, cli.end)?;
    
    println!("Generating commits from {} to {}", start_date, end_date);
    
    let (_pattern_name, commits) = if let Some(target_total) = cli.target_total {
        // Target-based generation
        let current_year = start_date.year();
        let existing_commits = count_existing_commits(&git_ops, current_year)?;
        let commits_needed = target_total.saturating_sub(existing_commits);
        let days_in_range = (end_date - start_date).num_days() + 1;
        
        println!("🎯 Target: {} commits total for {}", target_total, current_year);
        println!("📊 Existing: {} commits", existing_commits);
        println!("➕ Generating: ~{} commits over {} days", commits_needed, days_in_range);
        
        if commits_needed == 0 {
            println!("✅ Target already reached!");
            return Ok(());
        }
        
        let config = calibrate_pattern_for_target(commits_needed, days_in_range);
        let pattern_impl = ConfigurablePattern::new(config);
        let commits = pattern_impl.generate(start_date, end_date);
        
        (format!("target-{}", target_total), commits)
    } else {
        // Traditional pattern-based generation
        println!("Pattern: {}", cli.pattern);
        let pattern = create_pattern(&cli.pattern)?;
        let commits = pattern.generate(start_date, end_date);
        (cli.pattern.clone(), commits)
    };
    
    println!("Generated {} commits", commits.len());
    
    if cli.dry_run {
        show_commit_summary(&commits);
        return Ok(());
    }
    
    execute_commits(&mut git_ops, commits)?;
    
    Ok(())
}

fn show_patterns() {
    println!("Available patterns:");
    println!("\nActivity levels (commits/year):");
    println!("  casual      - Weekend warrior, occasional PRs (~300/year)");
    println!("  realistic   - Professional developer activity (~1,200/year)");
    println!("  active      - Multiple projects, good practices (~2,500/year)");
    println!("  maintainer  - Managing repos, reviewing PRs (~5,000/year)");
    println!("  hyperactive - Startup pace, heavy open source (~12,000/year)");
    println!("  extreme     - Your level: 50-100+ commits/day (~20,000+/year)");
    println!("\nLegacy patterns:");
    println!("  steady      - Consistent daily activity");
    println!("  sporadic    - Irregular bursts of activity");
    println!("  contractor  - Mon-Fri focused with occasional weekends");
}

fn preview_pattern(pattern_name: &str, start: NaiveDate, end: NaiveDate) -> Result<()> {
    let pattern = create_pattern(pattern_name)?;
    let commits = pattern.generate(start, end);
    
    show_commit_calendar(&commits, start, end);
    show_commit_summary(&commits);
    
    Ok(())
}

fn show_commit_calendar(commits: &[CommitInfo], start: NaiveDate, end: NaiveDate) {
    println!("\n📅 Commit Calendar:");
    
    let mut current = start;
    while current <= end {
        let count = commits.iter()
            .filter(|c| c.date.date_naive() == current)
            .count();
            
        let symbol = match count {
            0 => "░",
            1..=3 => "▓",
            4..=10 => "█",
            _ => "🔥",
        };
        
        if current.weekday().number_from_monday() == 1 {
            println!();
            print!("{:>10} ", current.format("%b %d"));
        }
        
        print!("{}", symbol);
        current = current.succ_opt().unwrap();
    }
    println!("\n\nLegend: ░=0 ▓=1-3 █=4-10 🔥=10+ commits\n");
}

fn show_commit_summary(commits: &[CommitInfo]) {
    let total = commits.len();
    let avg_per_day = if total > 0 {
        commits.iter()
            .map(|c| c.date.date_naive())
            .collect::<std::collections::HashSet<_>>()
            .len()
    } else {
        0
    };
    
    println!("Summary:");
    println!("  Total commits: {}", total);
    println!("  Active days: {}", avg_per_day);
    if avg_per_day > 0 {
        println!("  Avg commits/day: {:.1}", total as f64 / avg_per_day as f64);
    }
    
    let weekend_commits = commits.iter()
        .filter(|c| {
            let weekday = c.date.weekday();
            weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
        })
        .count();
    
    println!("  Weekend commits: {} ({:.1}%)", weekend_commits, 
             weekend_commits as f64 / total as f64 * 100.0);
}

fn determine_date_range(
    git_ops: &mut GitOperations,
    start: Option<String>,
    end: Option<String>,
) -> Result<(NaiveDate, NaiveDate)> {
    let end_date = match end {
        Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?,
        None => Local::now().date_naive(),
    };
    
    let start_date = match start {
        Some(date_str) => NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")?,
        None => {
            match git_ops.get_latest_autogen_commit()? {
                Some(last_commit) => last_commit.date_naive() + chrono::Duration::days(1),
                None => end_date - chrono::Duration::days(365),
            }
        }
    };
    
    Ok((start_date, end_date))
}

fn create_pattern(name: &str) -> Result<Box<dyn Pattern>> {
    match name {
        // Legacy patterns
        "realistic" => Ok(Box::new(RealisticPattern::new())),
        "steady" => Ok(Box::new(SteadyPattern::new())),
        "sporadic" => Ok(Box::new(SporadicPattern::new())),
        "contractor" => Ok(Box::new(ContractorPattern::new())),
        // Activity-level patterns
        "casual" => Ok(Box::new(CasualPattern::new())),
        "active" => Ok(Box::new(ActivePattern::new())),
        "maintainer" => Ok(Box::new(MaintainerPattern::new())),
        "hyperactive" => Ok(Box::new(HyperactivePattern::new())),
        "extreme" => Ok(Box::new(ExtremePattern::new())),
        _ => Err(GitHubGridError::Config(format!("Unknown pattern: {}", name))),
    }
}

fn execute_commits(
    git_ops: &mut GitOperations,
    commits: Vec<CommitInfo>,
) -> Result<()> {
    let pb = ProgressBar::new(commits.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap(),
    );
    
    let mut batch_count = 0;
    const BATCH_SIZE: usize = 500;
    
    for commit in commits {
        pb.set_message(format!("Committing {}", commit.date.format("%Y-%m-%d %H:%M")));
        
        git_ops.create_commit(&commit)?;
        
        batch_count += 1;
        if batch_count >= BATCH_SIZE {
            pb.set_message("Pushing batch...".to_string());
            git_ops.push_commits()?;
            batch_count = 0;
        }
        
        pb.inc(1);
    }
    
    if batch_count > 0 {
        pb.set_message("Final push...".to_string());
        git_ops.push_commits()?;
    }
    
    pb.finish_with_message("✅ All commits created successfully!");
    Ok(())
}

fn init_github_repo(
    name: Option<String>,
    force: bool,
    local_dir: Option<String>,
) -> Result<()> {
    println!("🚀 Initializing GitHub repository for commit patterns...");
    
    // Create GitHub client
    let github = GitHubClient::new()?;
    let username = github.username();
    println!("📋 GitHub username: {}", username);
    
    // Determine repo name
    let repo_name = name.unwrap_or_else(|| format!("{}-grid", username));
    println!("📂 Repository name: {}", repo_name);
    
    // Determine local directory (default: ~/github/repo-name)
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let local_path = local_dir.unwrap_or_else(|| format!("{}/github/{}", home_dir, repo_name));
    println!("💾 Local directory: {}", local_path);
    
    // Check if repo exists on GitHub
    let repo_exists = github.repo_exists(&repo_name)?;
    
    if repo_exists {
        if force {
            println!("⚠️  Repository exists, deleting due to --force flag...");
            // Remove local directory first to avoid clone conflicts
            if PathBuf::from(&local_path).exists() {
                fs::remove_dir_all(&local_path)?;
                println!("🗑️  Removed local directory");
            }
            github.delete_repo(&repo_name)?;
        } else {
            println!("✅ Repository already exists: https://github.com/{}/{}", username, repo_name);
            println!("💡 Use --force to recreate or update the existing repo");
            
            // Check if local clone exists
            if PathBuf::from(&local_path).exists() {
                println!("📁 Local clone already exists at: {}", local_path);
                println!("🎯 Ready to use!");
                return Ok(());
            } else {
                println!("📥 Cloning existing repository...");
                github.clone_repo(&repo_name, &local_path)?;
                
                // Check if repo needs initialization (empty repo)
                let repo = Repository::open(&local_path)?;
                if repo.is_empty()? {
                    println!("🔧 Repository is empty, initializing...");
                    initialize_repo(&repo, &local_path)?;
                }
                
                println!("🎯 Ready to use!");
                return Ok(());
            }
        }
    }
    
    // Create new private repository
    println!("🏗️  Creating private repository...");
    github.create_repo(&repo_name)?;
    
    // Clone the repository locally
    println!("📥 Cloning repository...");
    github.clone_repo(&repo_name, &local_path)?;
    let repo = Repository::open(&local_path)?;
    
    // Initialize with empty commit
    initialize_repo(&repo, &local_path)?;
    
    println!("✅ Repository setup complete!");
    println!("🌐 GitHub: https://github.com/{}/{}", username, repo_name);
    println!("📁 Local: {}", local_path);
    println!();
    println!("🎯 Usage:");
    println!("  ./target/release/github-grid --target-total 5000");
    println!("  ./target/release/github-grid --pattern active");
    println!("  ./target/release/github-grid --dry-run");
    
    Ok(())
}

fn initialize_repo(repo: &Repository, local_path: &str) -> Result<()> {
    let repo_path = PathBuf::from(local_path);
    
    // Create initial README
    let readme_content = "# GitHub Contribution Grid\n\nThis repository contains generated commit patterns for GitHub contribution graphs.\n";
    fs::write(repo_path.join("README.md"), readme_content)?;
    
    // Stage the README
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("README.md"))?;
    index.write()?;
    
    // Create tree
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    
    // Create signature
    let sig = Signature::now("GitHub Grid", "github-grid@example.com")?;
    
    // Create initial commit
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Initial commit: Setup repository for grid patterns",
        &tree,
        &[],
    )?;
    
    // Push to GitHub using our git operations
    let mut git_ops = GitOperations::new(Repository::open(local_path)?);
    git_ops.push_commits()?;
    
    Ok(())
}

fn count_existing_commits(git_ops: &GitOperations, year: i32) -> Result<u32> {
    let repo_path = git_ops.repo().workdir().unwrap();
    let output = std::process::Command::new("git")
        .current_dir(repo_path)
        .args(&[
            "log",
            "--oneline",
            &format!("--since={}-01-01", year),
            &format!("--until={}-12-31", year),
        ])
        .output()
        .map_err(|e| GitHubGridError::Io(e))?;
    
    if !output.status.success() {
        return Ok(0); // Empty repo or no commits in range
    }
    
    let commit_lines = String::from_utf8_lossy(&output.stdout);
    let count = commit_lines.lines().count() as u32;
    
    Ok(count)
}

fn calibrate_pattern_for_target(commits_needed: u32, days_in_range: i64) -> PatternConfig {
    let avg_per_day = commits_needed as f64 / days_in_range as f64;
    
    // Choose intensity level based on required daily average
    // Calibrated based on testing: 0.9x gets us close to target with variance
    let target_avg = avg_per_day * 0.9;  // Balanced: accounts for spikes while hitting target
    
    let intensity = if target_avg < 5.0 {
        IntensityLevel::Casual
    } else if target_avg < 15.0 {
        IntensityLevel::Active  
    } else if target_avg < 30.0 {
        IntensityLevel::Maintainer
    } else if target_avg < 50.0 {
        IntensityLevel::Hyperactive
    } else {
        IntensityLevel::Extreme
    };
    
    // Create pattern config with enhanced variance for target hitting
    let vacation_freq = match intensity {
        IntensityLevel::Casual => 0.03,
        IntensityLevel::Active => 0.02,
        IntensityLevel::Maintainer => 0.015,
        IntensityLevel::Hyperactive => 0.01,
        IntensityLevel::Extreme => 0.008,
    };
    
    // Enhanced spike probability for more realistic patterns
    let spike_prob = match intensity {
        IntensityLevel::Casual => 0.18,
        IntensityLevel::Active => 0.22,
        IntensityLevel::Maintainer => 0.28,
        IntensityLevel::Hyperactive => 0.32,
        IntensityLevel::Extreme => 0.38,
    };
    
    PatternConfig {
        intensity,
        use_weekly_rhythm: true,
        vacation_frequency: vacation_freq,
        vacation_duration: (1, 4),
        spike_probability: spike_prob,
        spike_multiplier: 2.8,  // Higher spikes for more realistic deadline/feature patterns
    }
}
