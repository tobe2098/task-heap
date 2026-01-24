use std::fmt;
#[derive(Debug)]
pub enum HeapError {
    CorruptData(String),
    //CorruptKey(String),
    FileError(std::io::Error),
    FileDoesNotExist,
    RequiresTask(String),
    DescriptionError(String),
    WeightError(String),
    PushError(String),
}
impl fmt::Display for HeapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HeapError::FileError(e) => write!(f, "File Input Error: {}", e),
            HeapError::CorruptData(str) => {
                write!(f, "Data row is corrupt:{str}")
            }
            //HeapError::CorruptKey(e) => write!(f, "Parsing Error: {}", e),
            HeapError::FileDoesNotExist => write!(f, "File does not exist."),
            HeapError::RequiresTask(str) => write!(f, "A task is required to define a {str}."),
            HeapError::PushError(str) => write!(f, "--push: {str}."),
            HeapError::DescriptionError(str) => write!(f, "--description: {str}"),
            HeapError::WeightError(str) => {
                write!(f, "--weight: {str}")
            }
        }
    }
}
impl From<std::io::Error> for HeapError {
    fn from(err: std::io::Error) -> HeapError {
        HeapError::FileError(err)
    }
}
