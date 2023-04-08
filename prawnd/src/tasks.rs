use std::collections::HashMap;
use std::io::{BufReader,BufWriter};

use anyhow::Result;

use chrono::{DateTime,Utc};

use serde::{Serialize,Deserialize};

use crate::task::*;

const TASKS_FILE : &'static str = "/home/lethe/.local/prawn/tasks";

#[derive(Serialize,Deserialize)]
pub struct Tasks {
    tasks: HashMap<usize,Task>,
}

impl Tasks {
    pub fn new() -> Self {
        Tasks {
            tasks: HashMap::new(),
        }
    }

    pub fn read() -> Result<Option<Self>> {
        if let Ok(file) = std::fs::File::open(TASKS_FILE) {
            let reader = BufReader::new(file);
            let tasks = serde_json::from_reader(reader)?;
            Ok(Some(tasks))
        } else {
            Ok(None)
        }
    }

    pub fn write(&self) -> Result<()> {
        let file = std::fs::File::create(TASKS_FILE)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer,self)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn add_task(&mut self, task: Task) -> usize {
        let uuid = self.tasks.keys().max().map(|x| x + 1).unwrap_or(0);
        self.tasks.insert(uuid,task);
        uuid
    }

    pub fn modify_task(&mut self, uuid: usize, task_new: Task) {
        if let Some(task) = self.tasks.get_mut(&uuid) {
            *task = task_new;
        }
    }

    pub fn enable_task(&mut self, uuid: usize) {
        if let Some(task) = self.tasks.get_mut(&uuid) {
            task.enable();
        }
    }

    pub fn disable_task(&mut self, uuid: usize) {
        if let Some(task) = self.tasks.get_mut(&uuid) {
            task.disable();
        }
    }

    pub fn update_all(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        self.tasks.values().filter_map(|task| task.get_next_event(now)).min()
    }

    pub fn list_all(&self) {
        let now = Utc::now();
        for task in self.tasks.values() {
            println!("{:?}",task);
            println!("Status: {}", task.get_status(now));
            println!("");
        }
    }
}
