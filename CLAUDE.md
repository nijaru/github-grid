# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GitHub Grid is a Rust CLI application that generates realistic Git commit patterns for GitHub contribution graphs. It creates natural developer activity patterns including sprints, vacations, spike days, and different work styles through a trait-based pattern system.

## Development Commands

### Build
```bash
cargo build          # Debug build
cargo build --release  # Optimized release build
```

### Format and Lint
```bash
cargo fmt            # Format code
cargo clippy         # Lint checks
```

### Test Application
```bash
# Show available patterns
./target/debug/github-grid patterns

# Preview pattern without committing  
./target/debug/github-grid preview --start 2024-01-01 --end 2024-01-07 --pattern realistic

# Dry run on current repo
./target/debug/github-grid --dry-run

# Run on external repository
./target/debug/github-grid --repo /path/to/target/repo
```

## Architecture

### Module Structure
- `src/main.rs` - CLI parsing with clap, orchestration, UI display
- `src/patterns.rs` - Pattern trait system and implementations
- `src/git_ops.rs` - Git operations using git2 library

### Key Components

1. **Pattern System** (`src/patterns.rs`)
   - `Pattern` trait for extensible commit generation strategies
   - `RealisticPattern` - Sprint cycles, vacations, spike days
   - `SteadyPattern` - Consistent daily activity
   - `SporadicPattern` - Irregular bursts with quiet periods  
   - `ContractorPattern` - Mon-Fri focused with occasional weekends
   - Each pattern generates `CommitInfo` structs with timestamps and messages

2. **Git Operations** (`src/git_ops.rs`)
   - Uses `git2` crate for native Git operations (no shell exec)
   - `GitOperations::create_commit()` - Creates commits with backdated timestamps
   - `GitOperations::push_commits()` - Batches pushes (50 commits per batch)
   - `GitOperations::get_latest_autogen_commit()` - Finds last [AutoGen] commit for continuation
   - Automatically switches to main branch and validates repo state

3. **CLI Interface** (`src/main.rs`)
   - Built with `clap` derive macros for modern CLI parsing
   - Progress bars with `indicatif` for batch operations
   - ASCII calendar preview with commit density visualization
   - Subcommands: `patterns`, `preview`
   - Comprehensive error handling with `Result<T, E>`

### Pattern Features

- **Realistic Pattern**: 14-day sprints, 3-10 day vacations, 5% spike day probability
- **Time Distribution**: Work hours (9-19), extended hours for sporadic pattern
- **Weekend Logic**: Pattern-specific weekend work probability (10-30%)
- **Commit Messages**: 20 realistic [AutoGen] prefixed messages with variety

### Performance Optimizations

- Batches commits (50 per push vs original 10)
- Native git2 operations (no shell overhead) 
- Single-pass commit generation with sorted timestamps
- Progress tracking with minimal UI updates

## Important Behaviors

- **Repository Isolation**: Designed to run separately with `--repo` flag targeting external repos
- **Continuation Logic**: Automatically continues from last [AutoGen] commit if no --start specified
- **Branch Management**: Always operates on `main` branch, switches automatically
- **Error Recovery**: Proper error propagation, no silent failures
- **Signal Handling**: Context-aware cancellation (Ctrl+C support)
- **Timestamp Precision**: Nanosecond-precise timestamps to avoid collisions

## Dependencies

Key external crates:
- `git2` - Native Git operations  
- `clap` - CLI argument parsing
- `chrono` - Date/time handling
- `rand` - Pattern randomization (v0.9 with Rust 2024 compatibility)
- `indicatif` - Progress bars
- `ratatui` - Future TUI enhancements