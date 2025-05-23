use crate::models::PomoStatus;
use crate::parser::execute;
use crate::{
    helper::get_home_directory,
    models::{parse_date, parse_duration, LSArgs, PomoTask, PomoType, Priority, Task, TaskStatus},
};
use chrono::{Date, DateTime, Datelike, Duration, Local};
use rusqlite::{named_params, params, types::Value, Connection, ToSql};
use std::collections::HashMap;
use std::fs;
use std::ptr::replace;

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
        DESC limit :limit"#;

const INSERT_TASK: &str = r#"
    INSERT INTO tasks (status, title, due_date, pomo_type, category)
        VALUES (:status, :title, :due_date, :pomo_type, :category)
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
        Err(err) => {
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
    let due_date = now
        .with_day(now.day() + (ls_args.days as u32))
        .unwrap()
        .to_rfc3339();

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

pub fn get_pomodoro(ls_args: LSArgs) -> Result<Vec<PomoTask>, String> {
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
    let start_date = parse_date(&start_date_str).unwrap();

    let end_date_str: String = row.get(4)?;
    let end_date = parse_date(&end_date_str).unwrap();

    let duration_str: String = row.get(5)?;
    let duration = parse_duration(&duration_str).unwrap();

    Ok(PomoTask {
        id: row.get(0)?,
        pomo_type: PomoType::from_usize(row.get::<_, usize>(1)?),
        title: row.get(2)?,
        duration,
        category: row.get(7)?,
        status: PomoStatus::from_usize(row.get::<_, usize>(6)?),
        start_time: start_date,
        end_time: end_date,
    })
}

#[cfg(test)]
mod tests {
    use crate::models::DurationField;

    use super::*;
    use chrono::Duration;
    use std::fs;
    use std::path::Path;

    // Helper function to create a temporary database for testing
    fn setup_test_db() -> Result<(Connection, String), String> {
        // Create a temporary file path
        let db_path = "test_db.sqlite";

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
    fn test_get_tasks() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        // Insert some test data
        let now = Local::now();
        let task1_due = now + Duration::days(1);
        let task2_due = now + Duration::days(2);

        conn.execute(
            INSERT_TASK,
            params![0, "Task 1", task1_due.to_rfc3339(), 0, "Category A"],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            INSERT_TASK,
            params![0, "Task 2", task2_due.to_rfc3339(), 1, "Category B"],
        )
        .map_err(|e| e.to_string())?;

        // Test case 1: No filters
        let ls_args = LSArgs {
            days: 7,
            limit: 10,
            priority: None,
            category: None,
            status: None,
        };
        let tasks = get_tasks(&ls_args)?;
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].title, "Task 2"); // Check ordering

        // Test case 2: Filter by category
        let ls_args = LSArgs {
            days: 7,
            limit: 10,
            priority: None,
            category: Some("Category A".to_string()),
            status: None,
        };
        let tasks = get_tasks(&ls_args)?;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task 1");

        // Test case 3: Filter by priority
        let ls_args = LSArgs {
            days: 7,
            limit: 10,
            priority: Some(Priority::Medium),
            category: None,
            status: None,
        };
        let tasks = get_tasks(&ls_args)?;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Task 1");

        // Test case 4: Filter by status
        let ls_args = LSArgs {
            days: 7,
            limit: 10,
            priority: None,
            category: None,
            status: Some(TaskStatus::Open),
        };
        let tasks = get_tasks(&ls_args)?;
        assert_eq!(tasks.len(), 2);

        cleanup_test_db(&db_path)?; // Clean up the database file
        Ok(())
    }

    #[test]
    fn test_save_task() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        let now = Local::now();
        let mut task = Task {
            id: 0,
            status: TaskStatus::Open,
            title: "New Task".to_string(),
            due_date: now + Duration::days(3),
            priority: Priority::Low,
            category: Some("Category C".to_string()),
        };

        save_task(&mut task)?;
        assert_ne!(task.id, 0); // Check that the ID was set

        // Verify the task was saved correctly
        let saved_task = get_task_by_id(task.id as usize)?;
        assert_eq!(saved_task.title, "New Task");
        assert_eq!(saved_task.status, TaskStatus::Open);
        assert_eq!(saved_task.priority, Priority::Low);
        assert_eq!(saved_task.category, Some("Category C".to_string()));

        cleanup_test_db(&db_path)?; // Clean up the database file
        Ok(())
    }

    #[test]
    fn test_get_task_by_id() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        // Insert a task
        let now = Local::now();
        let due_date = now + Duration::days(4);
        conn.execute(
            INSERT_TASK,
            params![0, "Task to Get", due_date.to_rfc3339(), 2, "Category D"],
        )
        .map_err(|e| e.to_string())?;
        let task_id = conn.last_insert_rowid();

        // Retrieve the task by ID
        let task = get_task_by_id(task_id as usize)?;
        assert_eq!(task.title, "Task to Get");
        assert_eq!(task.id as i64, task_id);

        cleanup_test_db(&db_path)?; // Clean up the database file
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

    #[test]
    fn test_get_pomodoro() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        // Insert some test data
        let now = Local::now();

        conn.execute(
            INSERT_POMO,
            params![0, "Pomo 1", now.to_rfc3339(), 1500, 0, "Category X"],
        )
        .map_err(|e| e.to_string())?;
        conn.execute(
            INSERT_POMO,
            params![1, "Pomo 2", now.to_rfc3339(), 3000, 0, "Category Y"],
        )
        .map_err(|e| e.to_string())?;

        // Test case: Get all pomodoros
        let ls_args = LSArgs {
            days: 0,
            limit: 10,
            priority: None,
            category: None,
            status: None,
        };
        let pomos = get_pomodoro(ls_args)?;
        assert_eq!(pomos.len(), 2);
        assert_eq!(pomos[0].title, "Pomo 2"); // Check ordering

        cleanup_test_db(&db_path)?; // Clean up the database file
        Ok(())
    }

    #[test]
    fn test_add_pomodoro() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        let mut pomo_task = PomoTask {
            id: 0,
            status: PomoStatus::Paused,
            pomo_type: PomoType::Work,
            title: "New Pomo".to_string(),
            duration: DurationField(Duration::seconds(25 * 60)),
            category: Some("Category Z".to_string()),
            start_time: Local::now(),
            end_time: Local::now(),
        };

        add_pomodoro(&mut pomo_task)?;
        assert_ne!(pomo_task.id, 0);

        // Verify the pomodoro was added correctly.  Since get_pomodoro uses parse_date, and add_pomodoro uses Local::now(),
        //  we fetch the pomo directly and compare.
        let mut stmt = conn.prepare(GET_POMODORO_LIST).map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![1], |row| {
                //limit 1
                Ok(PomoTask {
                    id: row.get(0)?,
                    pomo_type: PomoType::from_usize(row.get::<_, usize>(1)?),
                    title: row.get(2)?,
                    duration: DurationField(Duration::seconds(row.get::<_, i64>(5)?)), // Extract as i64 first
                    category: row.get(7)?,
                    status: PomoStatus::from_usize(row.get::<_, usize>(6)?),
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Local),
                    end_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Local),
                })
            })
            .map_err(|e| e.to_string())?;

        let saved_pomo = rows.next().unwrap().unwrap(); // Get the first row
        assert_eq!(saved_pomo.title, "New Pomo");
        assert_eq!(saved_pomo.pomo_type, PomoType::Work);
        assert_eq!(saved_pomo.duration.0, Duration::seconds(25 * 60));
        assert_eq!(saved_pomo.category, Some("Category Z".to_string()));
        assert_eq!(saved_pomo.status, PomoStatus::Paused);

        cleanup_test_db(&db_path)?; // Clean up the database file
        Ok(())
    }

    #[test]
    fn test_update_pomodoro() -> Result<(), String> {
        let (conn, db_path) = setup_test_db()?;

        // Insert a pomodoro
        let now = Local::now();
        conn.execute(
            INSERT_POMO,
            params![0, "Pomo to Update", now.to_rfc3339(), 1500, 0, "Category A"],
        )
        .map_err(|e| e.to_string())?;
        let pomo_id = conn.last_insert_rowid();

        let mut pomo_task = PomoTask {
            id: pomo_id as u64,
            status: PomoStatus::Paused,
            pomo_type: PomoType::Work,
            title: "Pomo to Update".to_string(), //won'tbe updated
            duration: DurationField(Duration::seconds(25 * 60)),
            category: Some("Category A".to_string()), //won't be updated
            start_time: now,
            end_time: now,
        };

        // Update the pomodoro
        update_pomodoro(&pomo_task)?;

        // Verify the pomodoro was updated correctly
        let mut stmt = conn.prepare(GET_POMODORO_LIST).map_err(|e| e.to_string())?;
        let mut rows = stmt
            .query_map(params![1], |row| {
                Ok(PomoTask {
                    id: row.get(0)?,
                    pomo_type: PomoType::from_usize(row.get::<_, usize>(1)?),
                    title: row.get(2)?,
                    duration: DurationField(Duration::seconds(row.get::<_, i64>(5)?)),
                    category: row.get(7)?,
                    status: PomoStatus::from_usize(row.get::<_, usize>(6)?),
                    start_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Local),
                    end_time: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Local),
                })
            })
            .map_err(|e| e.to_string())?;
        let updated_pomo = rows.next().unwrap().unwrap();
        assert_eq!(updated_pomo.status, PomoStatus::Paused);

        cleanup_test_db(&db_path)?; // Clean up the database file
        Ok(())
    }
}
