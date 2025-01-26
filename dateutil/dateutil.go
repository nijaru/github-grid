package dateutil

import (
	"math/rand"
	"sort"
	"time"
)

// Constants related to date utilities
const (
	commitTimeStartHour   = 8
	commitTimeEndHour     = 20
	commitReductionFactor = 2

	skipWeekdayChance = 1.5 / 10.0 // 15%
	skipWeekendChance = 1.0 / 3.0  // 33.33%

	weekendCommitLimit = 8  // Maximum commits on weekends
	weekdayCommitLimit = 20 // Maximum commits on weekdays
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

// generateSingleCommitTime generates a single random commit time based on parameters
func generateSingleCommitTime(date time.Time, startHour, endHour int) time.Time {
	hour := rand.Intn(endHour-startHour+1) + startHour // Between startHour and endHour
	minute := rand.Intn(60)
	second := rand.Intn(60)
	nanosecond := rand.Intn(1e9) // Adds nanosecond precision
	return time.Date(
		date.Year(),
		date.Month(),
		date.Day(),
		hour,
		minute,
		second,
		nanosecond,
		date.Location(),
	)
}

// calculateDailyCommits determines the number of commits for the day
func calculateDailyCommits(isWeekend bool) int {
	if isWeekend {
		return rand.Intn(weekendCommitLimit + 1)
	}
	return rand.Intn(weekdayCommitLimit + 1)
}

// GenerateCommitTimes generates sorted commit times for a given date
func GenerateCommitTimes(date time.Time) []time.Time {
	isWeekend := IsWeekend(date)
	dailyCommits := calculateDailyCommits(isWeekend)

	if dailyCommits > 5 && rand.Float64() < 0.5 {
		dailyCommits /= commitReductionFactor
	}

	commitTimes := make([]time.Time, dailyCommits)
	for i := 0; i < dailyCommits; i++ {
		commitTimes[i] = generateSingleCommitTime(date, commitTimeStartHour, commitTimeEndHour)
	}

	sort.Slice(commitTimes, func(i, j int) bool {
		return commitTimes[i].Before(commitTimes[j])
	})

	return commitTimes
}
