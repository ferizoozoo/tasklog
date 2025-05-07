use chrono::{Local, NaiveDate};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::fmt::Debug;

pub trait CommandArgs {
    fn validate(&self) -> Result<(), String>;
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
    /// List tasks with an optional limit
    LS(LSArgs),
    /// Add a new task
    #[command(visible_alias = "new")]
    Add(AddArgs),
    /// Analyze tasks
    Analyze(AnalyzeArgs),
    /// Mark a task as done
    Done {
        /// Task ID to mark as done
        id: usize,
    },
    /// Manage pomodoro sessions
    #[command(visible_alias = "pm")]
    Pomo(PomoArgs),
}

#[derive(Args, Debug)]
pub struct LSArgs {
    #[arg(short = 'l', long, default_value_t = 50)]
    pub limit: usize,
    #[arg(short = 'd', long, default_value_t = 1)]
    pub days: usize,
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

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Task title
    #[arg(short = 't', long)]
    pub title: String,

    /// Due date (YYYY-MM-DD)
    #[arg(short, long = "due-date", value_parser = parse_date)]
    pub due_date: Option<NaiveDate>,

    /// Task priority
    #[arg(short = 'p', long, value_enum, default_value_t = Priority::Medium)]
    pub priority: Priority,

    /// Task category
    #[arg(short = 'c', long)]
    pub category: Option<String>,
}

impl CommandArgs for AddArgs {
    fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if let Some(due_date) = &self.due_date {
            if due_date < &Local::now().naive_utc().date() {
                return Err("Due date cannot be before 2020-01-01".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Analyze tasks for n days before now
    #[arg(long="days",short = 'n', value_parser = parse_date, default_value_t=1)]
    pub days: usize,
}

impl CommandArgs for AnalyzeArgs {
    fn validate(&self) -> Result<(), String> {
        if self.days > 365 {
            return Err("Days cannot be greater than 365".to_string());
        };

        Ok(())
    }
}

#[derive(Args, Debug)]
pub struct PomoArgs {
    #[command(subcommand)]
    pub command: Option<PomoCommands>,

    /// Pomodoro session title
    #[arg(short = 't', long)]
    pub title: Option<String>,

    /// Session duration (e.g., 25m, 1h)
    #[arg(long, value_parser = parse_duration)]
    pub duration: Option<String>,

    /// Session category
    #[arg(short = 'c', long)]
    pub category: Option<String>,
}

impl CommandArgs for PomoArgs {
    fn validate(&self) -> Result<(), String> {
        if let Some(command) = &self.command {
            match command.validate() {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }

        if let Some(duration) = &self.duration {
            if duration.trim().is_empty() {
                return Err("Duration cannot be empty".to_string());
            }
        }

        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err("Title cannot be empty".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Subcommand)]
pub enum PomoCommands {
    /// View pomodoro logs
    Logs(PomoLogsArgs),
}

impl CommandArgs for PomoCommands {
    fn validate(&self) -> Result<(), String> {
        match self {
            PomoCommands::Logs(args) => args.validate(),
        }
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
    Low,
    Medium,
    High,
    Urgent,
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

impl From<usize> for Priority {
    fn from(n: usize) -> Self {
        match n {
            1 => Priority::Low,
            2 => Priority::Medium,
            3 => Priority::High,
            4 => Priority::Urgent,
            _ => Priority::Medium,
        }
    }
}

impl From<Priority> for usize {
    fn from(p: Priority) -> Self {
        match p {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Urgent => 4,
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

pub fn parse_date(date_str: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .map_err(|e| format!("Invalid date format: {}", e))
}

pub fn parse_duration(duration_str: &str) -> Result<String, String> {
    // This is just a placeholder for actual duration parsing
    // In a real implementation, you'd validate and parse the duration here
    Ok(duration_str.to_string())
}

pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Magenta,
    Cyan,
    White,
    Black,
}

pub fn format_string_with_color(str: &str, color: Color) -> String {
    match color {
        Color::Red => format!("\x1b[91m{}\x1b[0m", str),
        Color::Green => format!("\x1b[92m{}\x1b[0m", str),
        Color::Blue => format!("\x1b[94m{}\x1b[0m", str),
        Color::Yellow => format!("\x1b[93m{}\x1b[0m", str),
        Color::Magenta => format!("\x1b[95m{}\x1b[0m", str),
        Color::Cyan => format!("\x1b[96m{}\x1b[0m", str),
        Color::White => format!("\x1b[97m{}\x1b[0m", str),
        Color::Black => format!("\x1b[90m{}\x1b[0m", str),
        _ => str.to_string(),
    }
}

