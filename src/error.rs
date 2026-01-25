use std::fmt;
#[derive(Debug)]
pub enum HeapError {
    CorruptData(String),
    //CorruptKey(String),
    FileError(std::io::Error),
    FileDoesNotExist,
    RequiresTask(String),
    TagCannotBeEmpty,
}
impl fmt::Display for HeapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use HeapError::*;
        match self {
            FileError(e) => write!(f, "File Input Error: {}", e),
            CorruptData(str) => {
                write!(f, "Data row is corrupt:{str}")
            }
            //HeapError::CorruptKey(e) => write!(f, "Parsing Error: {}", e),
            FileDoesNotExist => write!(f, "File does not exist."),
            RequiresTask(str) => write!(f, "A task is required to define a {str}."),
            TagCannotBeEmpty => writeln!(f, "Tag cannot be empty or whitespace"),
        }
    }
}
impl From<std::io::Error> for HeapError {
    fn from(err: std::io::Error) -> HeapError {
        HeapError::FileError(err)
    }
}
