use crate::error::HeapError;
use crate::task::{Task, TaskStatus};
use crate::utils::{self, DEFAULT_WEIGHT, NumOrStr, Weight};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::str::FromStr;

pub struct HeapBuilder {
    name: String,
    weight: Option<Weight>,
    tags: HashSet<String>,
}
impl HeapBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            weight: None,
            tags: HashSet::new(),
        }
    }
    pub fn weight(&mut self, weight: Weight) -> &mut Self {
        self.weight = Some(weight);
        self
    }
    pub fn add_tag(&mut self, tag: impl Into<String>) -> &mut Self {
        self.tags.insert(tag.into());
        self
    }
}

pub struct TaskHeap {
    name: String,
    weight: Weight,
    tags: HashSet<String>,
    tasks: VecDeque<Task>,
}

impl TaskHeap {
    pub fn new(name: impl Into<String>, weight: Weight, tags: HashSet<String>) -> Self {
        Self {
            name: name.into(),
            weight,
            tags,
            tasks: VecDeque::new(),
        }
    }
    pub fn push(&mut self, task: Task) {
        self.tasks.push_back(task)
    }
    pub fn insert_task(&mut self, task: Task, index: usize) {
        self.tasks.insert(index, task)
    }
    pub fn remove_task(&mut self, index: usize) -> bool {
        self.tasks.remove(index).is_some()
    }
    pub fn get_task(&self, name_or_idx: &NumOrStr) -> Option<(usize, &Task)> {
        match name_or_idx {
            NumOrStr::Num(idx) => self.tasks.get(*idx).map(|val| (*idx, val)),
            NumOrStr::Str(name) => self
                .tasks
                .iter()
                .enumerate()
                .find(|pair| pair.1.get_name() == name),
        }
    }
    pub fn get_task_mut(&mut self, name_or_idx: &NumOrStr) -> Option<(usize, &mut Task)> {
        match name_or_idx {
            NumOrStr::Num(idx) => self.tasks.get_mut(*idx).map(|val| (*idx, val)),
            NumOrStr::Str(name) => self
                .tasks
                .iter_mut()
                .enumerate()
                .find(|pair| pair.1.get_name() == name),
        }
    }
    pub fn get_first_unfinished_task(&self) -> Option<usize> {
        for task in self.tasks.iter().enumerate() {
            if !task.1.is_finished() {
                return Some(task.0);
            }
        }
        None
    }
    pub fn get_staged(&self) -> (Vec<Weight>, Vec<usize>) {
        self.tasks
            .iter()
            .enumerate()
            .filter_map(|task| {
                if task.1.is_staged() {
                    return Some((task.1.get_weight(), task.0));
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().collect()
    }
    pub fn get_staged_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| task.is_staged()).collect()
    }
    pub fn get_current_tasks(&self) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|task| task.is_in_progress())
            .collect()
    }
    pub fn get_states_sum(&self) -> HashMap<TaskStatus, u32> {
        let mut hashmap = HashMap::new();
        for task in &self.tasks {
            *hashmap
                .entry(task.get_state())
                .and_modify(|counter| *counter += 1)
                .or_insert(1);
        }
        hashmap
    }
    pub fn set_done(&mut self, name_or_idx: &NumOrStr) -> Result<(), HeapError> {
        self.get_task_mut(name_or_idx)
            .ok_or(HeapError::IndexError)?
            .1
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
    pub fn clear_done(&mut self) {
        self.tasks.retain(|task| !task.is_finished());
    }
    pub fn clear_all(&mut self) {
        self.tasks.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
    pub fn is_all_complete(&self) -> bool {
        self.tasks.iter().all(|task| task.is_finished())
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
    pub fn add_tag(&mut self, tag: impl Into<String>) -> bool {
        self.tags.insert(tag.into())
    }
    pub fn remove_tag(&mut self, tag: impl AsRef<str>) -> bool {
        self.tags.remove(tag.as_ref())
    }
}
impl fmt::Display for TaskHeap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tasks_str = self
            .tasks
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("\n");
        let tags_str = self
            .tags
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>()
            .join(",");
        write!(
            f,
            "{}\n{}",
            [self.name.clone(), self.weight.to_string(), tags_str].join(utils::SEPARATOR),
            tasks_str
        )
    }
}
impl FromStr for TaskHeap {
    type Err = HeapError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        //The first line is the header, the rest are tasks.
        //The heap name must be set according to filename externally
        let mut lines = input.lines().into_iter();
        let header = lines
            .next()
            .ok_or_else(|| HeapError::CorruptData(input.to_owned()))?;
        let mut header_iter = header.split(utils::SEPARATOR).into_iter();
        let name = header_iter
            .next()
            .ok_or_else(|| HeapError::CorruptData(input.to_owned()))?
            .trim();
        let weight: Weight = header_iter
            .next()
            .map(|w| {
                w.trim()
                    .parse()
                    .map_err(|_| HeapError::CorruptData(input.to_string()))
            })
            .transpose()?
            .unwrap_or(DEFAULT_WEIGHT);
        let tags = header_iter
            .next()
            .ok_or_else(|| HeapError::CorruptData(input.to_owned()))?
            .split(",")
            .map(|tag| tag.trim().to_owned())
            .collect::<HashSet<String>>();

        let mut heap = TaskHeap::new(name, weight, tags);
        for task in lines.filter(|l| !l.trim().is_empty()).map(|l| l.parse()) {
            heap.push(task?);
        }
        Ok(heap)
    }
}
impl From<HeapBuilder> for TaskHeap {
    fn from(builder: HeapBuilder) -> Self {
        TaskHeap {
            name: builder.name,
            weight: builder.weight.unwrap_or(DEFAULT_WEIGHT),
            tags: builder.tags,
            tasks: VecDeque::new(),
        }
    }
}
