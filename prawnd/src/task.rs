use std::io::Write;

use anyhow::Result;
#[allow(unused_imports)]
use chrono::{DateTime,Local,Utc,Duration,serde::ts_seconds};
use serde::{Serialize,Deserialize};

const TASK_STATUS_STR : [&'static str; 4] = ["waiting", "pre-deadline", "post-deadline", "late"];

#[derive(Copy,Clone)]
pub enum TaskStatus {
    Waiting,
    PreDeadline,
    PostDeadline,
    Late,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.pad(TASK_STATUS_STR[*self as usize])?;
        Ok(())
    }
}

#[derive(PartialEq,Serialize,Deserialize)]
pub enum EnableStatus {
    Enabled,
    Disabled,
}

#[derive(Serialize,Deserialize)]
pub struct Task {
    title: String,
    description: String,
    enabled: EnableStatus,
    #[serde(with = "ts_seconds")]
    deadline: DateTime<Utc>,

    // // I would love to just use this
    // // but I have to wait on chrono
    // // to bump the time dependency
    // pre_period: Duration,
    // post_period: Duration,

    pre_period: i64, // seconds
    post_period: i64, // seconds
}

impl Task {
    pub fn get_next_event(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
        match self.enabled {
            EnableStatus::Enabled => {
                match self.get_status(now) {
                    TaskStatus::Waiting => Some(self.deadline - Duration::seconds(self.pre_period)),
                    TaskStatus::PreDeadline => Some(self.deadline),
                    TaskStatus::PostDeadline => Some(self.deadline + Duration::seconds(self.post_period)),
                    TaskStatus::Late => None,
                }
            },
            EnableStatus::Disabled => None,
        }
    }

    pub fn get_status(&self, now: DateTime<Utc>) -> TaskStatus {
        if now <= self.deadline - Duration::seconds(self.pre_period) {
            TaskStatus::Waiting
        } else if now <= self.deadline {
            TaskStatus::PreDeadline
        } else if now <= self.deadline + Duration::seconds(self.post_period) {
            TaskStatus::PostDeadline
        } else {
            TaskStatus::Late
        }
    }

    pub fn enable(&mut self) {
        self.enabled = EnableStatus::Enabled;
    }

    pub fn disable(&mut self) {
        self.enabled = EnableStatus::Disabled;
    }
    
    // pub fn title(&self) -> &str {
    //     &self.title
    // }
    //
    // pub fn description(&self) -> &str {
    //     &self.description
    // }
    //
    // pub fn is_enabled(&self) -> bool {
    //     self.enabled == EnableStatus::Enabled
    // }
    //
    // pub fn deadline(&self) -> DateTime<Utc> {
    //     self.deadline
    // }
    //
    pub fn pre_period(&self) -> Duration {
        Duration::seconds(self.pre_period)
    }
    
    pub fn post_period(&self) -> Duration {
        Duration::seconds(self.post_period)
    }
}

impl std::fmt::Debug for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(&format!("{}\n", self.title))?;
        f.write_str(&format!("{}\n", self.description))?;
        f.write_str(&format!("Deadline: {} (-{},+{})", self.deadline,self.pre_period(),self.post_period()))?;
        Ok(())
    }
}

fn input(label: &str) -> Result<String> {
    let mut input = String::new();
    print!("{}",label);
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    input.pop();
    Ok(input)
}

pub fn get_task_from_stdin() -> Result<Task> {
    let title = input("Title: ")?;
    let description = input("Description: ")?;

    let mut deadline = input("Deadline (YYYY-MM-DD): ")?;
    deadline.push_str("T00:00:00Z");
    let deadline = deadline.parse::<DateTime<Utc>>()?;

    let pre_period = input("Pre: ")?;
    let post_period = input("Post: ")?;

    Ok(Task {
        title,
        description,
        enabled: EnableStatus::Enabled,
        deadline,

        pre_period: pre_period.parse::<i64>()? * 24 * 3600,
        post_period: post_period.parse::<i64>()? * 24 * 3600,
    })
}
