use core::fmt;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use crate::HeapError;

use sha2::Digest;

const DEFAULT_WEIGHT: u32 = 100;

pub struct Task {
    name: String,
    description: String,
    weight: u32,
    tags: HashSet<String>,
}
impl Task {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        weight: u32,
        tags: HashSet<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
            tags,
        }
    }
    pub fn from_arg(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: "".to_owned(),
            weight: DEFAULT_WEIGHT,
            tags: HashSet::new(),
        }
    }
    fn get_ctr_arg(&self) -> &str {
        &self.name
    }
    //pub fn to_csv(&self) -> String {
    //    let mut tags = Vec::new();
    //    for tag in &self.tags {
    //        tags.push(tag.to_owned());
    //    }
    //    let tags = tags.join(" ");
    //    format!(
    //        "{},{},{},{}",
    //        self.name, self.description, self.weight, tags
    //    )
    //}
    //pub fn from_csv(csv_line: impl Into<String>) -> Result<Self, HeapError> {
    //    let csv_line = csv_line.into();
    //    let mut parts = csv_line.split(',');

    //    // Use '?' to exit early if a field is missing
    //    let name = parts
    //        .next()
    //        .ok_or_else(|| HeapError::CorruptData("No name found".to_string()))?;
    //    let description = parts.next().unwrap_or_default();
    //    let weight = parts
    //        .next()
    //        .and_then(|w| w.parse().ok())
    //        .unwrap_or(DEFAULT_WEIGHT);

    //    let tags = parts
    //        .next()
    //        .unwrap_or("")
    //        .split_whitespace()
    //        .map(String::from)
    //        .collect();
    //    //let elements: Vec<&str> = csv_line.split(',').collect();
    //    //let [name, description, weight_str, tags_str] = elements.as_slice() else {
    //    //    return Err(HeapError::CorruptData(csv_line));
    //    //};

    //    //let weight: u32 = weight_str.parse().unwrap_or(DEFAULT_WEIGHT);
    //    //let tags = tags_str.split(" ").map(String::from).collect();
    //    //let mut tags: HashSet<String> = HashSet::new();
    //    //for tag in elements[index].split(" ").into_iter() {
    //    //    tags.insert(tag.to_owned());
    //    //}
    //    Ok(Task::new(name, description, weight, tags))
    //}
    pub fn get_hash(&self) -> [u8; 32] {
        sha2::Sha256::digest(self.get_ctr_arg()).into()
    }
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }
    pub fn set_desc(&mut self, desc: String) -> &mut Self {
        self.description = desc;
        self
    }
    pub fn set_weight(&mut self, weight: u32) -> &mut Self {
        self.weight = weight;
        self
    }
    pub fn add_tag(&mut self, tag: String) -> &mut Self {
        self.tags.insert(tag);
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
        let tags = parts
            .next()
            .unwrap_or("")
            .split_whitespace()
            .map(String::from)
            .collect();

        Ok(Task::new(name, description, weight, tags))
    }
}
impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        //let mut tags = Vec::new();
        //for tag in &self.tags {
        //    tags.push(tag.to_owned());
        //}
        //let tags = tags.join(" ");
        let tags: String = self
            .tags
            .iter()
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
            .join(" ");
        write!(
            f,
            "{},{},{},{},",
            self.name, self.description, self.weight, tags
        )
    }
}
