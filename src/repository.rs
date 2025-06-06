use crate::models::{AnalyzeArgs, DurationField, PomoStatus};
use crate::{
    helper::get_home_directory,
    models::{DailyAnalysis, LSArgs, PomoTask, PomoType, Priority, Task, TaskStatus},
};
use chrono::{DateTime, Local};
use rusqlite::{named_params, params, Connection, ToSql};
use std::fs;

const DB_FILE_PATH: &str = "/.tasklog";
const DB_FILE_NAME: &str = "/db.sqlite";

const CREATE_TASKS_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS tasks (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        status INTEGER NOT NULL DEFAULT 0,
        title TEXT NOT NULL,
        due_date TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        priority INTEGER NOT NULL DEFAULT 2,
        category TEXT,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status);
    CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks (due_date);
    CREATE INDEX IF NOT EXISTS idx_tasks_category ON tasks (category);
"#;

const CREATE_POMODORO_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS pomodoro (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        type INTEGER NOT NULL DEFAULT 0,
        title TEXT NOT NULL,
        start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        end_time TIMESTAMP,
        duration INTEGER NOT NULL DEFAULT 1500, -- 25 minutes
        status INTEGER NOT NULL DEFAULT 0,
        category TEXT,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE INDEX IF NOT EXISTS idx_pomodoro_pomo_type ON pomodoro (pomo_type);
    CREATE INDEX IF NOT EXISTS idx_pomodoro_category ON pomodoro (category);
    CREATE INDEX IF NOT EXISTS idx_pomodoro_quited ON pomodoro (quited);
"#;

const GET_TASK_BY_ID: &str = r#"
    SELECT id, status, title, due_date, priority, category FROM tasks
        WHERE id = :id"#;

const GET_TASKS: &str = r#"
    SELECT id, status, title, due_date, priority, category FROM tasks
        WHERE due_date <= :due_date {{where_category}} {{where_priority}} {{where_status}}
        ORDER BY created_at
        LIMIT :limit"#;

const INSERT_TASK: &str = r#"
    INSERT INTO tasks (status, title, due_date, priority, category)
        VALUES (:status, :title, :due_date, :priority, :category)
"#;

const DONE_TASK: &str = r#"UPDATE tasks SET status = 1 WHERE id = :id"#;

const UPDATE_POMODORO: &str = r#"
    UPDATE pomodoro
    SET
        status = :status,
        end_time = :end_date
    WHERE id = :id"#;

const INSERT_POMO: &str = r#"
INSERT INTO pomodoro (type, title, start_time, duration, status, category)
    VALUES (:type, :title, :start_time, :duration, :status, :category)
    RETURNING id
"#;

const GET_POMODORO_LIST: &str = r#"
    SELECT
        id, type, title, start_time, end_time, duration, status, category
    FROM pomodoro
    ORDER BY start_time DESC
    limit :limit
"#;

/// Count All the tasks that are done and ones that are pending
const GET_TASK_ANALYTICS: &str = r#"
WITH due_dates AS (SELECT
    DATE(due_date) AS date,
    COUNT(*) as total, 
    COUNT (CASE WHEN status = 0 THEN 1 ELSE null END) AS pending,
    COUNT (CASE WHEN (status = 1 and DATE(updated_at) >= DATE(due_date)) THEN 1 ELSE null END) AS delivered_over_due
    FROM tasks
    WHERE due_date >= :start_date AND due_date <=  :end_date
    GROUP BY DATE(due_date)
    )
SELECT 
    dd.date,
    dd.total, 
    dd.delivered_over_due, 
    dd.pending,
    ( (dd.total - dd.pending - dd.delivered_over_due) * 100.00 ) / dd.total as burn_down_rate
FROM 
    due_dates dd 
ORDER BY 
    dd.date 
DESC
"#;

// NOTE: The 'Connection' as Ok value type of Result can become more generic later
pub fn init_db(home_dir: String) -> Result<(), String> {
    let mut path = home_dir + DB_FILE_PATH;

    if let Ok(ok) = fs::exists(&path) {
        if !ok {
            if let Err(err) = fs::create_dir_all(&path) {
                return Err(err.to_string());
            }
        }
    }

    path += DB_FILE_NAME;

    let conn = match Connection::open(path) {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    match conn.execute(CREATE_TASKS_TABLE, []) {
        Ok(_) => {}
        Err(err) => return Err(err.to_string()),
    };

    match conn.execute(CREATE_POMODORO_TABLE, []) {
        Ok(_) => {}
        Err(err) => return Err(err.to_string()),
    };

    Ok(())
}

fn get_connection() -> Result<Connection, String> {
    let home_dir = match get_home_directory() {
        Ok(val) => val,
        Err(_) => {
            return Err(
                "Could connect to the database, please run the init command again".to_string(),
            )
        }
    };

    let path = home_dir + DB_FILE_PATH + DB_FILE_NAME;

    Connection::open(path).map_err(|err| err.to_string())
}

// TODD: add priority and category filters later.
pub fn get_tasks(ls_args: &LSArgs) -> Result<Vec<Task>, String> {
    let conn = get_connection()?;
    let now = Local::now();
    let due_date = (now + chrono::Duration::days(ls_args.days as i64)).to_rfc3339();

    let mut query = GET_TASKS.to_string();
    let mut params_values: Vec<(&str, &dyn ToSql)> =
        vec![(":due_date", &due_date), (":limit", &ls_args.limit)];

    let p_value: usize;
    match ls_args.priority {
        None => query = query.replace("{{where_priority}}", ""),
        Some(p) => {
            query = query.replace("{{where_priority}}", "AND priority = :priority");
            p_value = p as usize;
            params_values.push((":priority", &p_value));
        }
    };

    let category_value: String;
    let query = match &ls_args.category {
        None => query.replace("{{where_category}}", ""),
        Some(category) => {
            category_value = category.clone();
            params_values.push((":category", &category_value));
            query.replace("{{where_category}}", "AND category = :category")
        }
    };

    let status_value: usize;
    let query = match &ls_args.status {
        None => query.replace("{{where_status}}", "AND status = 0"),
        Some(status) => match status {
            TaskStatus::Done | TaskStatus::Open => {
                status_value = status.to_usize();
                params_values.push((":status", &status_value));
                query.replace("{{where_status}}", "AND status = :status")
            }
            _ => query.replace("{{where_status}}", ""),
        },
    };

    let mut stmt = conn
        .prepare(query.as_str())
        .map_err(|err| err.to_string())?;

    let tasks_iter = stmt.query_map(params_values.as_slice(), parse_task);

    let tasks_iter = match tasks_iter {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let mut tasks: Vec<Task> = Vec::new();

    for task in tasks_iter {
        tasks.push(task.unwrap());
    }

    Ok(tasks)
}

pub fn save_task(task: &mut Task) -> Result<(), String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let mut stmt = conn.prepare(INSERT_TASK).map_err(|err| err.to_string())?;

    println!("{:?}", task);

    let status = task.status as usize;
    println!("status as usize: {}", status);

    let res = stmt.execute(named_params! {
        ":status": status,
        ":title": task.title,
        ":due_date": task.due_date.to_rfc3339(),
        ":priority": task.priority as usize,
        ":category": task.category,
    });

    match res {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

pub fn get_task_by_id(task_id: usize) -> Result<Task, String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    conn.query_row(GET_TASK_BY_ID, params![task_id], parse_task)
        .map_err(|err| err.to_string())
}

pub fn done_task(task_id: usize) -> Result<(), String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let rows_affected = conn
        .execute(DONE_TASK, params![task_id])
        .map_err(|err| err.to_string())?;

    if rows_affected == 0 {
        return Err("Could not mark task as done".to_string());
    };

    Ok(())
}

pub fn get_pomodoro(ls_args: &LSArgs) -> Result<Vec<PomoTask>, String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };
    let mut stmt = match conn.prepare(GET_POMODORO_LIST) {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let pomo_tasks_iter = stmt.query_map(
        named_params! {
            ":limit": ls_args.limit,
        },
        parse_pomo_task,
    );

    let pomo_tasks_iter = match pomo_tasks_iter {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let mut pomo_tasks: Vec<PomoTask> = Vec::new();

    for pomo_task in pomo_tasks_iter {
        pomo_tasks.push(pomo_task.unwrap());
    }

    Ok(pomo_tasks)
}

pub fn add_pomodoro(pomo_task: &mut PomoTask) -> Result<(), String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let mut stmt = conn.prepare(INSERT_POMO).map_err(|e| e.to_string())?;

    let start_time = Local::now();
    let res = stmt.query_row(
        named_params! {
            ":type": pomo_task.pomo_type.to_usize(),
            ":title": pomo_task.title,
            ":category": pomo_task.category,
            ":duration": pomo_task.duration.to_i64(),
            ":start_time": start_time.to_rfc3339(),
            ":status": pomo_task.status.to_usize(),

        },
        |row| Ok(row.get::<_, u64>(0)),
    );

    let id = match res {
        Ok(val) => val.unwrap(),
        Err(err) => return Err(err.to_string()),
    };

    pomo_task.id = id;
    pomo_task.start_time = start_time;

    Ok(())
}

pub fn update_pomodoro(pomo: &PomoTask) -> Result<(), String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let rows_affected = conn.execute(
        UPDATE_POMODORO,
        named_params! {
            ":id": pomo.id,
            ":status": pomo.status.to_usize(),
            ":end_date": pomo.end_time.to_rfc3339(),
        },
    );

    match rows_affected {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

pub fn get_analysis(ls_args: &AnalyzeArgs) -> Result<Vec<DailyAnalysis>, String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let base_date_time = Local::now() - chrono::Duration::days(ls_args.days as i64);
    let base_date = base_date_time.to_utc();
    let end_date = (Local::now() + chrono::Duration::days(ls_args.days as i64)).to_utc();

    let mut stmt = match conn.prepare(GET_TASK_ANALYTICS) {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let daily_analytics_iter = stmt.query_map(
        named_params! {
            ":start_date": base_date.to_rfc3339(),
            ":end_date": end_date.to_rfc3339(),
        },
        parse_daily_analysis,
    );

    let daily_analytics_iter = match daily_analytics_iter {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let mut daily_analytics: Vec<DailyAnalysis> = Vec::new();

    for daily_analysis in daily_analytics_iter {
        let daily_analysis = match daily_analysis {
            Ok(val) => val,
            Err(err) => return Err(err.to_string()),
        };

        daily_analytics.push(daily_analysis);
    }

    Ok(daily_analytics)
}

fn parse_task(row: &rusqlite::Row) -> Result<Task, rusqlite::Error> {
    let date_str: String = row.get::<_, String>(3)?;
    let due_date: DateTime<Local> = DateTime::parse_from_rfc3339(&date_str)
        .unwrap()
        .with_timezone(&Local);
    Ok(Task {
        id: row.get(0)?,
        status: TaskStatus::from_usize(row.get::<_, usize>(1)?),
        title: row.get(2)?,
        due_date,
        priority: Priority::from_usize(row.get::<_, usize>(4)?),
        category: row.get(5)?,
    })
}

fn parse_pomo_task(row: &rusqlite::Row) -> Result<PomoTask, rusqlite::Error> {
    let start_date_str: String = row.get(3)?;
    let start_date = DateTime::parse_from_rfc3339(&start_date_str)
        .unwrap()
        .with_timezone(&Local);

    let end_date_str: String = row.get(4)?;
    let end_date = DateTime::parse_from_rfc3339(&end_date_str)
        .unwrap()
        .with_timezone(&Local);

    Ok(PomoTask {
        id: row.get(0)?,
        pomo_type: PomoType::from_usize(row.get::<_, usize>(1)?),
        title: row.get(2)?,
        duration: DurationField::from_i64(row.get::<_, i64>(5)?),
        category: row.get(7)?,
        status: PomoStatus::from_usize(row.get::<_, usize>(6)?),
        start_time: start_date,
        end_time: end_date,
    })
}

fn parse_daily_analysis(row: &rusqlite::Row) -> Result<DailyAnalysis, rusqlite::Error> {
    Ok(DailyAnalysis {
        date: row.get::<_, String>(0)?,
        total: row.get(1)?,
        pending: row.get(2)?,
        delivered_over_due: row.get(3)?,
        burn_down_rate: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use crate::models::TaskStatus;

    use super::*;
    use chrono::Duration;
    use std::fs;
    use std::path::Path;

    // Helper function to create a temporary database for testing
    fn setup_test_db() -> Result<(Connection, String), String> {
        // Create a temporary file path
        let db_path = ":memory:";

        // Ensure the database file does not exist
        if Path::new(db_path).exists() {
            fs::remove_file(db_path).map_err(|e| e.to_string())?;
        }

        // Initialize the database
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
        conn.execute(CREATE_TASKS_TABLE, [])
            .map_err(|e| e.to_string())?;
        conn.execute(CREATE_POMODORO_TABLE, [])
            .map_err(|e| e.to_string())?;

        Ok((conn, db_path.to_string()))
    }

    // Helper function to clean up the test database file
    fn cleanup_test_db(db_path: &str) -> Result<(), String> {
        if Path::new(db_path).exists() {
            fs::remove_file(db_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    #[test]
    fn test_done_task() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        // Insert a task
        let now = Local::now();
        let due_date = now + Duration::days(5);
        conn.execute(
            INSERT_TASK,
            params![0, "Task to Done", due_date.to_rfc3339(), 2, "Category E"],
        )
        .map_err(|e| e.to_string())?;
        let task_id = conn.last_insert_rowid();

        // Mark the task as done
        done_task(task_id as usize)?;

        // Verify the task's status was updated
        let updated_task = get_task_by_id(task_id as usize)?;
        assert_eq!(updated_task.status, TaskStatus::Done);

        cleanup_test_db(&db_path)?; // Clean up the database file
        Ok(())
    }
}
