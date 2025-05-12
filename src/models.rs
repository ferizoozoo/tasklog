use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub trait CommandArgs {
    fn validate(&self) -> Result<(), String>;
}

pub trait TableRow {
    fn headers(&self) -> Vec<&'static str>;
    fn row(&self) -> Vec<String>;
}

/// A formidable command-line tool for managing digital assets.
#[derive(Debug, Parser)]
#[command(author, version, about = "a cli based tasks manager tool", long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initializes the database and required files
    Init,
    /// List tasks or pomodoro session logs
    LS(LSArgs),
    /// Add a new task
    #[command(visible_alias = "new")]
    Add(Task),
    /// Analyze tasks or pomodoro sessions
    Analyze(AnalyzeArgs),
    /// Mark a task as done
    Done(DoneArgs),
    /// add pomodoro sessions
    #[command(visible_alias = "pm")]
    Pomo(PomoTask),
}

#[derive(Debug, Args)]
pub struct DoneArgs {
    #[arg(long, short)]
    /// Task Id to mark as done
    pub id: usize,
}

#[derive(Args, Debug)]
pub struct LSArgs {
    #[arg(short = 'l', long, default_value_t = 50)]
    pub limit: usize,
    #[arg(short = 'd', long, default_value_t = 1)]
    pub days: usize,
    #[arg(short = 'c', long)]
    pub category: Option<String>,
    #[arg(short = 'p', long)]
    pub priority: Option<Priority>,
    #[arg(short = 's', aliases = ["o"], value_enum, long)]
    pub status: Option<TaskStatus>,
    #[arg(short = 't', long = "type", value_enum, default_value_t = LSType::Task)]
    pub ls_type: LSType,
}

impl CommandArgs for LSArgs {
    fn validate(&self) -> Result<(), String> {
        if self.limit > 100 {
            return Err("Limit cannot be greater than 100".to_string());
        }
        if self.days > 365 {
            return Err("Days cannot be greater than 365".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, ValueEnum, Copy, Clone, Default)]
pub enum LSType {
    Task = 0,
    #[default]
    Pomo = 1,
}

impl From<LSType> for String {
    fn from(t: LSType) -> Self {
        match t {
            LSType::Task => "Task".to_string(),
            LSType::Pomo => "Pomo".to_string(),
        }
    }
}

impl From<String> for LSType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "task" => LSType::Task,
            "pomo" => LSType::Pomo,
            _ => LSType::Task,
        }
    }
}

impl LSType {
    pub fn from_usize(n: usize) -> LSType {
        match n {
            0 => LSType::Task,
            _ => LSType::Pomo,
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            LSType::Task => 0,
            LSType::Pomo => 1,
        }
    }
}

#[derive(Debug, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Default)]
pub enum TaskStatus {
    All = 2,
    Done = 1,
    #[default]
    Open = 0,
}
impl From<TaskStatus> for String {
    fn from(s: TaskStatus) -> Self {
        match s {
            TaskStatus::Done => "Done".to_string(),
            TaskStatus::Open => "Open".to_string(),
            TaskStatus::All => "All".to_string(),
        }
    }
}

impl From<String> for TaskStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "done" => TaskStatus::Done,
            "open" => TaskStatus::Open,
            _ => TaskStatus::All,
        }
    }
}

impl TaskStatus {
    pub fn is_done(&self) -> bool {
        *self == TaskStatus::Done
    }

    pub fn is_pending(&self) -> bool {
        *self == TaskStatus::Open
    }

    pub fn from_bool(b: bool) -> TaskStatus {
        if b {
            TaskStatus::Done
        } else {
            TaskStatus::Open
        }
    }

    pub fn from_usize(n: usize) -> TaskStatus {
        match n {
            0 => TaskStatus::Open,
            1 => TaskStatus::Done,
            _ => TaskStatus::All,
        }
    }

    pub fn to_usize(self) -> usize {
        match self {
            TaskStatus::Done => 1,
            TaskStatus::Open => 0,
            TaskStatus::All => 2,
        }
    }
}

#[derive(Args, Debug)]
pub struct Task {
    #[clap(skip)]
    pub id: u64,
    #[clap(skip)]
    pub status: TaskStatus,

    /// Task title
    #[arg(short = 't', long)]
    pub title: String,
    /// Task Due date in the form of "xDateUnit form now" or absolute date value (e.g: 1d, 1m, 1y or YYYY-MM-DD: 2025-10-01)
    #[arg(short, long = "due-date", value_parser = parse_date, default_value = "1d")]
    pub due_date: DateTime<Local>,
    /// Task priority
    #[arg(short = 'p', long, value_enum, default_value_t = Priority::Medium)]
    pub priority: Priority,
    /// Task category
    #[arg(short = 'c', long)]
    pub category: Option<String>,
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: 0,
            status: TaskStatus::Open,
            title: String::new(),
            due_date: Local::now() + Duration::days(1),
            priority: Priority::Medium,
            category: None,
        }
    }
}

impl CommandArgs for Task {
    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if self.due_date < Local::now() {
            return Err("Due date cannot be before 2020-01-01".to_string());
        }

        Ok(())
    }
}

impl TableRow for Task {
    fn headers(&self) -> Vec<&'static str> {
        vec!["id", "title", "due-date", "priority", "category", "status"]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.title.clone(),
            self.due_date.format("%Y-%m-%d").to_string(),
            String::from(self.priority),
            self.category.clone().unwrap_or_else(|| "-".to_string()),
            String::from(self.status),
        ]
    }
}

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Analyze tasks for n days before now
    #[arg(long="days",short = 'n', value_parser = parse_date, default_value="1d")]
    pub days: DateTime<Local>,
}

impl CommandArgs for AnalyzeArgs {
    fn validate(&self) -> Result<(), String> {
        let today = Local::now();
        if let Some(next_year) = today.with_year(today.year() + 1) {
            if self.days > next_year {
                return Err("Days cannot be greater than 365".to_string());
            };
        } else {
            return Err("Days cannot be greater than 365".to_string());
        }

        Ok(())
    }
}

#[derive(Default, Debug, ValueEnum, Copy, Clone, PartialEq)]
pub enum PomoType {
    Rest = 1,
    #[default]
    Work = 0,
}

impl From<String> for PomoType {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "rest" => PomoType::Rest,
            _ => PomoType::Work,
        }
    }
}

impl From<PomoType> for String {
    fn from(p: PomoType) -> Self {
        match p {
            PomoType::Rest => "Rest".to_string(),
            PomoType::Work => "Work".to_string(),
        }
    }
}

impl PomoType {
    pub fn from_usize(n: usize) -> PomoType {
        match n {
            0 => PomoType::Rest,
            _ => PomoType::Work,
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            PomoType::Rest => 0,
            PomoType::Work => 1,
        }
    }

    pub fn is_rest(&self) -> bool {
        matches!(*self, PomoType::Rest)
    }

    pub fn is_work(&self) -> bool {
        matches!(*self, PomoType::Work)
    }
}

#[derive(Debug, Clone)]
pub struct DurationField(chrono::Duration);

impl FromStr for DurationField {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let len = s.len();
        if len < 2 {
            return Err("Duration format must be a number followed by a unit (ms, s, m, h)".to_string());
        }

        // Check for milliseconds first since it's two characters
        if s.ends_with("ms") {
            let value_str = &s[..len - 2];
            let value: i64 = value_str
                .parse()
                .map_err(|_| format!("Cannot parse '{}' as a number", value_str))?;
            return Ok(DurationField(chrono::Duration::milliseconds(value)));
        }

        let (value_str, unit) = s.split_at(len - 1);
        let value: i64 = value_str
            .parse()
            .map_err(|_| format!("Cannot parse '{}' as a number", value_str))?;

        match unit {
            "s" => Ok(DurationField(chrono::Duration::seconds(value))),
            "m" => Ok(DurationField(chrono::Duration::minutes(value))),
            "h" => Ok(DurationField(chrono::Duration::hours(value))),
            _ => Err(format!("Invalid duration unit '{}'. Must be 'ms', 's', 'm', or 'h'", unit)),
        }
    }
}

impl From<DurationField> for String {
    fn from(d: DurationField) -> Self {
        let mut s = String::new();
        if d.0.num_milliseconds() < 1000 {
            s.push_str(&format!("{}ms", d.0.num_milliseconds()));
        } else if d.0.num_seconds() < 60 {
            s.push_str(&format!("{}s", d.0.num_seconds()));
        } else if d.0.num_minutes() < 60 {
            s.push_str(&format!("{}m", d.0.num_minutes()));
        } else {
            s.push_str(&format!("{}h", d.0.num_hours()));
        }
        s
    }
}

impl DurationField {
    pub fn to_i64(&self) -> i64 {
        if self.0.num_milliseconds() < 1000 {
            self.0.num_milliseconds()
        } else {
            self.0.num_seconds()
        }
    }

    pub fn from_i64(n: i64) -> Self {
        DurationField(Duration::seconds(n))
    }

    pub fn add_date(&self, date: &DateTime<Local>) -> DateTime<Local> {
        *date + self.0
    }

    pub fn to_time_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.0.num_seconds() as u64)
    }
}

impl From<DurationField> for chrono::Duration {
    fn from(d: DurationField) -> Self {
        d.0
    }
}

impl From<chrono::Duration> for DurationField {
    fn from(d: chrono::Duration) -> Self {
        DurationField(d)
    }
}

impl Default for DurationField {
    fn default() -> Self {
        DurationField(chrono::Duration::minutes(25))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum PomoStatus {
    Paused = 1,
    Finished = 2,
    #[default]
    Running = 0,
}

impl From<String> for PomoStatus {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "paused" => PomoStatus::Paused,
            "finished" => PomoStatus::Finished,
            _ => PomoStatus::Running,
        }
    }
}

impl From<PomoStatus> for String {
    fn from(p: PomoStatus) -> Self {
        match p {
            PomoStatus::Paused => "Paused".to_string(),
            PomoStatus::Finished => "Finished".to_string(),
            PomoStatus::Running => "Running".to_string(),
        }
    }
}

impl PomoStatus {
    pub fn from_usize(n: usize) -> PomoStatus {
        match n {
            0 => PomoStatus::Running,
            1 => PomoStatus::Paused,
            2 => PomoStatus::Finished,
            _ => PomoStatus::Running,
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            PomoStatus::Paused => 1,
            PomoStatus::Finished => 2,
            PomoStatus::Running => 0,
        }
    }
}

#[derive(Args, Debug)]
pub struct PomoTask {
    #[clap(skip)]
    pub id: u64,
    #[clap(skip)]
    pub status: PomoStatus,
    #[clap(skip)]
    pub pomo_type: PomoType,
    /// Session title
    #[arg(short = 't', long)]
    pub title: String,
    /// Session duration (e.g., 25m, 1h)
    #[arg(short='d',long, value_parser = parse_duration, default_value = "25m")]
    pub duration: DurationField,
    /// Session category
    #[arg(short = 'c', long)]
    pub category: Option<String>,
    #[clap(skip)]
    pub start_time: DateTime<Local>,
    #[clap(skip)]
    pub end_time: DateTime<Local>,
}

impl Default for PomoTask {
    fn default() -> Self {
        PomoTask {
            id: 0,
            pomo_type: PomoType::Work,
            title: "".to_string(),
            duration: DurationField::default(),
            category: None,
            start_time: Local::now(),
            end_time: Local::now() + Duration::minutes(25),
            status: PomoStatus::Running,
        }
    }
}

impl CommandArgs for PomoTask {
    fn validate(&self) -> Result<(), String> {
        if self.duration.0.is_zero() {
            return Err("Duration cannot be 0".to_string());
        }

        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        Ok(())
    }
}

impl TableRow for PomoTask {
    fn headers(&self) -> Vec<&'static str> {
        vec![
            "id",
            "title",
            "duration",
            "category",
            "status",
            "type",
            "start-time",
            "end-time",
        ]
    }
    fn row(&self) -> Vec<String> {
        vec![
            self.id.to_string(),
            self.title.clone(),
            String::from(self.duration.clone()),
            self.category.clone().unwrap_or_else(|| "-".to_string()),
            String::from(self.status.clone()),
            String::from(self.pomo_type),
            self.start_time.format("%Y-%m-%d %H:%M:%S").to_string(),
            self.end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]
    }
}

#[derive(Args, Debug)]
pub struct PomoLogsArgs {
    /// Show logs since date (YYYY-MM-DD)
    #[arg(long, value_parser = parse_date)]
    pub since: Option<NaiveDate>,
}

impl CommandArgs for PomoLogsArgs {
    fn validate(&self) -> Result<(), String> {
        if let Some(since) = &self.since {
            if since < &Local::now().naive_utc().date() {
                return Err("Since date cannot be before 2020-01-01".to_string());
            };
        };

        Ok(())
    }
}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
    Urgent = 4,
}

impl From<String> for Priority {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "urgent" => Priority::Urgent,
            _ => Priority::Medium,
        }
    }
}

impl From<Priority> for String {
    fn from(p: Priority) -> Self {
        match p {
            Priority::Low => "Low".to_string(),
            Priority::Medium => "Medium".to_string(),
            Priority::High => "High".to_string(),
            Priority::Urgent => "Urgent".to_string(),
        }
    }
}

impl Priority {
    pub fn from_usize(n: usize) -> Priority {
        match n {
            1 => Priority::Low,
            2 => Priority::Medium,
            3 => Priority::High,
            4 => Priority::Urgent,
            _ => Priority::Medium,
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Urgent => 4,
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::Low => write!(f, "Low"),
            Priority::Medium => write!(f, "Medium"),
            Priority::High => write!(f, "High"),
            Priority::Urgent => write!(f, "Urgent"),
        }
    }
}

pub fn parse_date(s: &str) -> Result<DateTime<Local>, String> {
    let len = s.len();
    if len < 2 {
        return Err(
            "argument format must be a number followed by a unit (d, M, y) like 10d for 10 days"
                .to_string(),
        );
    }

    let (value_str, unit) = s.split_at(len - 1);
    let value: u32 = value_str
        .parse()
        .map_err(|_| format!("Cannot parse '{}' as a number", value_str))?;

    match unit {
        "d" => Ok(Local::now() + Duration::days(value as i64)),
        "m" => {
            let date = Local::now();
            match date.with_month(date.month() + value) {
                Some(d) => Ok(d),
                None => Err(format!("could not parse date {}", s)),
            }
        }
        "y" => {
            let date = Local::now();
            match date.with_year(date.year() + (value as i32)) {
                Some(d) => Ok(d),
                None => Err(format!("could not parse date {}", s)),
            }
        }
        _ => {
            // check if possible to parse it like YYYY-MM-DD
            let date = DateTime::parse_from_str(s, "%Y-%m-%d").map_err(|e| e.to_string())?;

            Ok(date.with_timezone(&Local))
        }
    }
}

pub fn parse_duration(duration_str: &str) -> Result<DurationField, String> {
    DurationField::from_str(duration_str)
}

pub enum Color {
    Red,
    Green,
    Yellow,
    Cyan,
}

pub fn format_string_with_color(str: &str, color: Color) -> String {
    match color {
        Color::Red => format!("\x1b[91m{}\x1b[0m", str),
        Color::Green => format!("\x1b[92m{}\x1b[0m", str),
        Color::Yellow => format!("\x1b[93m{}\x1b[0m", str),
        Color::Cyan => format!("\x1b[96m{}\x1b[0m", str),
    }
}

#[derive(Clone, Debug)]
pub enum PomodoroEvent {
    Resize(u16, u16),
    Quit,
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub title: String,
    pub term_width: u16,
    pub term_height: u16,
    pub current_time: std::time::Duration,
    pub quited: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_conversions() {
        // Test from_usize
        assert_eq!(TaskStatus::from_usize(0), TaskStatus::Open);
        assert_eq!(TaskStatus::from_usize(1), TaskStatus::Done);
        assert_eq!(TaskStatus::from_usize(2), TaskStatus::All);
        assert_eq!(TaskStatus::from_usize(999), TaskStatus::All); // Invalid value defaults to All

        // Test to_usize
        assert_eq!(TaskStatus::Open.to_usize(), 0);
        assert_eq!(TaskStatus::Done.to_usize(), 1);
        assert_eq!(TaskStatus::All.to_usize(), 2);

        // Test from String
        assert_eq!(TaskStatus::from("done".to_string()), TaskStatus::Done);
        assert_eq!(TaskStatus::from("open".to_string()), TaskStatus::Open);
        assert_eq!(TaskStatus::from("invalid".to_string()), TaskStatus::All);

        // Test to String
        assert_eq!(String::from(TaskStatus::Done), "Done");
        assert_eq!(String::from(TaskStatus::Open), "Open");
        assert_eq!(String::from(TaskStatus::All), "All");

        // Test boolean conversions
        assert_eq!(TaskStatus::from_bool(true), TaskStatus::Done);
        assert_eq!(TaskStatus::from_bool(false), TaskStatus::Open);
    }

    #[test]
    fn test_priority_conversions() {
        // Test from_usize
        assert_eq!(Priority::from_usize(1), Priority::Low);
        assert_eq!(Priority::from_usize(2), Priority::Medium);
        assert_eq!(Priority::from_usize(3), Priority::High);
        assert_eq!(Priority::from_usize(4), Priority::Urgent);
        assert_eq!(Priority::from_usize(999), Priority::Medium); // Invalid value defaults to Medium

        // Test to_usize
        assert_eq!(Priority::Low.to_usize(), 1);
        assert_eq!(Priority::Medium.to_usize(), 2);
        assert_eq!(Priority::High.to_usize(), 3);
        assert_eq!(Priority::Urgent.to_usize(), 4);

        // Test from String
        assert_eq!(Priority::from("low".to_string()), Priority::Low);
        assert_eq!(Priority::from("medium".to_string()), Priority::Medium);
        assert_eq!(Priority::from("high".to_string()), Priority::High);
        assert_eq!(Priority::from("urgent".to_string()), Priority::Urgent);
        assert_eq!(Priority::from("invalid".to_string()), Priority::Medium);

        // Test to String
        assert_eq!(String::from(Priority::Low), "Low");
        assert_eq!(String::from(Priority::Medium), "Medium");
        assert_eq!(String::from(Priority::High), "High");
        assert_eq!(String::from(Priority::Urgent), "Urgent");
    }

    #[test]
    fn test_pomo_type_conversions() {
        // Test from_usize
        assert_eq!(PomoType::from_usize(0), PomoType::Rest);
        assert_eq!(PomoType::from_usize(1), PomoType::Work);
        assert_eq!(PomoType::from_usize(999), PomoType::Work); // Invalid value defaults to Work

        // Test to_usize
        assert_eq!(PomoType::Rest.to_usize(), 0);
        assert_eq!(PomoType::Work.to_usize(), 1);

        // Test from String
        assert_eq!(PomoType::from("rest".to_string()), PomoType::Rest);
        assert_eq!(PomoType::from("work".to_string()), PomoType::Work);
        assert_eq!(PomoType::from("invalid".to_string()), PomoType::Work);

        // Test to String
        assert_eq!(String::from(PomoType::Rest), "Rest");
        assert_eq!(String::from(PomoType::Work), "Work");

        // Test helper methods
        assert!(PomoType::Rest.is_rest());
        assert!(!PomoType::Rest.is_work());
        assert!(PomoType::Work.is_work());
        assert!(!PomoType::Work.is_rest());
    }

    #[test]
    fn test_pomo_status_conversions() {
        // Test from_usize
        assert_eq!(PomoStatus::from_usize(0), PomoStatus::Running);
        assert_eq!(PomoStatus::from_usize(1), PomoStatus::Paused);
        assert_eq!(PomoStatus::from_usize(2), PomoStatus::Finished);
        assert_eq!(PomoStatus::from_usize(999), PomoStatus::Running); // Invalid value defaults to Running

        // Test to_usize
        assert_eq!(PomoStatus::Running.to_usize(), 0);
        assert_eq!(PomoStatus::Paused.to_usize(), 1);
        assert_eq!(PomoStatus::Finished.to_usize(), 2);

        // Test from String
        assert_eq!(PomoStatus::from("running".to_string()), PomoStatus::Running);
        assert_eq!(PomoStatus::from("paused".to_string()), PomoStatus::Paused);
        assert_eq!(PomoStatus::from("finished".to_string()), PomoStatus::Finished);
        assert_eq!(PomoStatus::from("invalid".to_string()), PomoStatus::Running);

        // Test to String
        assert_eq!(String::from(PomoStatus::Running), "Running");
        assert_eq!(String::from(PomoStatus::Paused), "Paused");
        assert_eq!(String::from(PomoStatus::Finished), "Finished");
    }

    #[test]
    fn test_duration_field_conversions() {
        // Test from string parsing
        assert_eq!(DurationField::from_str("30s").unwrap().to_i64(), 30);
        assert_eq!(DurationField::from_str("5m").unwrap().to_i64(), 300);
        assert_eq!(DurationField::from_str("2h").unwrap().to_i64(), 7200);
        assert_eq!(DurationField::from_str("500ms").unwrap().to_i64(), 500);

        // Test invalid formats
        assert!(DurationField::from_str("").is_err());
        assert!(DurationField::from_str("abc").is_err());
        assert!(DurationField::from_str("5x").is_err());

        // Test to string formatting
        assert_eq!(String::from(DurationField::from_str("30s").unwrap()), "30s");
        assert_eq!(String::from(DurationField::from_str("5m").unwrap()), "5m");
        assert_eq!(String::from(DurationField::from_str("2h").unwrap()), "2h");
        assert_eq!(String::from(DurationField::from_str("500ms").unwrap()), "500ms");

        // Test to/from i64
        let duration = DurationField::from_i64(300); // 5 minutes
        assert_eq!(duration.to_i64(), 300);
    }
}
