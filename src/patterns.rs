use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Weekday, Datelike};
use rand::{rng, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub date: DateTime<Local>,
    pub message: String,
}

pub trait Pattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo>;
}

// Deterministic RNG seeded by date for consistent results
fn date_rng(date: NaiveDate) -> ChaCha8Rng {
    // Add microsecond entropy to vary between runs while keeping dates consistent
    let base_seed = date.num_days_from_ce() as u64;
    let time_entropy = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_micros() as u64;
    
    // Mix seeds so same dates still cluster similarly but with run variation
    let seed = base_seed.wrapping_mul(1000000) + (time_entropy % 1000);
    ChaCha8Rng::seed_from_u64(seed)
}

// Base intensity levels with ranges
#[derive(Debug, Clone)]
pub enum IntensityLevel {
    Casual,      // ~300/year
    Active,      // ~2,500/year  
    Maintainer,  // ~5,000/year
    Hyperactive, // ~12,000/year
    Extreme,     // ~20,000+/year
}

impl IntensityLevel {
    fn get_weekday_range(&self) -> (u32, u32) {
        match self {
            IntensityLevel::Casual => (0, 5),        // Often zero, max 5
            IntensityLevel::Active => (0, 15),       // Varied activity
            IntensityLevel::Maintainer => (2, 25),   // Regular but varied
            IntensityLevel::Hyperactive => (5, 45),  // Heavy but realistic
            IntensityLevel::Extreme => (10, 80),     // Still human
        }
    }
    
    fn get_weekend_range(&self) -> (u32, u32) {
        match self {
            IntensityLevel::Casual => (0, 3),       // Rarely work weekends
            IntensityLevel::Active => (0, 5),       // Occasional weekend
            IntensityLevel::Maintainer => (0, 10),  // Sometimes on call
            IntensityLevel::Hyperactive => (0, 20), // Startup life
            IntensityLevel::Extreme => (2, 35),     // Always on
        }
    }
    
    fn get_work_probability(&self) -> f64 {
        match self {
            IntensityLevel::Casual => 0.15,      // Work 1-2 days/week
            IntensityLevel::Active => 0.65,      // Work 4-5 days/week  
            IntensityLevel::Maintainer => 0.75,  // Work most weekdays
            IntensityLevel::Hyperactive => 0.85, // Almost daily
            IntensityLevel::Extreme => 0.92,     // Rarely take breaks
        }
    }
}

// Weekly rhythm multipliers (realistic work patterns with slight randomization)
fn get_weekly_multiplier(weekday: Weekday, rng: &mut ChaCha8Rng) -> f64 {
    let base = match weekday {
        Weekday::Mon => 0.7,  // Monday blues
        Weekday::Tue => 1.1,  // Peak productivity
        Weekday::Wed => 1.1,  // Peak productivity
        Weekday::Thu => 1.1,  // Peak productivity
        Weekday::Fri => 0.8,  // Winding down
        Weekday::Sat => 0.6,  // Lighter weekends
        Weekday::Sun => 0.6,  // Lighter weekends
    };
    
    // Add ±5% randomization to avoid exact patterns
    let variation = rng.random_range(-0.05..=0.05);
    f64::max(base + variation, 0.1) // Ensure positive multiplier
}

// Configuration for pattern generation
#[derive(Debug, Clone)]
pub struct PatternConfig {
    pub intensity: IntensityLevel,
    pub use_weekly_rhythm: bool,
    pub vacation_frequency: f64,    // Probability per day of starting vacation
    pub vacation_duration: (u32, u32), // Min/max vacation days
    pub spike_probability: f64,     // Chance of high-activity days
    pub spike_multiplier: f64,      // Multiplier for spike days
}

impl PatternConfig {
    pub fn casual() -> Self {
        Self {
            intensity: IntensityLevel::Casual,
            use_weekly_rhythm: false, // Casual devs work irregularly
            vacation_frequency: 0.02, // Occasional breaks
            vacation_duration: (0, 0),
            spike_probability: 0.08,  // Occasional burst days
            spike_multiplier: 2.5,
        }
    }
    
    pub fn active() -> Self {
        Self {
            intensity: IntensityLevel::Active,
            use_weekly_rhythm: true,
            vacation_frequency: 0.03, // Regular breaks
            vacation_duration: (2, 7),
            spike_probability: 0.12,  // Regular feature days
            spike_multiplier: 2.0,
        }
    }
    
    pub fn maintainer() -> Self {
        Self {
            intensity: IntensityLevel::Maintainer,
            use_weekly_rhythm: true,
            vacation_frequency: 0.04,  // Frequent breaks
            vacation_duration: (3, 10),
            spike_probability: 0.15,   // Frequent busy days
            spike_multiplier: 1.8,
        }
    }
    
    pub fn hyperactive() -> Self {
        Self {
            intensity: IntensityLevel::Hyperactive,
            use_weekly_rhythm: true,
            vacation_frequency: 0.025, // Still takes breaks
            vacation_duration: (2, 5),
            spike_probability: 0.20,   // Many marathon sessions
            spike_multiplier: 2.2,
        }
    }
    
    pub fn extreme() -> Self {
        Self {
            intensity: IntensityLevel::Extreme,
            use_weekly_rhythm: true,
            vacation_frequency: 0.02,  // Burnout prevention
            vacation_duration: (1, 4),
            spike_probability: 0.25,   // Constant sprints
            spike_multiplier: 2.5,
        }
    }
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

// Generic pattern generator using configuration
pub struct ConfigurablePattern {
    config: PatternConfig,
}

impl ConfigurablePattern {
    pub fn new(config: PatternConfig) -> Self {
        Self { config }
    }
    
    fn should_work_today(&self, date: NaiveDate, rng: &mut ChaCha8Rng) -> bool {
        let base_probability = self.config.intensity.get_work_probability();
        let is_weekend = matches!(date.weekday(), Weekday::Sat | Weekday::Sun);
        
        // Significantly reduce weekend work for all patterns
        let mut probability = if is_weekend {
            match self.config.intensity {
                IntensityLevel::Casual => 0.05,      // 5% chance
                IntensityLevel::Active => 0.15,      // 15% chance
                IntensityLevel::Maintainer => 0.25,  // 25% chance
                IntensityLevel::Hyperactive => 0.35, // 35% chance
                IntensityLevel::Extreme => 0.50,     // 50% chance
            }
        } else {
            base_probability
        };
        
        // Add more randomization to create natural variance (±10%)
        let variation = rng.random_range(-0.1..=0.1);
        probability = (probability + variation).clamp(0.0, 1.0);
        
        rng.random::<f64>() < probability
    }
    
    fn get_base_commits(&self, date: NaiveDate, rng: &mut ChaCha8Rng) -> u32 {
        let is_weekend = matches!(date.weekday(), Weekday::Sat | Weekday::Sun);
        
        let range = if is_weekend {
            self.config.intensity.get_weekend_range()
        } else {
            self.config.intensity.get_weekday_range()
        };
        
        let mut commits = rng.random_range(range.0..=range.1);
        
        // Apply weekly rhythm if enabled
        if self.config.use_weekly_rhythm {
            let multiplier = get_weekly_multiplier(date.weekday(), rng);
            commits = (commits as f64 * multiplier) as u32;
        }
        
        // Apply spike days with more dramatic effect
        if rng.random::<f64>() < self.config.spike_probability {
            let spike_variation = rng.random_range(0.5..=1.5); // 50% to 150% extra
            let multiplier = self.config.spike_multiplier + spike_variation;
            commits = (commits as f64 * multiplier) as u32;
            // Cap spikes at reasonable levels
            commits = commits.min(match self.config.intensity {
                IntensityLevel::Casual => 15,
                IntensityLevel::Active => 40,
                IntensityLevel::Maintainer => 60,
                IntensityLevel::Hyperactive => 100,
                IntensityLevel::Extreme => 150,
            });
        }
        
        // Allow zero commits sometimes even on "work" days
        if commits == 0 && rng.random::<f64>() < 0.3 {
            0  // 30% chance of zero commits even when "working"
        } else {
            commits.max(1)
        }
    }
}

impl Pattern for ConfigurablePattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        let mut commits = Vec::new();
        let mut in_vacation = false;
        let mut vacation_end = start;
        
        let mut current = start;
        while current <= end {
            let mut rng = date_rng(current);
            
            // Check for vacation start
            if !in_vacation && rng.random::<f64>() < self.config.vacation_frequency {
                let vacation_days = rng.random_range(
                    self.config.vacation_duration.0..=self.config.vacation_duration.1
                );
                vacation_end = current + chrono::Duration::days(vacation_days as i64);
                in_vacation = true;
            }
            
            // Skip vacation days
            if in_vacation {
                if current >= vacation_end {
                    in_vacation = false;
                }
                current = current.succ_opt().unwrap();
                continue;
            }
            
            // Check if working today
            if !self.should_work_today(current, &mut rng) {
                current = current.succ_opt().unwrap();
                continue;
            }
            
            // Generate commits for the day
            let day_commits = self.get_base_commits(current, &mut rng);
            
            for _ in 0..day_commits {
                let hour = rng.random_range(6..=23);
                let minute = rng.random_range(0..60);
                commits.push(create_commit_at_time(current, hour, minute));
            }
            
            current = current.succ_opt().unwrap();
        }
        
        commits.sort_by_key(|c| c.date);
        commits
    }
}

// Wrapper patterns using the new configurable system
pub struct RealisticPattern {
    inner: ConfigurablePattern,
}

pub struct SteadyPattern {
    inner: ConfigurablePattern,
}

pub struct SporadicPattern {
    inner: ConfigurablePattern,
}

pub struct ContractorPattern {
    inner: ConfigurablePattern,
}

pub struct CasualPattern {
    inner: ConfigurablePattern,
}

pub struct ActivePattern {
    inner: ConfigurablePattern,
}

pub struct MaintainerPattern {
    inner: ConfigurablePattern,
}

pub struct HyperactivePattern {
    inner: ConfigurablePattern,
}

pub struct ExtremePattern {
    inner: ConfigurablePattern,
}

// Implementation of all patterns using the new configurable system

impl RealisticPattern {
    pub fn new() -> Self {
        Self {
            inner: ConfigurablePattern::new(PatternConfig::active()),
        }
    }
}

impl Pattern for RealisticPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl SteadyPattern {
    pub fn new() -> Self {
        // Custom config for steady pattern
        let config = PatternConfig {
            intensity: IntensityLevel::Active,
            use_weekly_rhythm: false, // No weekly variation
            vacation_frequency: 0.005, // Very rare breaks
            vacation_duration: (1, 2),
            spike_probability: 0.02,   // Minimal spikes
            spike_multiplier: 1.2,     // Small spikes
        };
        Self {
            inner: ConfigurablePattern::new(config),
        }
    }
}

impl Pattern for SteadyPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl SporadicPattern {
    pub fn new() -> Self {
        // Custom config for sporadic pattern
        let config = PatternConfig {
            intensity: IntensityLevel::Active,
            use_weekly_rhythm: false,
            vacation_frequency: 0.02,  // Frequent breaks
            vacation_duration: (1, 5),
            spike_probability: 0.15,   // High spike chance
            spike_multiplier: 3.0,     // Big spikes
        };
        Self {
            inner: ConfigurablePattern::new(config),
        }
    }
}

impl Pattern for SporadicPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl ContractorPattern {
    pub fn new() -> Self {
        // Custom config for contractor pattern
        let config = PatternConfig {
            intensity: IntensityLevel::Active,
            use_weekly_rhythm: true,   // Strong weekday focus
            vacation_frequency: 0.008, // Regular time off
            vacation_duration: (2, 4),
            spike_probability: 0.08,
            spike_multiplier: 1.4,
        };
        Self {
            inner: ConfigurablePattern::new(config),
        }
    }
}

impl Pattern for ContractorPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl CasualPattern {
    pub fn new() -> Self {
        Self {
            inner: ConfigurablePattern::new(PatternConfig::casual()),
        }
    }
}

impl Pattern for CasualPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl ActivePattern {
    pub fn new() -> Self {
        Self {
            inner: ConfigurablePattern::new(PatternConfig::active()),
        }
    }
}

impl Pattern for ActivePattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl MaintainerPattern {
    pub fn new() -> Self {
        Self {
            inner: ConfigurablePattern::new(PatternConfig::maintainer()),
        }
    }
}

impl Pattern for MaintainerPattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl HyperactivePattern {
    pub fn new() -> Self {
        Self {
            inner: ConfigurablePattern::new(PatternConfig::hyperactive()),
        }
    }
}

impl Pattern for HyperactivePattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}

impl ExtremePattern {
    pub fn new() -> Self {
        Self {
            inner: ConfigurablePattern::new(PatternConfig::extreme()),
        }
    }
}

impl Pattern for ExtremePattern {
    fn generate(&self, start: NaiveDate, end: NaiveDate) -> Vec<CommitInfo> {
        self.inner.generate(start, end)
    }
}