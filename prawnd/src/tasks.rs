use std::collections::HashMap;
use std::io::{BufReader,BufWriter};

use anyhow::Result;

use chrono::{DateTime,Utc};

use serde::{Serialize,Deserialize};

use crate::task::*;

const TASKS_FILE : &'static str = "/home/lethe/.local/prawn/tasks";

#[derive(Serialize,Deserialize)]
pub struct Tasks {
    tasks: HashMap<i64,Task>,
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

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(task.id(),task);
    }

    pub fn update_task(&mut self, task_new: Task) {
        if let Some(task) = self.tasks.get_mut(&task_new.id()) {
            *task = task_new;
        }
    }

    pub fn update_all(&self) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        self.tasks.values().filter_map(|task| task.update(now)).min()
    }

    pub fn list_all(&self) {
        for task in self.tasks.values() {
            println!("{:?}",task);
        }
    }
}
