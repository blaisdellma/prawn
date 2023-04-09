use std::io::Read;

use anyhow::{anyhow,bail,Result};
use chrono::{Local,Duration};
#[allow(unused_imports)]
use tracing::{info,debug,warn,error,trace,Level};
use tracing_subscriber as ts;
use tracing_appender as ta;

mod task;
mod tasks;
mod schedule;

pub fn get_path(tail: &str) -> Result<std::path::PathBuf> {
    match std::env::var("HOME") {
        Ok(s) => {
            let mut path: std::path::PathBuf = s.into();
            path.push(".local/prawn/");
            path.push(tail);
            Ok(path)
        },
        Err(_) => bail!("$HOME not set"),
    }
}

fn init_log() -> Result<ta::non_blocking::WorkerGuard> {
    let log_path = get_path("log")?;
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(&log_path)?;

    let prefix = "testing.log";
    let (file, guard) = ta::non_blocking(ta::rolling::never(log_path,prefix));
    ts::fmt()
        .with_writer(file)
        .with_max_level(Level::DEBUG)
        .init();

    debug!("Log init successful");
    Ok(guard)
}

fn print_usage() {
    println!(
r#"USAGE: prawnd COMMAND [ARGS ..]

commands:
    digest     : prints out task digest
    register   : registers systemd timer
    unregister : unregisters systemd timer
    status     : displays status
    init       : initialize task storage
    add        : add task to list - pipe in JSON or use interactively
    modify X   : modify task with id X - pipe in JSON or use interactively
    enable X   : enable task with id X
    disable X  : disable task with id X
    complete X : complete task with id X and set new deadline
    update     : updates all tasks and registers systemd timer for next event"#);
}

fn run() -> Result<()> {
    match std::env::args().nth(1) {
        Some(command) if command == "register" => {
            debug!("Called with register");
            schedule::register(Local::now() + Duration::seconds(15))?
        },
        Some(command) if command == "unregister" => {
            debug!("Called with unregister");
            schedule::unregister()?
        },
        Some(command) if command == "init" => {
            debug!("Called with init");
            let tasks = tasks::Tasks::new();
            tasks.write()?;
        },
        Some(command) if command == "status" => {
            debug!("Called with status");
            if let Some(tasks) = tasks::Tasks::read()? {
                println!("Found {} tasks",tasks.len());
                tasks.list_all();
            } else {
                println!("No tasks found.");
            }
            match schedule::read_timer_id()? {
                Some(timer_id) => {
                    println!("Timer id: {}", timer_id);
                },
                None => {
                    println!("No timer id found");
                },
            }
        },
        Some(command) if command == "digest" => {
            debug!("Called with digest");
            if let Some(tasks) = tasks::Tasks::read()? {
                debug!("Found {} tasks",tasks.len());
                tasks.print_digest();
            } else {
                println!("No tasks found.");
            }
        },
        Some(command) if command == "add" => {
            debug!("Called with add");
            let mut tasks = match tasks::Tasks::read()? {
                Some(x) => x,
                None => tasks::Tasks::new(),
            };
            let task = if atty::is(atty::Stream::Stdin) {
                task::get_task_from_stdin()?
            } else {
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                serde_json::from_str(&input)?
            };
            let uuid = tasks.add_task(task);
            debug!("Added task {}", uuid);
            tasks.write()?;
        },
        Some(command) if command == "modify" => {
            debug!("Called with modify");
            let mut tasks = match tasks::Tasks::read()? {
                Some(x) => x,
                None => {
                    warn!("No tasks found");
                    return Ok(());
                },
            };
            let uuid: usize = std::env::args()
                .nth(2)
                .ok_or(anyhow!("Not enough arguments"))?
                .parse()?;
            let task = if atty::is(atty::Stream::Stdin) {
                task::get_task_from_stdin()?
            } else {
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                serde_json::from_str(&input)?
            };
            tasks.modify_task(uuid,task);
            debug!("Modified task {}", uuid);
            tasks.write()?;
        },
        Some(command) if command == "enable" => {
            debug!("Called with enable");
            let mut tasks = match tasks::Tasks::read()? {
                Some(x) => x,
                None => {
                    warn!("No tasks found");
                    return Ok(());
                },
            };
            let uuid: usize = std::env::args()
                .nth(2)
                .ok_or(anyhow!("Not enough arguments"))?
                .parse()?;
            tasks.enable_task(uuid);
            debug!("Enabled task {}", uuid);
            tasks.write()?;
        },
        Some(command) if command == "disable" => {
            debug!("Called with disable");
            let mut tasks = match tasks::Tasks::read()? {
                Some(x) => x,
                None => {
                    warn!("No tasks found");
                    return Ok(());
                },
            };
            let uuid: usize = std::env::args()
                .nth(2)
                .ok_or(anyhow!("Not enough arguments"))?
                .parse()?;
            tasks.disable_task(uuid);
            debug!("Disabled task {}", uuid);
            tasks.write()?;
        },
        Some(command) if command == "complete" => {
            debug!("Called with complete");
            let mut tasks = match tasks::Tasks::read()? {
                Some(x) => x,
                None => {
                    warn!("No tasks found");
                    return Ok(());
                },
            };
            let uuid: usize = std::env::args()
                .nth(2)
                .ok_or(anyhow!("Not enough arguments"))?
                .parse()?;
            tasks.complete_task(uuid);
            debug!("Completed task {}", uuid);
            tasks.write()?;
        },
        Some(command) if command == "update" => {
            debug!("Called with update");
            if let Some(tasks) = tasks::Tasks::read()? {
                match tasks.update_all() {
                    Some(next_event) => schedule::register(next_event.with_timezone(&Local))?,
                    None => {},
                }
                tasks.write()?;
            }
        },
        Some(command) => {
            debug!("Called with some other command: {}", command);
            println!("Command not recognized");
            print_usage();
        },
        None => {
            debug!("Called with no argument");
            print_usage();
        },
    }
    Ok(())
}

fn main() {
    let _guard = init_log().expect("Error during log init");
    match run() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("ERROR: {}", e);
            error!("Exiting with error {}", e);
        },
    }
}
