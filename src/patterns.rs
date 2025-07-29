use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Weekday, Datelike};
use rand::{rng, Rng};

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub date: DateTime<Local>,
    pub message: String,
}

pub trait Pattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo>;
}

const COMMIT_MESSAGES: &[&str] = &[
    "[AutoGen] Add new feature implementation",
    "[AutoGen] Fix critical bug in core logic", 
    "[AutoGen] Refactor existing codebase",
    "[AutoGen] Add comprehensive tests",
    "[AutoGen] Update documentation",
    "[AutoGen] Optimize performance bottleneck",
    "[AutoGen] Implement user feedback",
    "[AutoGen] Fix merge conflicts",
    "[AutoGen] Add error handling",
    "[AutoGen] Update dependencies",
    "[AutoGen] Clean up code structure",
    "[AutoGen] Add logging and monitoring",
    "[AutoGen] Fix security vulnerability",
    "[AutoGen] Improve user interface",
    "[AutoGen] Add API endpoints",
    "[AutoGen] Fix failing tests",
    "[AutoGen] Add database migrations",
    "[AutoGen] Improve code coverage",
    "[AutoGen] Add configuration options",
    "[AutoGen] Fix production issue",
];

fn get_random_message() -> String {
    let mut rng = rng();
    COMMIT_MESSAGES[rng.random_range(0..COMMIT_MESSAGES.len())].to_string()
}

fn create_commit_at_time(date: NaiveDate, hour: u32, minute: u32) -> CommitInfo {
    let time = NaiveTime::from_hms_opt(hour, minute, 0).unwrap();
    let datetime = Local.from_local_datetime(&date.and_time(time)).unwrap();
    
    CommitInfo {
        date: datetime,
        message: get_random_message(),
    }
}

pub struct RealisticPattern {
    sprint_length: u32,
    vacation_probability: f64,
    spike_probability: f64,
}

impl RealisticPattern {
    pub fn new() -> Self {
        Self {
            sprint_length: 14,
            vacation_probability: 0.02, // 2% chance per day
            spike_probability: 0.05,    // 5% chance per day
        }
    }
    
    fn is_sprint_start(&self, date: NaiveDate) -> bool {
        date.ordinal() % self.sprint_length == 1
    }
    
    fn is_sprint_end(&self, date: NaiveDate) -> bool {
        date.ordinal() % self.sprint_length == 0
    }
    
    fn generate_vacation_period(&self, start: NaiveDate) -> Vec<NaiveDate> {
        let mut rng = rng();
        let length = rng.random_range(3..=10); // 3-10 day vacations
        
        (0..length)
            .map(|i| start + chrono::Duration::days(i as i64))
            .collect()
    }
    
    fn get_base_commits_for_day(&self, date: NaiveDate) -> u32 {
        let mut rng = rng();
        let is_weekend = matches!(date.weekday(), Weekday::Sat | Weekday::Sun);
        
        if is_weekend {
            // 30% chance of weekend work, usually 1-3 commits
            if rng.random::<f64>() < 0.3 {
                rng.random_range(1..=3)
            } else {
                0
            }
        } else {
            // Weekday: 2-8 commits normally
            rng.random_range(2..=8)
        }
    }
}

impl Pattern for RealisticPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        let mut commits = Vec::new();
        let mut rng = rng();
        let mut in_vacation = false;
        let mut vacation_end = start;
        
        let mut current = start;
        while current <= end {
            // Check if starting vacation
            if !in_vacation && rng.random::<f64>() < self.vacation_probability {
                let vacation_days = self.generate_vacation_period(current);
                vacation_end = vacation_days.last().copied().unwrap_or(current);
                in_vacation = true;
            }
            
            // Skip if in vacation
            if in_vacation {
                if current >= vacation_end {
                    in_vacation = false;
                }
                current = current.succ_opt().unwrap();
                continue;
            }
            
            let mut day_commits = self.get_base_commits_for_day(current);
            
            // Sprint modifiers
            if self.is_sprint_start(current) || self.is_sprint_end(current) {
                day_commits = (day_commits as f64 * 1.5) as u32;
            }
            
            // Random spike days
            if rng.random::<f64>() < self.spike_probability {
                day_commits += rng.random_range(10..=30); // Big feature day
            }
            
            // Generate commits for the day
            for _ in 0..day_commits {
                let hour = rng.random_range(9..=19); // Work hours
                let minute = rng.random_range(0..60);
                commits.push(create_commit_at_time(current, hour, minute));
            }
            
            current = current.succ_opt().unwrap();
        }
        
        // Sort commits by date
        commits.sort_by_key(|c| c.date);
        commits
    }
}

pub struct SteadyPattern {
    daily_commits: std::ops::Range<u32>,
}

impl SteadyPattern {
    pub fn new() -> Self {
        Self {
            daily_commits: 3..7,
        }
    }
}

impl Pattern for SteadyPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        let mut commits = Vec::new();
        let mut rng = rng();
        
        let mut current = start;
        while current <= end {
            let day_commits = rng.random_range(self.daily_commits.clone());
            
            for _ in 0..day_commits {
                let hour = rng.random_range(8..=20);
                let minute = rng.random_range(0..60);
                commits.push(create_commit_at_time(current, hour, minute));
            }
            
            current = current.succ_opt().unwrap();
        }
        
        commits.sort_by_key(|c| c.date);
        commits
    }
}

pub struct SporadicPattern {
    active_probability: f64,
    burst_probability: f64,
}

impl SporadicPattern {
    pub fn new() -> Self {
        Self {
            active_probability: 0.6, // 60% of days have commits
            burst_probability: 0.1,  // 10% chance of burst day
        }
    }
}

impl Pattern for SporadicPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        let mut commits = Vec::new();
        let mut rng = rng();
        
        let mut current = start;
        while current <= end {
            if rng.random::<f64>() < self.active_probability {
                let day_commits = if rng.random::<f64>() < self.burst_probability {
                    rng.random_range(15..=40) // Burst day
                } else {
                    rng.random_range(1..=5)   // Normal day
                };
                
                for _ in 0..day_commits {
                    let hour = rng.random_range(10..=22); // Extended hours for sporadic work
                    let minute = rng.random_range(0..60);
                    commits.push(create_commit_at_time(current, hour, minute));
                }
            }
            
            current = current.succ_opt().unwrap();
        }
        
        commits.sort_by_key(|c| c.date);
        commits
    }
}

pub struct ContractorPattern {
    weekend_probability: f64,
}

impl ContractorPattern {
    pub fn new() -> Self {
        Self {
            weekend_probability: 0.1, // 10% chance of weekend work
        }
    }
}

impl Pattern for ContractorPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        let mut commits = Vec::new();
        let mut rng = rng();
        
        let mut current = start;
        while current <= end {
            let is_weekend = matches!(current.weekday(), Weekday::Sat | Weekday::Sun);
            
            let day_commits = if is_weekend {
                if rng.random::<f64>() < self.weekend_probability {
                    rng.random_range(1..=4) // Light weekend work
                } else {
                    0
                }
            } else {
                rng.random_range(4..=12) // Regular weekday work
            };
            
            for _ in 0..day_commits {
                let hour = if is_weekend {
                    rng.random_range(10..=16) // Relaxed weekend hours
                } else {
                    rng.random_range(9..=17)  // Business hours
                };
                let minute = rng.random_range(0..60);
                commits.push(create_commit_at_time(current, hour, minute));
            }
            
            current = current.succ_opt().unwrap();
        }
        
        commits.sort_by_key(|c| c.date);
        commits
    }
}