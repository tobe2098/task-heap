use crate::{HeapError, Task};
use directories::ProjectDirs;
use std::{
    collections::HashMap,
    env, fs,
    io::{self, BufRead, BufReader, Write, stdin, stdout},
    path::PathBuf,
    str::FromStr,
};
use terminal_size::{Width, terminal_size};
use textwrap::wrap;
fn get_db_path() -> PathBuf {
    match env::var("TASK_HEAP_DBPATH") {
        Ok(path) => PathBuf::from_str(&path).unwrap().join("./db.csv"),
        Err(_) => {
            if let Some(proj_dirs) = ProjectDirs::from("com", "tobe", "task-heap") {
                // 2. Get the specific data directory (e.g., AppData/Roaming/task-heap)
                let data_dir = proj_dirs.data_dir();

                // 3. Create the directory if it doesn't exist (Crucial for first run!)
                if !data_dir.exists() {
                    fs::create_dir_all(data_dir).expect("Could not create data directory");
                }

                // 4. Append your filename
                data_dir.join("./db.csv")
            } else {
                PathBuf::from("./db.csv")
            }
        }
    }
}
pub fn write_task_heap(heap: HashMap<[u8; 32], Task>) -> std::io::Result<()> {
    let db_path = get_db_path();
    let db_file: fs::File = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(db_path)?;
    for (_, task) in heap {
        match writeln!(&db_file, "{}", task) {
            Ok(()) => (),
            Err(e) => {
                println!("File write error: {e}")
            }
        }
    }
    Ok(())
}
pub fn read_task_heap() -> Result<HashMap<[u8; 32], Task>, HeapError> {
    // Your code here
    let db_path = get_db_path();
    if db_path.exists() {
        let file: fs::File = fs::OpenOptions::new().read(true).open(db_path)?;
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
pub fn print_task_table(tasks: &Vec<&Task>) {
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    const W_WEIGHT: usize = 6;
    const W_TAGS: usize = 20;
    const BORDER_OVERHEAD: usize = 13;

    let remaining_width = term_width.saturating_sub(W_WEIGHT + W_TAGS + BORDER_OVERHEAD);
    const RATIO_NAME_DESC: f64 = 0.3;
    let w_name = (remaining_width as f64 * RATIO_NAME_DESC) as usize;
    let w_description = (remaining_width as f64 * (1. - RATIO_NAME_DESC)) as usize;
    let w_name = w_name.max(5);
    let w_description = w_description.max(10);
    println!(
        "{:<n$} | {:<d$} | {:>w$} | {:<t$}",
        "NAME",
        "DESCRIPTION",
        "WEIGHT",
        "TAGS",
        n = w_name,
        d = w_description,
        w = W_WEIGHT,
        t = W_TAGS
    );
    println!("{}", "-".repeat(term_width));
    for task in tasks {
        // Tags need to be sorted to look consistent (HashSet is random!)
        let mut tags = task.get_tags();
        tags.sort();
        let tags_string = tags
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join(" ");
        let tag_lines = wrap(&tags_string, W_TAGS);
        let name_lines = wrap(task.get_name(), w_name);
        let desc_lines = wrap(task.get_description(), w_description);
        let max_lines = name_lines.len().max(desc_lines.len()).max(tag_lines.len());
        for i in 0..max_lines {
            let name_part = name_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let desc_part = desc_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let tags_part = tag_lines.get(i).map(|s| s.as_ref()).unwrap_or("");

            // Only print Weight/Tags on the FIRST line of the row
            let weight_part = if i == 0 {
                truncate(&task.get_weight().to_string(), W_WEIGHT)
            } else {
                "".to_owned()
            };

            println!(
                "{:<n$} | {:<d$} | {:>w$} | {:<t$}",
                name_part,
                desc_part,
                weight_part,
                tags_part,
                n = w_name,
                d = w_description,
                w = W_WEIGHT,
                t = W_TAGS
            );
        }
        println!("{}", "-".repeat(term_width));
    }
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}..", &s[..max_width - 2])
    } else {
        s.to_string()
    }
}
pub fn print_single_task(task: &Task) {
    print_task_table(&vec![task; 1]);
}
pub fn get_yes_no() -> Result<String, HeapError> {
    print!("[y/n]: ");
    stdout().flush().unwrap(); //Flush so prompt appears before user input.

    let mut input = String::new();

    match stdin().read_line(&mut input) {
        Ok(_) => {
            input = input.trim().to_owned();
            Ok(input)
        }
        Err(e) => Err(HeapError::FileError(e)),
    }
}
