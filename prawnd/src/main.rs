use anyhow::Result;

use chrono::{Local,Duration,DateTime};

#[allow(unused_imports)]
use tracing::{info,debug,warn,error,trace,Level};
use tracing_subscriber as ts;
use tracing_appender as ta;

mod task;
mod tasks;
mod schedule;

const LOG_PATH: &'static str = "/home/lethe/.local/prawn/log";

fn init_log() -> Result<ta::non_blocking::WorkerGuard> {
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(LOG_PATH)?;

    let prefix = "testing.log";
    let (file, guard) = ta::non_blocking(ta::rolling::never(LOG_PATH,prefix));
    ts::fmt()
        .with_writer(file)
        .with_max_level(Level::DEBUG)
        .init();

    debug!("Log init successful");
    Ok(guard)
}

fn print_usage() {
    println!(
r#"USAGE: prawnd COMMAND

commands:
    register : registers systemd timer
    unregister : unregisters systemd timer
    status : displays status"#);
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
        },
        Some(command) if command == "add" => {
            debug!("Called with add");
            let mut tasks = match tasks::Tasks::read()? {
                Some(x) => x,
                None => tasks::Tasks::new(),
            };
            let uuid = 0;
            let task = task::gen_task(uuid)?;
            tasks.add_task(task);
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
