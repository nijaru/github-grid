package dateutil

import (
	"math/rand"
	"sort"
	"time"
)

// Constants related to date utilities
const (
	commitTimeStartHour   = 9
	commitTimeEndHour     = 22
	commitReductionFactor = 2

	skipWeekdayChance = 1.0 / 8.0 // 12.5%
	skipWeekendChance = 1.0 / 6.0 // 16.67%

	weekendCommitLimit = 4  // Maximum commits on weekends
	weekdayCommitLimit = 16 // Maximum commits on weekdays
)

// ShouldSkipDay determines whether to skip committing on a given day
func ShouldSkipDay(date time.Time) bool {
	if IsWeekend(date) {
		return rand.Float64() < skipWeekendChance
	}
	return rand.Float64() < skipWeekdayChance
}

// IsWeekend checks if a given date falls on a weekend
func IsWeekend(date time.Time) bool {
	weekday := date.Weekday()
	return weekday == time.Saturday || weekday == time.Sunday
}

// GenerateCommitTimes generates random commit times for a given date
func GenerateCommitTimes(date time.Time) []time.Time {
	var dailyCommits int
	if IsWeekend(date) {
		dailyCommits = rand.Intn(
			weekendCommitLimit + 1,
		) // rand.Intn is exclusive of the upper bound
	} else {
		dailyCommits = rand.Intn(weekdayCommitLimit + 1)
	}

	if dailyCommits > 5 && rand.Float64() < 0.5 {
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
