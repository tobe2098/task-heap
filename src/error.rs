use std::fmt;
use std::num::ParseIntError;
#[derive(Debug)]
pub enum HeapError {
    CorruptData(String),
    //CorruptKey(String),
    FileError(std::io::Error),
    WeightParseError(ParseIntError),
    MissingCommand,
    UnknownCommand(String),
    MissingOption((String, String)),
    ArgumentCannotHaveWhitespace,
    ArgumentCannotHaveSeparator,
    InvalidHeapName,
    InvalidWeights,
    TaskNotFound(String),
    TaskAlreadyExists(String),
    NoHeapsFound,
    //SomeHeapsAreFinished,
    IndexError,
    FormattingError(fmt::Error),
    UserSaidNo,
    HeapNotFound(String),
    HeapAlreadyExists(String),
    ATaskIsAlreadyInProgress(String),
    NoTaskInProgress,
}
impl fmt::Display for HeapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HeapError::*;
        match self {
            FileError(e) => write!(f, "File Input Error: {}", e),
            WeightParseError(e) => write!(f, "Weight parsing error: {}", e),
            FormattingError(e) => write!(f, "Formatting error: {}", e),
            CorruptData(str) => {
                write!(f, "Data row is corrupt: {str}")
            }
            //HeapError::CorruptKey(e) => write!(f, "Parsing Error: {}", e),
            MissingCommand => write!(f, "No command was used."),
            UserSaidNo => write!(f, "You backed down."),
            UnknownCommand(str) => write!(f, "Unknown command: {str}"),
            MissingOption((arg, cmd)) => write!(f, "A {arg} is required for `{cmd}`."),
            ArgumentCannotHaveWhitespace => write!(f, "The argument cannot have whitespace"),
            InvalidHeapName => write!(
                f,
                "A task stack must only contain alphanumeric and -_ characters."
            ),
            InvalidWeights => write!(
                f,
                "The sum of weights in stacks or staged tasks cannot be zero"
            ),
            ArgumentCannotHaveSeparator => {
                write!(
                    f,
                    "Arguments cannot contain \"{}\"",
                    crate::utils::SEPARATOR
                )
            }
            NoTaskInProgress => writeln!(f, "There is no task in progress, use `pop`."),
            TaskNotFound(name) => writeln!(f, "Task \"{name}\" was not found."),
            TaskAlreadyExists(name) => writeln!(f, "Task \"{name}\" already exists."),
            NoHeapsFound => writeln!(f, "No task stacks were found."),
            //SomeHeapsAreFinished => writeln!(f, "Some heaps only have finished tasks."),
            IndexError => writeln!(f, "Index was out of bounds."),
            HeapNotFound(heap_name) => writeln!(f, "Task stack {heap_name} not found."),
            HeapAlreadyExists(heap_name) => writeln!(f, "Stack {heap_name} already exists."),
            ATaskIsAlreadyInProgress(task_name) => {
                writeln!(
                    f,
                    "Task {task_name} is already in progress, only one task can be in
progress at a time."
                )
            }
        }
    }
}
impl From<std::io::Error> for HeapError {
    fn from(err: std::io::Error) -> HeapError {
        HeapError::FileError(err)
    }
}
impl From<std::num::ParseIntError> for HeapError {
    fn from(err: std::num::ParseIntError) -> HeapError {
        HeapError::WeightParseError(err)
    }
}
impl From<std::fmt::Error> for HeapError {
    fn from(err: std::fmt::Error) -> HeapError {
        HeapError::FormattingError(err)
    }
}
