use core::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::{HeapError, Weight, utils::DEFAULT_WEIGHT};

enum TaskStatus {
    Unstaged,
    Staged,
    Finished,
}
impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Unstaged => write!(f, "0"),
            Staged => write!(f, "1"),
            Finished => write!(f, "2"),
        }
    }
}
use TaskStatus::*;

pub struct Task {
    pub name: String,
    pub description: String,
    pub weight: Weight,
    pub status: TaskStatus,
}
impl Task {
    fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        weight: Weight,
        status: TaskStatus,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
            status,
        }
    }
    pub fn from_name(name: impl Into<String>) -> Self {
        Task::new(name, "...", DEFAULT_WEIGHT, Unstaged)
    }
    pub fn get_name(&self) -> &str {
        &self.name
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
    pub fn set_weight(&mut self, weight_str: impl AsRef<str>) -> &mut Self {
        self.weight = weight_str.as_ref().parse().unwrap_or_else(|err| {
            println!("Could not parse: {err}\nSetting weight to default value:{DEFAULT_WEIGHT}");
            DEFAULT_WEIGHT
        });
        self
    }
    pub fn get_weight(&self) -> Weight {
        self.weight
    }
    pub fn is_finished(&self) -> bool {
        matches!(self.status, Finished)
    }
    pub fn is_staged(&self) -> bool {
        matches!(self.status, Staged)
    }
    pub fn is_unstaged(&self) -> bool {
        matches!(self.status, Unstaged)
    }
    pub fn finish(&mut self) -> &mut Self {
        self.status = Finished;
        self
    }
    pub fn stage(&mut self) -> &mut Self {
        self.status = Staged;
        self
    }
    pub fn unstage(&mut self) -> &mut Self {
        self.status = Unstaged;
        self
    }
}
impl FromStr for Task {
    type Err = HeapError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(',');

        // 1. Name: Strict (Must exist and not be empty)
        let name = parts
            .next()
            .map(|s| s.trim()) // Clean up whitespace
            .filter(|s| !s.is_empty())
            .ok_or(HeapError::CorruptData(s.to_string()))?
            .to_string();

        // 2. Description: Permissive (Defaults to empty)
        let description = parts.next().map(|s| s.trim()).unwrap_or("").to_string();

        // 3. Weight: Strict on Garbage, Permissive on Missing
        // If the field is there ("100") but bad ("100a"), we return Error.
        // If the field is missing entirely, we use Default.
        let weight = match parts.next() {
            Some(val) => val
                .trim()
                .parse()
                .map_err(|_| HeapError::CorruptData(s.to_string()))?,
            None => DEFAULT_WEIGHT,
        };

        // 4. Tags: Collect remaining
        let status = match parts.next().unwrap_or("0").parse().unwrap_or(0) {
            0 => Unstaged,
            1 => Staged,
            2 => Finished,
            _ => {
                return Err(HeapError::CorruptData(s.to_string()));
            }
        };

        Ok(Task::new(name, description, weight, status))
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
            "{},{},{},{}",
            self.name, self.description, self.weight, self.status
        )
    }
}
