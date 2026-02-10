use std::fmt;
use std::num::ParseIntError;
#[derive(Debug)]
pub enum HeapError {
    CorruptData(String),
    //CorruptKey(String),
    FileError(std::io::Error),
    WeightParseError(ParseIntError),
    FileDoesNotExist,
    MissingCommand,
    UnknownCommand(String),
    MissingOption((String, String)),
    ArgumentCannotHaveWhitespace,
    ArgumentCannotHaveSeparator,
    MissingArgument,
    InvalidHeapName,
    InvalidWeights,
    InvalidHeapTaskPair(String),
    DoesNotTakeArg(String),
    TagCannotBeEmpty,
    NoTaggedElements(String),
    TaskNotFound(String),
    TaskAlreadyExists(String),
    NoHeapsFound,
    SomeHeapsAreFinished,
    NoTasksFound,
    IndexError,
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
            CorruptData(str) => {
                write!(f, "Data row is corrupt: {str}")
            }
            //HeapError::CorruptKey(e) => write!(f, "Parsing Error: {}", e),
            FileDoesNotExist => write!(f, "File does not exist"),
            MissingCommand => write!(f, "No command was used"),
            UserSaidNo => write!(f, "You backed down."),
            UnknownCommand(str) => write!(f, "Unknown command: {str}"),
            MissingOption((arg, cmd)) => write!(f, "A {arg} is required for `{cmd}`."),
            DoesNotTakeArg(str) => write!(f, "`{str}` does not take options."),
            ArgumentCannotHaveWhitespace => write!(f, "The argument cannot have whitespace"),
            InvalidHeapName => write!(
                f,
                "A heap must only contain alphanumeric and -_ characters."
            ),
            InvalidWeights => write!(
                f,
                "The sum of weights in heaps or staged tasks cannot be zero"
            ),
            ArgumentCannotHaveSeparator => {
                write!(
                    f,
                    "Arguments cannot contain \"{}\"",
                    crate::utils::SEPARATOR
                )
            }
            InvalidHeapTaskPair(reason) => writeln!(
                f,
                "Not a valid heap.task pair: {reason}.\n Use
                \"heap_name.task_name\""
            ),
            NoTaskInProgress => writeln!(f, "There is no task in progress, use `pop`."),
            MissingArgument => writeln!(f, "Missing argument."),
            TagCannotBeEmpty => writeln!(f, "Tag cannot be empty or contain whitespace."),
            TaskNotFound(name) => writeln!(f, "Task \"{name}\" was not found."),
            TaskAlreadyExists(name) => writeln!(f, "Task \"{name}\" already exists."),
            NoTaggedElements(tag) => writeln!(f, "No elements found where tags {tag} intersect."),
            NoTasksFound => writeln!(f, "No tasks were found."),
            NoHeapsFound => writeln!(f, "No heaps were found."),
            SomeHeapsAreFinished => writeln!(f, "Some heaps only have finished tasks."),
            IndexError => writeln!(f, "Index was out of bounds."),
            HeapNotFound(heap_name) => writeln!(f, "Heap {heap_name} not found."),
            HeapAlreadyExists(heap_name) => writeln!(f, "Heap {heap_name} already exists."),
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
