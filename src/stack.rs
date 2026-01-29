use crate::error::HeapError;
use crate::task::Task;
use crate::utils::Weight;
use std::collections::{HashSet, VecDeque};
use std::fmt;
use std::str::FromStr;

pub struct TaskStack {
    name: String,
    description: String,
    weight: Weight,
    tags: HashSet<String>,
    tasks: VecDeque<Task>,
}

impl TaskStack {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        weight: Weight,
        tags: HashSet<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
            tags,
            tasks: VecDeque::new(),
        }
    }
    pub fn push(&mut self, task: Task) {
        self.tasks.push_back(task)
    }
    pub fn insert(&mut self, task: Task, index: usize) {
        self.tasks.insert(index, task)
    }
    pub fn delete(&mut self, index: usize) -> bool {
        self.tasks.remove(index).is_some()
    }
    pub fn set_done(&mut self, index: usize) -> Result<(), HeapError> {
        self.tasks
            .get_mut(index)
            .ok_or(HeapError::IndexError("set_done".to_owned()))?
            .finish();
        if self.tasks.iter().all(|task| !task.is_staged()) {
            for task in &mut self.tasks {
                if task.is_unstaged() {
                    task.stage();
                    break;
                }
            }
        }
        Ok(())
    }
    pub fn get_staged_weights(&self) -> Vec<Weight> {
        self.tasks.iter().map(|task| task.get_weight()).collect()
    }
    pub fn clear_done(&mut self) {
        self.tasks.retain(|task| !task.is_finished());
    }
    pub fn clear_all(&mut self) {
        self.tasks.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }
    pub fn get_description(&self) -> &str {
        &self.description
    }
    pub fn set_description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = description.into();
        self
    }
    pub fn get_weight(&self) -> Weight {
        self.weight
    }
    pub fn set_weight(&mut self, weight: Weight) -> &mut Self {
        self.weight = weight;
        self
    }
    pub fn has_tags(&self, tags: &[String]) -> bool {
        tags.iter().all(|tag| self.tags.contains(tag))
    }
    pub fn get_tags(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_ref()).collect()
    }
}
impl fmt::Display for TaskStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TaskStack {{ name: {}, description: {}, weight: {}, tags: {:?}, tasks: {} }}",
            self.name,
            self.description,
            self.weight,
            self.tags,
            self.tasks.len()
        )
    }
}
impl FromStr for TaskStack {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Implement parsing logic here if needed
        Err("Not implemented".to_string())
    }
}
