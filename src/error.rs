use std::fmt;
#[derive(Debug)]
pub enum HeapError {
    CorruptData(String),
    //CorruptKey(String),
    FileError(std::io::Error),
    FileDoesNotExist,
    MissingArgument((String, String)),
    DoesNotTakeArg(String),
    TagCannotBeEmpty,
    NoTaggedElements(String),
    TaskNotFound(String),
    TaskAlreadyExists(String),
    NoTasksOnHeap,
}
impl fmt::Display for HeapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HeapError::*;
        match self {
            FileError(e) => write!(f, "File Input Error: {}", e),
            CorruptData(str) => {
                write!(f, "Data row is corrupt: {str}")
            }
            //HeapError::CorruptKey(e) => write!(f, "Parsing Error: {}", e),
            FileDoesNotExist => write!(f, "File does not exist"),
            MissingArgument((arg, cmd)) => write!(f, "A {arg} is required for --{cmd}."),
            DoesNotTakeArg(str) => write!(f, "--{str} does not take arguments."),
            TagCannotBeEmpty => writeln!(f, "Tag cannot be empty or contain whitespace."),
            TaskNotFound(name) => writeln!(f, "Task \"{name}\" was not found."),
            TaskAlreadyExists(name) => writeln!(f, "Task \"{name}\" already exists."),
            NoTaggedElements(tag) => writeln!(f, "No elements found where tags {tag} intersect."),
            NoTasksOnHeap => writeln!(f, "No tasks found in the heap."),
        }
    }
}
impl From<std::io::Error> for HeapError {
    fn from(err: std::io::Error) -> HeapError {
        HeapError::FileError(err)
    }
}
