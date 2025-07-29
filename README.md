# GitHub Grid - Realistic Commit Pattern Generator

Generate realistic Git commit patterns for GitHub contribution graphs with natural developer activity patterns.

## Features

- **Realistic Patterns**: Sprints, vacations, spike days, and natural work rhythms
- **Multiple Presets**: Choose from realistic, steady, sporadic, or contractor patterns  
- **Interactive Preview**: ASCII calendar shows planned commits before execution
- **External Repos**: Work on any repository with `--repo` flag
- **Fast & Safe**: Rust implementation with proper error handling and batch operations

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [GitHub CLI](https://cli.github.com/) (for the `init` command)
- Git configured with your GitHub credentials

## Installation

```bash
git clone https://github.com/your-username/github-grid
cd github-grid
cargo build --release

# Ensure GitHub CLI is authenticated
gh auth login
```

## Usage

### Quick Start
```bash
# 1. Initialize a private GitHub repo for commit patterns
./target/release/github-grid init

# 2. Generate commits with realistic patterns
./target/release/github-grid --repo ~/github-grid-target --pattern realistic

# 3. Preview patterns before committing
./target/release/github-grid --repo ~/github-grid-target --dry-run
```

### Basic Usage
```bash
# Generate commits with realistic pattern (from last AutoGen commit to today)
./target/release/github-grid --repo ~/github-grid-target

# Preview before committing
./target/release/github-grid --repo ~/github-grid-target --dry-run

# Work on any external repository
./target/release/github-grid --repo ~/my-project
```

### Repository Setup
```bash
# Initialize with default settings (creates username-grid repo)
./target/release/github-grid init

# Initialize with custom name and location
./target/release/github-grid init --name my-commit-grid --local-dir ~/my-grid

# Force recreate existing repository
./target/release/github-grid init --force

# Check if GitHub CLI is set up
gh auth status
```

### Advanced Usage
```bash
# Generate commits for specific date range
./target/release/github-grid --repo ~/github-grid-target --start 2024-01-01 --end 2024-12-31

# Use different patterns
./target/release/github-grid --repo ~/github-grid-target --pattern contractor
./target/release/github-grid --repo ~/github-grid-target --pattern sporadic

# Preview different patterns
./target/release/github-grid preview --start 2024-01-01 --end 2024-01-07 --pattern realistic
```

### Available Patterns

- **realistic** - Natural developer activity with sprints, vacations, and spike days
- **steady** - Consistent daily activity with minor variations  
- **sporadic** - Irregular bursts of activity with quiet periods
- **contractor** - Monday-Friday focused with occasional weekend work

## Pattern Features

### Realistic Pattern
- 14-day sprint cycles with intensity bursts at start/end
- Random vacation periods (3-10 days) 
- Spike days with 10-30+ commits for big features
- 30% chance of weekend work (1-3 commits)
- Work hours: 9 AM - 7 PM

### Preview Example
```
=� Commit Calendar:

    Jan 01 �������
    Jan 08 �������

Legend: �=0 �=1-3 �=4-10 =%=10+ commits

Summary:
  Total commits: 36
  Active days: 6
  Avg commits/day: 6.0
  Weekend commits: 3 (8.3%)
```

## How It Works

1. **Setup**: `init` command creates a private GitHub repository and clones it locally
2. **Empty Commits**: Creates commits without files (like `git commit --allow-empty`) using git2 library
3. **Backdated Timestamps**: All commits use realistic historical timestamps for authentic patterns
4. **Batch Operations**: Pushes in batches of 50 commits for optimal performance
5. **Smart Continuation**: Automatically detects last `[AutoGen]` commit to seamlessly continue patterns
6. **Pattern Generation**: Uses sophisticated algorithms for realistic developer activity simulation

## Safety Features

- Always operates on `main` branch (switches automatically)
- Dry-run mode for safe previewing
- Proper error handling with detailed messages
- Batch operations with progress tracking
- Context-aware cancellation (Ctrl+C support)

## Recommended Workflow

**Important**: This tool should be run **separately** from your target repository:

```bash
#  Good: Run from the github-grid directory
cd ~/github-grid  
./target/release/github-grid --repo ~/my-actual-project

# L Bad: Don't run inside your actual project
cd ~/my-actual-project
~/github-grid/target/release/github-grid  # This works but clutters your project
```

The `--repo` flag lets you target any Git repository while keeping the tool isolated.