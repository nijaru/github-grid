package main

import (
	"context"
	"flag"
	"log"
	"math/rand"
	"os"
	"os/exec"
	"os/signal"
	"strings"
	"time"
)

const (
	dateFormat      = "2006-01-02 15:04:05"
	shortDateFormat = "2006-01-02"
	filename        = "edit.txt"
)

var commitMessages = []struct {
	message string
	weight  int
}{
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

// Git-related functions

func runCommand(command string, args ...string) (string, error) {
	log.Printf("Running command: %s %s", command, strings.Join(args, " "))
	cmd := exec.Command(command, args...)
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Error running command: %s %s, output: %s, error: %v", command, strings.Join(args, " "), string(output), err)
		return "", err
	}
	return strings.TrimSpace(string(output)), nil
}

func setGitCommitDate(date string) func() {
	log.Printf("Setting GIT_COMMITTER_DATE to %s", date)
	os.Setenv("GIT_COMMITTER_DATE", date)
	return func() {
		log.Printf("Unsetting GIT_COMMITTER_DATE")
		os.Unsetenv("GIT_COMMITTER_DATE")
	}
}

func commitChanges(filename string, date time.Time) error {
	formattedDate := date.Format(dateFormat)
	defer setGitCommitDate(formattedDate)()

	if _, err := runCommand("git", "add", filename); err != nil {
		return err
	}

	commitMsg := getRandomMessage()
	if _, err := runCommand("git", "commit", "--date", formattedDate, "-m", commitMsg); err != nil {
		return err
	}

	log.Printf("Committed changes with message: %s", commitMsg)
	return nil
}

func pushCommits() error {
	log.Println("Pushing commits")
	if _, err := runCommand("git", "push"); err != nil {
		log.Printf("Error during git push: %v", err)
		return err
	}
	return nil
}

func getLastCommitInfo(branch string) (string, string, error) {
	log.Printf("Getting last commit info for branch: %s", branch)
	date, err := runCommand("git", "log", branch, "-1", "--format=%cd", "--date=format:"+shortDateFormat)
	if err != nil {
		return "", "", err
	}
	message, err := runCommand("git", "log", branch, "-1", "--format=%s")
	if err != nil {
		return "", "", err
	}
	return date, message, nil
}

func compareLastCommitMessages() (bool, error) {
	log.Println("Comparing last commit messages between main and dev branches")
	_, mainMsg, err := getLastCommitInfo("main")
	if err != nil {
		log.Printf("Error retrieving last commit info from main: %v", err)
		return false, err
	}
	_, devMsg, err := getLastCommitInfo("dev")
	if err != nil {
		log.Printf("Error retrieving last commit info from dev: %v", err)
		return false, err
	}
	return mainMsg == devMsg, nil
}

// Commit message generation

func getRandomMessage() string {
	var weightedMessages []string
	for _, cm := range commitMessages {
		for i := 0; i < cm.weight; i++ {
			weightedMessages = append(weightedMessages, cm.message)
		}
	}
	message := weightedMessages[rand.Intn(len(weightedMessages))]
	log.Printf("Generated random commit message: %s", message)
	return message
}

// Date and time handling

func isWeekend(date time.Time) bool {
	weekday := date.Weekday()
	return weekday == time.Saturday || weekday == time.Sunday
}

func flipCoin() bool {
	return rand.Intn(2) == 0
}

func shouldSkipDay(date time.Time) bool {
	return isWeekend(date) && !flipCoin()
}

func generateCommitTimes(date time.Time, dailyCommits int) []time.Time {
	var commitTimes []time.Time
	for len(commitTimes) < dailyCommits {
		hour := rand.Intn(14) + 9 // Between 9 and 22
		minute := rand.Intn(60)
		second := rand.Intn(60)
		commitTime := time.Date(date.Year(), date.Month(), date.Day(), hour, minute, second, 0, date.Location())
		commitTimes = append(commitTimes, commitTime)
	}
	log.Printf("Generated %d commit times for date: %s", dailyCommits, date.Format(shortDateFormat))
	return commitTimes
}

// Commit operations

func performDailyCommits(ctx context.Context, date time.Time, filename string) error {
	if shouldSkipDay(date) {
		log.Printf("Skipping weekend day: %s", date.Format(shortDateFormat))
		return nil
	}

	dailyCommits := rand.Intn(16)
	if isWeekend(date) {
		dailyCommits = rand.Intn(4)
	}

	if dailyCommits > 5 && flipCoin() {
		dailyCommits /= 2
	}

	commitTimes := generateCommitTimes(date, dailyCommits)

	for _, commitTime := range commitTimes {
		select {
		case <-ctx.Done():
			log.Println("Context cancelled, stopping commits")
			return ctx.Err()
		default:
			content := commitTime.Format(dateFormat)
			log.Printf("Writing commit content to file: %s", content)
			if err := os.WriteFile(filename, []byte(content+"\n"), 0644); err != nil {
				log.Printf("Error writing to file %s: %v", filename, err)
				continue
			}
			if err := commitChanges(filename, commitTime); err != nil {
				log.Printf("Error during commit process: %v", err)
			}
			if err := os.Remove(filename); err != nil {
				log.Printf("Error removing file %s: %v", filename, err)
			}
		}
	}

	log.Printf("Performed %d commits on %s", len(commitTimes), date.Format(shortDateFormat))
	return nil
}

func writeCommits(ctx context.Context, startDate, endDate time.Time) error {
	log.Printf("Switching to main branch")
	if _, err := runCommand("git", "switch", "main"); err != nil {
		log.Printf("Error during git switch: %v", err)
		return err
	}
	log.Printf("Resetting main branch to dev")
	if _, err := runCommand("git", "reset", "--hard", "dev"); err != nil {
		log.Printf("Error during git reset: %v", err)
		return err
	}
	log.Printf("Pushing changes with --force")
	if _, err := runCommand("git", "push", "--force"); err != nil {
		log.Printf("Error during git push --force: %v", err)
		return err
	}

	dayCounter := 0
	for startDate.Before(endDate) || startDate.Equal(endDate) {
		select {
		case <-ctx.Done():
			log.Println("Context cancelled, stopping writeCommits")
			return ctx.Err()
		default:
			if err := performDailyCommits(ctx, startDate, filename); err != nil {
				return err
			}
			startDate = startDate.AddDate(0, 0, 1)
			dayCounter++
			// Push commits every 10 days
			if dayCounter%10 == 0 {
				if err := pushCommits(); err != nil {
					return err
				}
			}
		}
	}

	// Final push to ensure all commits are pushed
	if err := pushCommits(); err != nil {
		return err
	}
	log.Println("Finished processing all dates. Exiting gracefully.")
	return nil
}

func catchUp(ctx context.Context) error {
	log.Println("Catching up with commits")
	lastCommitDate, _, err := getLastCommitInfo("main")
	if err != nil {
		log.Printf("Error retrieving last commit date from main: %v", err)
		return err
	}
	startDate, err := parseDate(lastCommitDate)
	if err != nil {
		log.Printf("Error parsing last commit date: %v", err)
		return err
	}
	endDate := time.Now().AddDate(0, 0, 1)
	return writeCommits(ctx, startDate, endDate)
}

// Signal handling

func handleInterrupt(cancel context.CancelFunc) {
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt)
	go func() {
		<-sigChan
		log.Println("Received SIGINT (Ctrl+C). Exiting gracefully...")
		cancel()
	}()
}

// Ensure we are on the main branch before running any git operations
func ensureMainBranch() error {
	currentBranch, err := runCommand("git", "rev-parse", "--abbrev-ref", "HEAD")
	if err != nil {
		return err
	}
	if currentBranch != "main" {
		log.Printf("Current branch is %s, switching to main", currentBranch)
		if _, err := runCommand("git", "switch", "main"); err != nil {
			return err
		}
	}
	return nil
}

// Utility functions for date parsing and formatting

func parseDate(dateStr string) (time.Time, error) {
	return time.Parse(shortDateFormat, dateStr)
}

func formatDate(date time.Time) string {
	return date.Format(shortDateFormat)
}

// Main process

func main() {
	startDateStr := flag.String("start", "", "Start date for the commits (format: YYYY-MM-DD)")
	endDateStr := flag.String("end", "", "End date for the commits (format: YYYY-MM-DD)")
	flag.Parse()

	// Set default values if flags are not provided
	var startDate, endDate time.Time
	var err error
	if *startDateStr == "" {
		startDate = time.Now().AddDate(0, 0, -371) // 53 weeks ago
	} else {
		startDate, err = parseDate(*startDateStr)
		if err != nil {
			log.Fatalf("Invalid start date format: %v", err)
		}
	}
	if *endDateStr == "" {
		endDate = time.Now()
	} else {
		endDate, err = parseDate(*endDateStr)
		if err != nil {
			log.Fatalf("Invalid end date format: %v", err)
		}
	}

	if startDate.After(endDate) {
		log.Fatalf("Start date cannot be after end date")
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	handleInterrupt(cancel)

	if err := ensureMainBranch(); err != nil {
		log.Fatalf("Failed to ensure main branch: %v", err)
	}

	sameMessages, err := compareLastCommitMessages()
	if err != nil {
		log.Fatalf("Failed to compare last commit messages: %v", err)
	}

	if sameMessages {
		if err := writeCommits(ctx, startDate, endDate); err != nil {
			log.Fatalf("Failed to write commits: %v", err)
		}
	} else {
		if err := catchUp(ctx); err != nil {
			log.Fatalf("Failed to catch up: %v", err)
		}
	}

	<-ctx.Done()
	log.Println("Program exited gracefully.")
}
