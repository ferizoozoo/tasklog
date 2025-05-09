use std::collections::HashMap;
use rusqlite::{params, Connection, ToSql, types::Value, named_params};
use std::fs;
use chrono::{Datelike, Duration, Local};
use crate::{
    helper::get_home_directory,
    models::{parse_date, parse_duration, LSArgs, PomoTask, PomoType, Priority, Task, TaskStatus},
};
use crate::parser::execute;

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
        pomo_type INTEGER NOT NULL DEFAULT 0,
        title TEXT NOT NULL,
        start_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        end_time TIMESTAMP,
        duration INTEGER NOT NULL DEFAULT 1500, -- 25 minutes
        completed BOOLEAN NOT NULL DEFAULT 0,
        category TEXT,
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

    CREATE INDEX IF NOT EXISTS idx_pomodoro_pomo_type ON pomodoro (pomo_type);
    CREATE INDEX IF NOT EXISTS idx_pomodoro_category ON pomodoro (category);
    CREATE INDEX IF NOT EXISTS idx_pomodoro_completed ON pomodoro (completed);
"#;

const GET_TASKS: &str = r#"
    SELECT id, status, title, due_date, priority, category FROM tasks
        WHERE due_date >= :due_date {{where_category}} {{where_priority}}
        ORDER BY created_at
        DESC limit :limit"#;

const INSERT_TASK: &str = r#"
    INSERT INTO tasks (status, title, due_date, priority, category)
        VALUES (:status, :title, :due_date, :priority, :category)
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
    let due_date = now.with_day(now.day()+(ls_args.days as u32)).unwrap().to_rfc3339();

    let p_value:usize;
    let category_value:String;
    
    let mut params_values:Vec<(&str, &dyn ToSql)> = vec![
        (":due_date", &due_date),
        (":limit", &ls_args.limit),
    ];

    let mut query = GET_TASKS.to_owned();
    match ls_args.priority {
        None => {
            query = query.replace("{{where_priority}}", "");
        },
        Some(p) => {
            query = query.replace("{{where_priority}}", "AND priority = :priority");
            p_value = p as usize;
            params_values.push((":priority", &p_value ));
        },
    };

    
    let query = match &ls_args.category {
        None => query.replace("{{where_category}}", ""),
        Some(category) => {
            category_value = category.clone();
            params_values.push((":category",&category_value));
            query.replace("{{where_category}}", "AND category = :category")
        },
    };



    let mut stmt = conn.prepare(query.as_str()).map_err(|err| {err.to_string()})?;

    let tasks_iter = stmt
        .query_map(params_values.as_slice(), |row| {
            let date_str: String = row.get(3)?;
            let due_date = parse_date(&date_str).unwrap();
            Ok(Task {
                id: row.get(0)?,
                status: TaskStatus::from(row.get::<_, usize>(1)?),
                title: row.get(2)?,
                due_date,
                priority: Priority::from(row.get::<_, usize>(4)?),
                category: row.get(5)?,
            })
    });

    let tasks_iter = match tasks_iter{
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

    let mut stmt= conn.prepare(INSERT_TASK)
        .map_err(|err| err.to_string())?;

    let res =  stmt.execute(named_params! {
        "status": &(task.status as usize),
        "title": task.title,
        "due_date": task.due_date.to_rfc3339(),
        "priority": task.priority as usize,
        "category": task.category,
    });

    match res{
        Ok(val) => {
            task.id = val as u64;
            Ok(())
        },
        Err(err) => Err(err.to_string()),
    }
}

pub fn get_pomodoro(ls_args: LSArgs) -> Result<Vec<PomoTask>, String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };
    let mut stmt = match conn.prepare("SELECT * FROM pomodoro") {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let pomo_tasks_iter = match stmt.query_map([], |row| {
        let start_date_str: String = row.get(5)?;
        let start_date = parse_date(&start_date_str).unwrap();

        let end_date_str: String = row.get(6)?;
        let end_date = parse_date(&end_date_str).unwrap();

        let duration_str: String = row.get(3)?;
        let duration = parse_duration(&duration_str).unwrap();

        Ok(PomoTask {
            id: row.get(0)?,
            pomo_type: PomoType::from(row.get::<_, usize>(1)?),
            title: row.get(2)?,
            duration: duration,
            category: row.get(4)?,
            start_time: start_date,
            end_time: end_date,
        })
    }) {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };

    let mut pomo_tasks: Vec<PomoTask> = Vec::new();

    for pomo_task in pomo_tasks_iter {
        pomo_tasks.push(pomo_task.unwrap());
    }

    Ok(pomo_tasks)
}

fn add_pomodoro(pomo_task: PomoTask) -> Result<(), String> {
    let conn = match get_connection() {
        Ok(val) => val,
        Err(err) => return Err(err.to_string()),
    };
    // conn.execute(
    //     r#"INSERT INTO pomodoro (pomo_type, title, start_time, end_time,
    //             duration, completed, category, created_at, updated_at)
    //      VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
    //     params![
    //         pomo_task.pomo_type as usize,
    //         pomo_task.title,
    //         pomo_task.start_time.to_rfc3339(),
    //         pomo_task.end_time.to_rfc3339(),
    //         pomo_task.duration,
    //         pomo_task.completed,
    //         pomo_task.category,
    //         pomo_task.created_at,
    //         pomo_task.updated_at,
    //     ],
    // )
    // .map_err(|err| {
    //     return err.to_string();
    // });

    // Return the ID of the newly inserted task
    Ok(())
}
