# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

GitHub Grid is a Rust CLI application that generates realistic Git commit patterns for GitHub contribution graphs. It creates natural developer activity patterns with configurable intensity levels, weekly rhythms, and vacation periods using a deterministic, composable pattern system.

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

# Target-based generation (recommended)
./target/debug/github-grid --target-total 5000

# Dry run (defaults to ~/github/username-grid)  
./target/debug/github-grid --dry-run

# Run with specific pattern
./target/debug/github-grid --pattern active

# Run on specific repository
./target/debug/github-grid --repo /path/to/custom/repo
```

## Architecture

### Module Structure
- `src/main.rs` - CLI parsing with clap, orchestration, UI display
- `src/patterns.rs` - Pattern trait system and implementations
- `src/git_ops.rs` - Git operations using git2 library

### Key Components

1. **Pattern System** (`src/patterns.rs`) - **REFACTORED 2024**
   - `ConfigurablePattern` - Core composable pattern engine with deterministic RNG
   - `IntensityLevel` enum - Casual, Active, Maintainer, Hyperactive, Extreme levels
   - `PatternConfig` - Configures intensity, weekly rhythms, vacation frequency, spike probability
   - Date-seeded `ChaCha8Rng` for consistent results across runs
   - Shared weekly multipliers: Monday blues (0.7x), Tue-Thu peaks (1.1x), Friday wind-down (0.8x)
   - Activity-level patterns: casual (~300/yr), active (~2,500/yr), maintainer (~5,000/yr), hyperactive (~12,000/yr), extreme (~20,000+/yr)
   - Enhanced variance: 0-80 commits/day range, 30% chance of zero commits even on work days
   - Realistic weekend work: 5-50% chance depending on intensity level
   - Natural breaks: 2-4% daily vacation probability with 1-10 day durations
   - Legacy pattern wrappers for backward compatibility
   - Zero code duplication - all patterns use shared `ConfigurablePattern` core

2. **Git Operations** (`src/git_ops.rs`)
   - Uses `git2` crate for commit creation, shell command for push (better auth compatibility)
   - `GitOperations::create_commit()` - Creates commits with backdated timestamps
   - `GitOperations::push_commits()` - Uses simple git push command for authentication
   - `GitOperations::get_latest_autogen_commit()` - Finds last [AutoGen] commit for continuation
   - Automatically switches to main branch and validates repo state

3. **CLI Interface** (`src/main.rs`)
   - Built with `clap` derive macros for modern CLI parsing
   - Progress bars with `indicatif` for batch operations
   - ASCII calendar preview with commit density visualization
   - Subcommands: `patterns`, `preview`
   - Comprehensive error handling with `Result<T, E>`

### Key Features

- **Target-Based Generation**: `--target-total` counts existing commits and calibrates patterns
- **Configurable Patterns**: Intensity levels with realistic variance (streaks, spikes, breaks)
- **Deterministic RNG**: ChaCha8Rng seeded by date for consistent results

### Performance Notes

- 500 commits per push batch
- Native git2 for commit creation, shell commands for push  
- Single-pass generation with timestamp sorting

## Important Implementation Details

- **Default Repository**: `~/github/username-grid` (dynamically determined)
- **Commit Attribution**: Uses global git config for author name/email
- **Batch Operations**: 500 commits per push for optimal performance
- **Branch Management**: Always operates on `main` branch
- **Authentication**: Uses `gh` CLI credentials via shell git commands

## Dependencies

- `git2` - Git operations
- `clap` - CLI parsing
- `chrono` - Date/time handling  
- `rand` + `rand_chacha` - Deterministic randomization
- `indicatif` - Progress bars