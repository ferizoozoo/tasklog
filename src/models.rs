use std::error::Error;
use chrone::DateTime;

trait Model {
    fn validate(&self) -> Result<(), &str>;
}

struct Task {
    ID: usize,
    title: String,
    description: String,
    priority: Priority,
    due_date: DateTime<Local> 
    status: TaskStatus
}

impl Model for Task {
    fn validate(&self) -> Result<(), &str> {
        if self.title.is_empty() {
            return "Title cannot be empty";
        }

        if self.due_date.is_none() {
            return "Due date cannot be empty";
        }

        if self.due_date < Local::now() {
            return "Due date cannot be in the past";
        }

        if self.priority.is_none() {
            return "Priority cannot be empty";
        }

        if self.status == TaskStatus::closed {
            return "Task cannot be closed";
        }

        Ok(())
    }
}

impl Default for Task {
    fn default() -> Self {
        Task {
            ID: 0,
            title: String::new(),
            description: String::new(),
            priority: Priority::Medium,
            due_date: Local::now(),
            status: TaskStatus::Open
        }
    }
}

struct PomoSession {
    ID: usize,
    start_time: DateTime<Local>,
    duration: Duration,
    end_time: DateTime<Local>
    sequence: usize, 
    type: PomoType
}

impl Model for PomoSession {
    fn validate(&self) -> Result<(), &str> {
        if self.start_time.is_none() {
            return "Start time cannot be empty";
        }

        if self.duration.is_none() {
            return "Duration cannot be empty";
        }

        if self.end_time.is_none() {
            return "End time cannot be empty";
        }
        
        if self.end_time < self.start_time {
            return "End time cannot be before start time";
        }

        if self.type.is_none() {
            return "Type cannot be empty";
        }

        Ok(())
    }
}

impl Default for PomoSession {
    fn default() -> Self {
        PomoSession {
            ID: 0,
            start_time: Local::now(),
            duration: Duration::new(),
            end_time: Local::now(),
            sequence: 0,
            type: PomoType::Work
        }
    }
}

enum Priority {
    Urgent,
    High,
    Medium,
    Low
}

enum PomoType {
    Rest,
    Work
}

enum TaskStatus {
    Open,
    Closed
}