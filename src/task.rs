use core::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::utils::{SEPARATOR, Weight};
use crate::{HeapError, utils::DEFAULT_WEIGHT};

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum TaskStatus {
    Idle,
    Staged,
    InProgress,
    Finished,
}
impl FromStr for TaskStatus {
    type Err = HeapError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.trim() {
            "0" => Ok(Idle),
            "1" => Ok(Staged),
            "2" => Ok(InProgress),
            "3" => Ok(Finished),
            _ => Err(HeapError::CorruptData(input.to_string())),
        }
    }
}
impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Idle => write!(f, "0"),
            Staged => write!(f, "1"),
            InProgress => write!(f, "2"),
            Finished => write!(f, "3"),
        }
    }
}
use TaskStatus::*;

pub struct TaskBuilder {
    name: String,
    heap_name: String,
    description: Option<String>,
    weight: Option<Weight>,
}
impl TaskBuilder {
    pub fn new(name: impl Into<String>, heap_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            heap_name: heap_name.into(),
            description: None,
            weight: None,
        }
    }
    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }
    pub fn weight(&mut self, weight: Weight) -> &mut Self {
        self.weight = Some(weight);
        self
    }
}
pub struct Task {
    name: String,
    description: String,
    heap_name: String,
    weight: Weight,
    status: TaskStatus,
}
impl Task {
    fn new(
        name: impl Into<String>,
        heap_name: impl Into<String>,
        description: impl Into<String>,
        weight: Weight,
        status: TaskStatus,
    ) -> Self {
        Self {
            name: name.into(),
            heap_name: heap_name.into(),
            description: description.into(),
            weight,
            status,
        }
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_heap_name(&self) -> &str {
        &self.heap_name
    }
    pub fn get_full_name(&self) -> String {
        format!("{}.{}", self.heap_name, self.name)
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }
    pub fn set_desc(&mut self, desc: impl Into<String>) -> &mut Self {
        self.description = desc.into();
        self
    }
    pub fn set_weight(&mut self, weight: Weight) -> &mut Self {
        self.weight = weight;
        self
    }
    pub fn get_weight(&self) -> Weight {
        self.weight
    }
    pub fn is_finished(&self) -> bool {
        matches!(self.status, Finished)
    }
    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, InProgress)
    }
    pub fn is_staged(&self) -> bool {
        matches!(self.status, Staged)
    }
    pub fn get_state(&self) -> TaskStatus {
        self.status.clone()
    }
    pub fn finish(&mut self) -> &mut Self {
        self.status = Finished;
        self
    }
    pub fn in_progress(&mut self) -> &mut Self {
        self.status = InProgress;
        self
    }
    pub fn stage(&mut self) -> &mut Self {
        self.status = Staged;
        self
    }
    pub fn unstage(&mut self) -> &mut Self {
        self.status = Idle;
        self
    }
}
impl FromStr for Task {
    type Err = HeapError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.split(SEPARATOR);

        // 1. Name: Strict (Must exist and not be empty)
        let heap_task_raw = parts
            .next()
            .map(|s| s.trim()) // Clean up whitespace
            .filter(|s| !s.is_empty())
            .ok_or_else(|| HeapError::CorruptData(input.to_string()))?;
        let (heap_name, task_name) = heap_task_raw
            .split_once(".")
            .ok_or_else(|| HeapError::CorruptData(input.into()))?;
        // 2. Description: Permissive (Defaults to empty)
        let description = parts.next().map(|temp| temp.trim()).unwrap_or("");

        // 3. Weight: Strict on Garbage, Permissive on Missing
        // If the field is there ("100") but bad ("100a"), we return Error.
        // If the field is missing entirely, we use Default.
        let weight = match parts.next() {
            Some(val) => val
                .trim()
                .parse()
                .map_err(|_| HeapError::CorruptData(input.to_string()))?,
            None => DEFAULT_WEIGHT,
        };

        // 4. Status: Collect remaining
        let status = parts
            .next()
            .unwrap_or("0")
            .parse()
            .map_err(|_| HeapError::CorruptData(input.to_string()))?;

        Ok(Task::new(task_name, heap_name, description, weight, status))
    }
}
impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        //let mut tags = Vec::new();
        //for tag in &self.tags {
        //    tags.push(tag.to_owned());
        //}
        //let tags = tags.join(" ");
        write!(
            f,
            "{}",
            [
                self.get_full_name(),
                self.description.clone(),
                self.weight.to_string(),
                self.status.to_string()
            ]
            .join(SEPARATOR)
        )
    }
}
impl From<TaskBuilder> for Task {
    fn from(builder: TaskBuilder) -> Self {
        Task::new(
            builder.name,
            builder
                .description
                .unwrap_or("Empty description.".to_owned()),
            builder.heap_name,
            builder.weight.unwrap_or(DEFAULT_WEIGHT),
            Idle,
        )
    }
}
