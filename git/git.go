package git

import (
	"context"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"strings"
	"time"

	"math/rand"
	"sort"
	"errors"

	"github.com/nijaru/github-grid/dateutil"
)

// Constants related to Git operations
const (
	maxRetries      = 3
	filename        = "edit.txt"
	dateFormat      = "2006-01-02 15:04:05 -0700"
	shortDateFormat = "2006-01-02"
)

// CommitMessage represents a Git commit message with its weight for random selection
type CommitMessage struct {
	Message string
	Weight  int
}

var commitMessages = []CommitMessage{
	{"[AutoGen] Add a new feature", 10},
	{"[AutoGen] Fix a bug", 8},
	{"[AutoGen] Refactor some code", 6},
	{"[AutoGen] Add a new test", 5},
	{"[AutoGen] Update the requirements", 4},
	{"[AutoGen] Update the documentation", 3},
	{"[AutoGen] Update the README", 3},
	{"[AutoGen] Update the license", 2},
	{"[AutoGen] Update the gitignore", 1},
	{"[AutoGen] Update the CI/CD pipeline", 1},
	{"[AutoGen] Update the Dockerfile", 1},
	{"[AutoGen] Update the Makefile", 1},
	{"[AutoGen] Update the GitHub Actions", 1},
	{"[AutoGen] Update the Jenkinsfile", 1},
	{"[AutoGen] Update the AWS config", 1},
	{"[AutoGen] Update the GCP config", 1},
	{"[AutoGen] Update the Azure config", 1},
}

// WeightedRandomSelector handles weighted random selections
type WeightedRandomSelector struct {
	messages          []CommitMessage
	cumulativeWeights []int
	totalWeight       int
}

// NewWeightedRandomSelector initializes a new WeightedRandomSelector
func NewWeightedRandomSelector(messages []CommitMessage) *WeightedRandomSelector {
	cumulative := make([]int, len(messages))
	currentSum := 0
	for i, cm := range messages {
		currentSum += cm.Weight
		cumulative[i] = currentSum
	}
	return &WeightedRandomSelector{
		messages:          messages,
		cumulativeWeights: cumulative,
		totalWeight:       currentSum,
	}
}

// SelectRandom selects a random message based on weights
func (w *WeightedRandomSelector) SelectRandom() string {
	if w.totalWeight == 0 {
		return "Default commit message"
	}
	r := rand.Intn(w.totalWeight) + 1
	index := sort.Search(len(w.cumulativeWeights), func(i int) bool {
		return w.cumulativeWeights[i] >= r
	})
	if index < len(w.messages) {
		return w.messages[index].Message
	}
	return "Default commit message"
}

// GitOperations encapsulates Git-related functionalities
type GitOperations struct {
	logger   *slog.Logger
	selector *WeightedRandomSelector
}

// NewGitOperations creates a new instance of GitOperations
func NewGitOperations(logger *slog.Logger) *GitOperations {
	selector := NewWeightedRandomSelector(commitMessages)
	return &GitOperations{
		logger:   logger,
		selector: selector,
	}
}

// RunCommand executes a command with context and logs its output
func (g *GitOperations) RunCommand(
	ctx context.Context,
	command string,
	args ...string,
) (string, error) {
	if ctx.Err() != nil {
		return "", fmt.Errorf("operation cancelled: %w", ctx.Err())
	}

	g.logger.Info("Running command", "command", command, "args", strings.Join(args, " "))
	cmd := exec.CommandContext(ctx, command, args...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		g.logger.Error("Command failed",
			"command", command,
			"args", args,
			"error", err,
			"output", string(output))
		return "", fmt.Errorf("command failed: %v, output: %s", err, string(output))
	}
	return strings.TrimSpace(string(output)), nil
}

// RetryOperation retries a given operation up to maxRetries times
func (g *GitOperations) RetryOperation(description string, operation func() error) error {
	for i := 0; i < maxRetries; i++ {
		if err := operation(); err != nil {
			g.logger.Warn("Operation failed, retrying",
				"description", description,
				"attempt", i+1,
				"error", err)
			if i < maxRetries-1 {
				time.Sleep(time.Second * time.Duration(i+1))
				continue
			}
			return fmt.Errorf(
				"operation %s failed after %d attempts: %w",
				description,
				maxRetries,
				err,
			)
		}
		return nil
	}
	return fmt.Errorf("operation %s failed after %d attempts", description, maxRetries)
}

// Helper function for standardized error messages
func wrapError(context string, err error) error {
	return fmt.Errorf("%s: %w", context, err)
}

// EnsureGitRepository checks if the current directory is a Git repository
func (g *GitOperations) EnsureGitRepository(ctx context.Context) error {
	_, err := g.RunCommand(ctx, "git", "rev-parse", "--is-inside-work-tree")
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return wrapError("ensure git repository: git not installed", err)
		}
		return wrapError("ensure git repository: not a git repository", err)
	}
	return nil
}

// EnsureMainBranch ensures the current Git branch is 'main'
func (g *GitOperations) EnsureMainBranch(ctx context.Context) error {
	currentBranch, err := g.RunCommand(ctx, "git", "rev-parse", "--abbrev-ref", "HEAD")
	if err != nil {
		return wrapError("ensure main branch: failed to get current branch", err)
	}

	if currentBranch != "main" {
		g.logger.Info("Switching to main branch", "from", currentBranch)
		err = g.RetryOperation("switch to main branch", func() error {
			_, err := g.RunCommand(ctx, "git", "switch", "main")
			return err
		})
		if err != nil {
			return wrapError("ensure main branch: failed to switch to main branch", err)
		}
	}
	return nil
}

// GetLatestAutoGeneratedCommitDate retrieves the latest commit date with "[AutoGen]" prefix
func (g *GitOperations) GetLatestAutoGeneratedCommitDate(ctx context.Context) (time.Time, error) {
	// Git command to get the latest commit date with the "[AutoGen]" prefix
	command := "git"
	args := []string{"log", "--grep=^\\[AutoGen\\]", "-n", "1", "--format=%ci"}

	output, err := g.RunCommand(ctx, command, args...)
	if err != nil {
		return time.Time{}, fmt.Errorf("failed to retrieve latest auto-generated commit: %w", err)
	}

	// Check if output is empty
	if strings.TrimSpace(output) == "" {
		return time.Time{}, fmt.Errorf("no auto-generated commits found")
	}

	// Parse the commit date
	commitDate, err := time.Parse(dateFormat, output)
	if err != nil {
		return time.Time{}, fmt.Errorf("failed to parse commit date: %w", err)
	}

	return commitDate, nil
}

// CommitChanges stages and commits the changes with a specified date
func (g *GitOperations) CommitChanges(ctx context.Context, commitTime time.Time) error {
	if ctx.Err() != nil {
		return fmt.Errorf("operation cancelled: %w", ctx.Err())
	}

	formattedDate := commitTime.Format(dateFormat)
	commitMsg := g.GetRandomMessage()

	return g.RetryOperation("commit changes", func() error {
		if _, err := g.RunCommand(ctx, "git", "add", filename); err != nil {
			return fmt.Errorf("git add failed: %w", err)
		}

		// Execute commit with GIT_COMMITTER_DATE set
		cmd := exec.CommandContext(ctx, "git", "commit", "--date", formattedDate, "-m", commitMsg)
		cmd.Env = append(os.Environ(), fmt.Sprintf("GIT_COMMITTER_DATE=%s", formattedDate))
		output, err := cmd.CombinedOutput()
		if err != nil {
			g.logger.Error("Git commit failed", "error", err, "output", string(output))
			return fmt.Errorf("git commit failed: %v, output: %s", err, string(output))
		}

		g.logger.Info("Successfully committed changes",
			"date", formattedDate,
			"message", commitMsg)
		return nil
	})
}

// PushCommits pushes the committed changes to the remote repository
func (g *GitOperations) PushCommits(ctx context.Context) error {
	if ctx.Err() != nil {
		return fmt.Errorf("operation cancelled: %w", ctx.Err())
	}

	return g.RetryOperation("push commits", func() error {
		_, err := g.RunCommand(ctx, "git", "push")
		if err != nil {
			return err
		}
		g.logger.Info("Successfully pushed commits")
		return nil
	})
}

// GetRandomMessage selects a random commit message using the selector
func (g *GitOperations) GetRandomMessage() string {
	return g.selector.SelectRandom()
}

// Logger returns the logger instance
func (g *GitOperations) Logger() *slog.Logger {
	return g.logger
}

// writeCommitFile handles writing commit information to a file
func (g *GitOperations) writeCommitFile(commitTime time.Time) error {
	content := commitTime.Format(dateFormat)
	content += fmt.Sprintf(" ts=%d", commitTime.UnixNano())

	if _, err := os.Stat(filename); err == nil {
		g.logger.Warn("File already exists, overwriting", "filename", filename)
	}

	return os.WriteFile(filename, []byte(content+"\n"), 0644)
}

// cleanupCommitFile removes the commit file after committing
func (g *GitOperations) cleanupCommitFile() {
	if err := os.Remove(filename); err != nil {
		g.logger.Warn("Failed to remove file", "filename", filename, "error", err)
	}
}

// processCommit orchestrates the commit process for a single commit time
func (g *GitOperations) processCommit(ctx context.Context, commitTime time.Time) error {
	if err := g.writeCommitFile(commitTime); err != nil {
		return fmt.Errorf("failed to write commit file: %w", err)
	}

	if err := g.CommitChanges(ctx, commitTime); err != nil { // Directly call CommitChanges
		return err
	}

	g.cleanupCommitFile()
	return nil
}

// processSingleDay processes all commits for a single day
func (g *GitOperations) processSingleDay(ctx context.Context, date time.Time, lastCommitTime *time.Time) error {
	if dateutil.ShouldSkipDay(date) {
		g.logger.Info("Skipping day", "date", date.Format(shortDateFormat))
		return nil
	}

	commitTimes := dateutil.GenerateCommitTimes(date)
	for _, commitTime := range commitTimes {
		// Ensure commitTime is after lastCommitTime
		if commitTime.Before(*lastCommitTime) || commitTime.Equal(*lastCommitTime) {
			commitTime = commitTime.Add(time.Nanosecond)
		}
		*lastCommitTime = commitTime

		if err := g.processCommit(ctx, commitTime); err != nil {
			return err
		}
	}
	return nil
}

// ProcessDateRange handles the range of dates for committing
func (g *GitOperations) ProcessDateRange(
	ctx context.Context,
	startDate, endDate time.Time,
) error {
	if err := g.EnsureGitRepository(ctx); err != nil {
		return err
	}

	if err := g.EnsureMainBranch(ctx); err != nil {
		return err
	}

	dayCount := 0
	lastCommitTime := time.Time{}
	for current := startDate; !current.After(endDate); current = current.AddDate(0, 0, 1) {
		select {
		case <-ctx.Done():
			return fmt.Errorf("operation cancelled")
		default:
			if err := g.processSingleDay(ctx, current, &lastCommitTime); err != nil {
				return fmt.Errorf(
					"failed to process date %s: %w",
					current.Format(shortDateFormat),
					err,
				)
			}

			dayCount++
			// Push commits every 10 days
			if dayCount%10 == 0 {
				if err := g.PushCommits(ctx); err != nil {
					return err
				}
			}
		}
	}

	// Final push
	return g.PushCommits(ctx)
}
