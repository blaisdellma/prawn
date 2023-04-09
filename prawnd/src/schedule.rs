use std::io::{Read,Write};

use anyhow::{bail,Result};
use chrono::{Local,DateTime};
#[allow(unused_imports)]
use tracing::{info,debug,warn,error,trace,Level};

const TIMER_ID_FILE: &'static str = "timer_id";

fn extract_timer_id(stderr_output: &str) -> &str {
    stderr_output
        .split_once("run-")
        .unwrap()
        .1
        .split_once(".timer")
        .unwrap()
        .0
}

// registers systemd timer
pub fn register(event_time: DateTime<Local>) -> Result<()> {
    debug!("registering timer");

    let on_calendar = event_time.format("--on-calendar=%F %T").to_string();
    debug!("timer set for {}",on_calendar);

    let mut command = std::process::Command::new("systemd-run");
    command.arg("--user").arg(on_calendar).arg("prawnd");
    debug!("running timer command: {:?}",command);

    let output = command.output()?;
    if output.status.success() {
        debug!("started timer");
        if output.stdout.len() != 0 {
            debug!("stdout: {}", String::from_utf8(output.stdout)?);
        }
        let stderr = String::from_utf8(output.stderr)?;
        if stderr.len() != 0 {
            debug!("stderr: {}", stderr);
        }
        let timer_id = extract_timer_id(&stderr);
        debug!("timer id: {}", timer_id);
        write_timer_id(timer_id)?;
        Ok(())
    } else {
        error!("timer start failed: {}", output.status);
        error!("stdout: {}", String::from_utf8(output.stdout)?);
        error!("stderr: {}", String::from_utf8(output.stderr)?);
        bail!("failed to start timer");
    }
}

pub fn unregister() -> Result<()> {
    let timer_id = read_timer_id()?;
    match timer_id {
        Some(timer_id) => {
            let mut command = std::process::Command::new("systemctl");
            command.arg("--user").arg("stop").arg(format!("run-{}.timer",timer_id));
            debug!("running stop timer command: {:?}",command);

            let output = command.output()?;
            if output.status.success() {
                debug!("timer unregistered");
                if output.stdout.len() != 0 {
                    debug!("stdout: {}", String::from_utf8(output.stdout)?);
                }
                if output.stderr.len() != 0 {
                    debug!("stderr: {}", String::from_utf8(output.stderr)?);
                }
                erase_timer_id()?;
            } else {
                error!("unregistering timer failed: {}", output.status);
                error!("stdout: {}", String::from_utf8(output.stdout)?);
                error!("stderr: {}", String::from_utf8(output.stderr)?);
                bail!("failed to unregister timer");
            }


        },
        None => {},
    }
    Ok(())
}

pub fn write_timer_id(timer_id: &str) -> Result<()> {
    let filename = crate::get_path(TIMER_ID_FILE)?;
    let mut file = std::fs::File::create(filename)?;
    file.write(timer_id.as_bytes())?;
    Ok(())
}

pub fn erase_timer_id() -> Result<()> {
    let filename = crate::get_path(TIMER_ID_FILE)?;
    std::fs::remove_file(filename)?;
    Ok(())
}

pub fn read_timer_id() -> Result<Option<String>> {
    let filename = crate::get_path(TIMER_ID_FILE)?;
    let mut file = match std::fs::File::open(filename) {
        Ok(x) => x,
        Err(_) => return Ok(None),
    };
    let mut timer_id = String::new();
    file.read_to_string(&mut timer_id)?;
    Ok(Some(timer_id))
}
