use crate::{HeapError, Task};
use std::{
    collections::HashMap,
    env, fs,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
    str::FromStr,
};
fn get_db_path() -> PathBuf {
    let db_path: String = env::var("TASK_HEAP_DBPATH").unwrap_or_else(|_| {
        let home_path: String = env::var("HOME").unwrap();
        format!("{home_path}/.local/share/task-heap/heap.csv")
    });
    PathBuf::from_str("./db.csv").unwrap()
    //PathBuf::from_str(&db_path).unwrap()
}

pub fn write_task_heap(heap: HashMap<[u8; 32], Task>) -> std::io::Result<()> {
    let db_path = get_db_path();
    let db_file: fs::File = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(db_path)?;
    for (_, task) in heap {
        match writeln!(&db_file, "{}", task.to_string()) {
            Ok(()) => (),
            Err(e) => {
                println!("File write error:{e}")
            }
        }
    }
    Ok(())
}
pub fn read_task_heap() -> Result<HashMap<[u8; 32], Task>, HeapError> {
    // Your code here
    let db_path = get_db_path();
    if db_path.exists() {
        let file: fs::File = fs::OpenOptions::new().open(db_path)?;
        let reader = BufReader::new(file);
        let mut heap = HashMap::new();
        for line in reader.lines() {
            let csv_line = match line {
                Ok(line) => line,
                Err(e) => {
                    return Err(HeapError::FileError(e));
                }
            };
            let new_task: Task = csv_line.parse()?;
            heap.insert(new_task.get_hash(), new_task);
        }
        Ok(heap)
    } else {
        Err(HeapError::FileDoesNotExist)
    }
}
