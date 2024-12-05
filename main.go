package main

import (
	"flag"
	"fmt"
	"log/slog"
	"math/rand"
	"os"
	"os/exec"
	"os/signal"
	"sort"
	"strings"
	"time"
)

const (
	dateFormat            = "2006-01-02 15:04:05 -0700"
	shortDateFormat       = "2006-01-02"
	filename              = "edit.txt"
	maxRetries            = 3
	defaultStartDaysAgo   = -371
	commitTimeStartHour   = 9
	commitTimeEndHour     = 22
	commitReductionFactor = 2
	skipWeekdayChance     = 1.0 / 8.0 // 12.5%

	// Added constants
	weekendCommitLimit = 4  // Maximum commits on weekends
	weekdayCommitLimit = 16 // Maximum commits on weekdays
)

type CommitMessage struct {
	Message string
	Weight  int
}

var commitMessages = []CommitMessage{
	{"Add a new feature", 10},
	{"Fix a bug", 8},
	{"Refactor some code", 6},
	{"Add a new test", 5},
	{"Update the requirements", 4},
	{"Update the documentation", 3},
	{"Update the README", 3},
	{"Update the license", 2},
	{"Update the gitignore", 1},
	{"Update the CI/CD pipeline", 1},
	{"Update the Dockerfile", 1},
	{"Update the Makefile", 1},
	{"Update the GitHub Actions", 1},
	{"Update the Jenkinsfile", 1},
	{"Update the AWS config", 1},
	{"Update the GCP config", 1},
	{"Update the Azure config", 1},
}

var weightedCommitMessages []string

func init() {
	rand.Seed(time.Now().UnixNano())
	for _, cm := range commitMessages {
		for i := 0; i < cm.Weight; i++ {
			weightedCommitMessages = append(weightedCommitMessages, cm.Message)
		}
	}
}

type GitOperations struct {
	logger     *slog.Logger
	shouldStop bool
}

func NewGitOperations(logger *slog.Logger) *GitOperations {
	return &GitOperations{
		logger:     logger,
		shouldStop: false,
	}
}

func (g *GitOperations) runCommand(command string, args ...string) (string, error) {
	if g.shouldStop {
		return "", fmt.Errorf("operation cancelled")
	}

	g.logger.Info("Running command", "command", command, "args", strings.Join(args, " "))
	cmd := exec.Command(command, args...)
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

func (g *GitOperations) retryOperation(description string, operation func() error) error {
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

func (g *GitOperations) ensureGitRepository() error {
	_, err := g.runCommand("git", "rev-parse", "--is-inside-work-tree")
	if err != nil {
		return fmt.Errorf("not a git repository: %w", err)
	}
	return nil
}

func (g *GitOperations) ensureMainBranch() error {
	currentBranch, err := g.runCommand("git", "rev-parse", "--abbrev-ref", "HEAD")
	if err != nil {
		return fmt.Errorf("failed to get current branch: %w", err)
	}

	if currentBranch != "main" {
		g.logger.Info("Switching to main branch", "from", currentBranch)
		err = g.retryOperation("switch to main branch", func() error {
			_, err := g.runCommand("git", "switch", "main")
			return err
		})
		if err != nil {
			return fmt.Errorf("failed to switch to main branch: %w", err)
		}
	}
	return nil
}

func (g *GitOperations) commitChanges(filename string, date time.Time) error {
	if g.shouldStop {
		return fmt.Errorf("operation cancelled")
	}

	formattedDate := date.Format(dateFormat)
	commitMsg := getRandomMessage()

	return g.retryOperation("commit changes", func() error {
		if _, err := g.runCommand("git", "add", filename); err != nil {
			return fmt.Errorf("git add failed: %w", err)
		}

		// Execute commit with GIT_COMMITTER_DATE set
		cmd := exec.Command("git", "commit", "--date", formattedDate, "-m", commitMsg)
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

func (g *GitOperations) pushCommits() error {
	if g.shouldStop {
		return fmt.Errorf("operation cancelled")
	}

	return g.retryOperation("push commits", func() error {
		_, err := g.runCommand("git", "push")
		if err != nil {
			return err
		}
		g.logger.Info("Successfully pushed commits")
		return nil
	})
}

func processDateRange(git *GitOperations, startDate, endDate time.Time) error {
	if err := git.ensureGitRepository(); err != nil {
		return err
	}

	if err := git.ensureMainBranch(); err != nil {
		return err
	}

	dayCount := 0
	for current := startDate; !current.After(endDate); current = current.AddDate(0, 0, 1) {
		if git.shouldStop {
			return fmt.Errorf("operation cancelled")
		}

		if err := processSingleDay(git, current); err != nil {
			return fmt.Errorf("failed to process date %s: %w", current.Format(shortDateFormat), err)
		}

		dayCount++
		// Push commits every 10 days
		if dayCount%10 == 0 {
			if err := git.pushCommits(); err != nil {
				return err
			}
		}
	}

	// Final push
	return git.pushCommits()
}

func processSingleDay(git *GitOperations, date time.Time) error {
	if shouldSkipDay(date) {
		git.logger.Info("Skipping day", "date", date.Format(shortDateFormat))
		return nil
	}

	commitTimes := generateCommitTimes(date)
	for _, commitTime := range commitTimes {
		if git.shouldStop {
			return fmt.Errorf("operation cancelled")
		}

		if err := processCommit(git, commitTime); err != nil {
			return err
		}
	}
	return nil
}

func processCommit(git *GitOperations, commitTime time.Time) error {
	content := commitTime.Format(dateFormat)

	if err := os.WriteFile(filename, []byte(content+"\n"), 0644); err != nil {
		return fmt.Errorf("failed to write file: %w", err)
	}

	if err := git.commitChanges(filename, commitTime); err != nil {
		return err
	}

	if err := os.Remove(filename); err != nil {
		git.logger.Warn("Failed to remove file", "filename", filename, "error", err)
	}

	return nil
}

// Helper functions
func shouldSkipDay(date time.Time) bool {
	if isWeekend(date) {
		// Existing weekend logic: 50% chance to skip
		return !flipCoin()
	}
	// Weekday logic: 12.5% chance to skip
	return shouldSkipWeekday()
}

func isWeekend(date time.Time) bool {
	weekday := date.Weekday()
	return weekday == time.Saturday || weekday == time.Sunday
}

func shouldSkipWeekday() bool {
	// Flip three coins; skip if all three are true
	return flipCoin() && flipCoin() && flipCoin()
	// Alternatively, use probability directly:
	// return rand.Float64() < skipWeekdayChance
}

func flipCoin() bool {
	return rand.Intn(2) == 0
}

func generateCommitTimes(date time.Time) []time.Time {
	var dailyCommits int
	if isWeekend(date) {
		dailyCommits = rand.Intn(
			weekendCommitLimit + 1,
		) // rand.Intn is exclusive of the upper bound
	} else {
		dailyCommits = rand.Intn(weekdayCommitLimit + 1)
	}

	if dailyCommits > 5 && flipCoin() {
		dailyCommits /= commitReductionFactor
	}

	var commitTimes []time.Time
	for i := 0; i < dailyCommits; i++ {
		hour := rand.Intn(
			commitTimeEndHour-commitTimeStartHour+1,
		) + commitTimeStartHour // Between 9 and 22
		minute := rand.Intn(60)
		second := rand.Intn(60)
		commitTime := time.Date(
			date.Year(),
			date.Month(),
			date.Day(),
			hour,
			minute,
			second,
			0,
			date.Location(),
		)
		commitTimes = append(commitTimes, commitTime)
	}

	// Sort commit times to ensure chronological order
	sort.Slice(commitTimes, func(i, j int) bool {
		return commitTimes[i].Before(commitTimes[j])
	})

	return commitTimes
}

func getRandomMessage() string {
	return weightedCommitMessages[rand.Intn(len(weightedCommitMessages))]
}

func parseDateRange(startStr, endStr string) (time.Time, time.Time, error) {
	var startDate, endDate time.Time
	var err error

	if startStr == "" {
		startDate = time.Now().AddDate(0, 0, defaultStartDaysAgo) // Start from 371 days ago
	} else {
		startDate, err = time.Parse(shortDateFormat, startStr)
		if err != nil {
			return time.Time{}, time.Time{}, fmt.Errorf("invalid start date: %w", err)
		}
	}

	if endStr == "" {
		endDate = time.Now()
	} else {
		endDate, err = time.Parse(shortDateFormat, endStr)
		if err != nil {
			return time.Time{}, time.Time{}, fmt.Errorf("invalid end date: %w", err)
		}
	}

	if startDate.After(endDate) {
		return time.Time{}, time.Time{}, fmt.Errorf("start date cannot be after end date")
	}

	return startDate, endDate, nil
}

func main() {
	startDateStr := flag.String("start", "", "Start date (YYYY-MM-DD)")
	endDateStr := flag.String("end", "", "End date (YYYY-MM-DD)")
	flag.Parse()

	// Initialize a new logger with a text handler
	logger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		AddSource: true,
		Level:     slog.LevelInfo,
	}))
	git := NewGitOperations(logger)

	// Handle interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt)
	go func() {
		<-sigChan
		logger.Info("Received interrupt signal. Shutting down...")
		git.shouldStop = true
	}()

	startDate, endDate, err := parseDateRange(*startDateStr, *endDateStr)
	if err != nil {
		logger.Error("Failed to parse dates", "error", err)
		os.Exit(1)
	}

	if err := processDateRange(git, startDate, endDate); err != nil {
		logger.Error("Failed to process commits", "error", err)
		os.Exit(1)
	}

	logger.Info("Successfully completed all operations")
}
