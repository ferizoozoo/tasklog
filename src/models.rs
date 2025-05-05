use std::time::Duration;

use chrono::{DateTime, Local};

trait Model {
    fn validate(&self) -> Result<(), &str>;
}

struct Task {
    id: usize,
    title: String,
    description: String,
    priority: Priority,
    due_date: DateTime<Local>,
    status: TaskStatus,
}

impl Model for Task {
    fn validate(&self) -> Result<(), &str> {
        if self.title.is_empty() {
            return Err("Title cannot be empty");
        }

        if self.due_date < Local::now() {
            return Err("Due date cannot be in the past");
        }

        if let TaskStatus::Closed = self.status {
            return Err("Task cannot be closed");
        }

        Ok(())
    }
}

impl Default for Task {
    fn default() -> Self {
        Task {
            id: 0,
            title: String::new(),
            description: String::new(),
            priority: Priority::Medium,
            due_date: Local::now(),
            status: TaskStatus::Open,
        }
    }
}

struct PomoSession {
    id: usize,
    start_time: DateTime<Local>,
    duration: Duration,
    end_time: DateTime<Local>,
    sequence: usize,
    r#type: PomoType,
}

impl Model for PomoSession {
    fn validate(&self) -> Result<(), &str> {
        if self.end_time < self.start_time {
            return Err("End time cannot be before start time");
        }

        Ok(())
    }
}

impl Default for PomoSession {
    fn default() -> Self {
        PomoSession {
            id: 0,
            start_time: Local::now(),
            duration: Duration::new(0, 0),
            end_time: Local::now(),
            sequence: 0,
            r#type: PomoType::Work,
        }
    }
}

enum Priority {
    Urgent,
    High,
    Medium,
    Low,
}

enum PomoType {
    Rest,
    Work,
}

enum TaskStatus {
    Open,
    Closed,
}
