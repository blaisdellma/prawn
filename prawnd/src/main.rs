use anyhow::{bail,Result};

use chrono::{Local,Duration};

#[allow(unused_imports)]
use tracing::{info,debug,warn,error,trace,Level};
use tracing_subscriber as ts;
use tracing_appender as ta;

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

// registers timer
fn register() -> Result<()> {
    debug!("registering timer");
    // 15s in the future for testing
    let deadline = Local::now() + Duration::seconds(15);

    let on_calendar = deadline.format("--on-calendar=%F %T").to_string();
    debug!("timer set for {}",on_calendar);

    let mut command = std::process::Command::new("systemd-run");
    command.arg("--user").arg(on_calendar).arg("prawnd");
    debug!("running timer command: {:?}",command);

    let output = command.output()?;
    if output.status.success() {
        debug!("started timer:");
        debug!("stdout: {}", String::from_utf8(output.stdout)?);
        debug!("stderr: {}", String::from_utf8(output.stderr)?);
        Ok(())
    } else {
        error!("timer start failed: {}", output.status);
        error!("stdout: {}", String::from_utf8(output.stdout)?);
        error!("stderr: {}", String::from_utf8(output.stderr)?);
        bail!("failed to start timer");
    }

}

// unregisters timer
fn unregister() {
    todo!();
}

fn print_usage() {
    println!(
r#"USAGE: prawnd COMMAND

commands:
    register : registers systemd timer
    unregister : unregisters systemd timer"#);
}

fn main() {
    let _guard = init_log().expect("Error during log init");
    match match std::env::args().nth(1) {
        Some(command) if command == "register" => {
            debug!("Called with register");
            register()
        },
        Some(command) if command == "unregister" => {
            debug!("Called with unregister");
            unregister();
            Ok(())
        },
        Some(command) => {
            debug!("Called with some other command: {}", command);
            println!("Command not recognized");
            print_usage();
            Ok(())
        },
        None => {
            debug!("Called with no argument");
            print_usage();
            Ok(())
        },
    } {
        Ok(_) => {},
        Err(e) => {
            eprintln!("ERROR: {}", e);
            error!("Exiting with error {}", e);
        },
    }

}
