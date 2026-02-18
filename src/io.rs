use crate::{
    error::HeapError,
    heap::TaskHeap,
    task::{Task, TaskStatus},
    utils::{HeapMap, TaskID, check_task_id},
};
use directories::ProjectDirs;
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{BufRead, BufReader, Write, stdin, stdout},
    path::PathBuf,
    str::FromStr,
};
use terminal_size::{Width, terminal_size};
use textwrap::wrap;

const EXTENSION: &str = "dbsv";
const VOID: &str = "$VOID$";

fn get_db_path() -> PathBuf {
    match env::var("TASK_HEAP_DBPATH") {
        Ok(path) => PathBuf::from_str(&path).unwrap(),
        Err(_) => {
            if let Some(proj_dirs) = ProjectDirs::from("com", "tobe", "task-heap") {
                // 2. Get the specific data directory (e.g., AppData/Roaming/task-heap)
                let data_dir = proj_dirs.data_dir();

                // 3. Create the directory if it doesn't exist (Crucial for first run!)
                if !data_dir.exists() {
                    fs::create_dir_all(data_dir).expect("Could not create data directory");
                }

                // 4. Append your filename
                PathBuf::from(data_dir)
            } else {
                PathBuf::from(".")
            }
        }
    }
}
pub fn delete_stack_file(stack_name: &str) -> std::io::Result<()> {
    let db_path = get_db_path();
    let file_path = db_path.join(format!("{}.{}", stack_name, EXTENSION));
    if file_path.exists() {
        fs::remove_file(&file_path)?;
    }
    Ok(())
}
pub fn write_meta_file(active_task: Option<TaskID>) -> std::io::Result<()> {
    let db_path = get_db_path();
    let meta_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(db_path.join(format!("meta.{}", EXTENSION)))?;
    if let Some(active_task) = active_task {
        writeln!(&meta_file, "{}.{}", active_task.0, active_task.1)?;
    } else {
        writeln!(&meta_file, "{}", VOID)?;
    }
    Ok(())
}
pub fn write_task_heap(heapmap: HeapMap) -> std::io::Result<()> {
    let db_path = get_db_path();
    for heap in heapmap.values() {
        let db_file: fs::File = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(db_path.join(format!("{}.{}", heap.get_name(), EXTENSION)))?;
        writeln!(&db_file, "{}", heap)?;
    }
    Ok(())
}
pub fn read_task_heap() -> Result<HeapMap, HeapError> {
    // Your code here
    let db_path = get_db_path();
    let mut heapmap = HeapMap::new();
    for file in fs::read_dir(db_path)?.filter(|name| {
        name.as_ref()
            .is_ok_and(|v| v.file_name() != OsStr::new(&format!("meta.{}", EXTENSION)))
    }) {
        let file = file?;
        let content = fs::read_to_string(file.path())?;
        if content.trim().is_empty() {
            continue; // Skip empty files
        }
        let mut heap: TaskHeap = content.parse()?;
        heap.set_name(
            file.file_name()
                .to_str()
                .and_then(|s| s.split_once("."))
                .ok_or_else(|| HeapError::CorruptData(heap.get_name().to_owned()))?
                .0,
        );
        heapmap.insert(heap.get_name().to_owned(), heap);
    }
    Ok(heapmap)
}
pub fn read_meta_file(heapmap: &HeapMap) -> Result<Option<TaskID>, HeapError> {
    let db_path = get_db_path();
    let meta_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(db_path.join(format!("meta.{}", EXTENSION)))?;
    let mut reader = BufReader::new(meta_file).lines();
    if let Some(Ok(line)) = reader.next() {
        if line == VOID || line.is_empty() {
            let task_ref: Option<TaskID> = heapmap
                .iter()
                .flat_map(|stack| stack.1.get_tasks_iter())
                .find(|task| task.is_in_progress())
                .map(|opt| (opt.get_heap_name().to_owned(), opt.get_name().to_owned()));
            return Ok(task_ref);
        }
        let mut parts = line.splitn(2, ".");
        let heap_name = parts
            .next()
            .ok_or_else(|| HeapError::CorruptData(line.to_owned()))?;
        let task_name = parts
            .next()
            .ok_or_else(|| HeapError::CorruptData(line.to_owned()))?;
        let task_id: TaskID = (heap_name.to_owned(), task_name.to_owned());
        match check_task_id(heapmap, &task_id) {
            Ok(()) => (),
            Err(e) => {
                println!("Warning: Active task in meta file is invalid: {e}");
                return Ok(None);
            }
        }
        Ok(Some(task_id))
    } else {
        Ok(None)
    }
}
fn print_task_table(tasks: &Vec<&Task>) {
    if tasks.is_empty() {
        println!("No tasks to display.");
        return;
    }
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    const W_WEIGHT: usize = 8;
    const W_STATE: usize = 8;
    const BORDER_OVERHEAD: usize = 13;

    let remaining_width = term_width.saturating_sub(W_WEIGHT + W_STATE + BORDER_OVERHEAD);
    const RATIO_NAME_DESC: f64 = 0.3;
    let w_name = (remaining_width as f64 * RATIO_NAME_DESC) as usize;
    let w_description = (remaining_width as f64 * (1. - RATIO_NAME_DESC)) as usize;
    let w_name = w_name.max(5);
    let w_description = w_description.max(10);
    //println!("{}", "-".repeat(term_width));
    println!(
        "{:<n$} | {:<d$} | {:>w$} | {:<t$}",
        "TASK",
        "DESCRIPTION",
        "WEIGHT",
        "STATE",
        n = w_name,
        d = w_description,
        w = W_WEIGHT,
        t = W_STATE
    );
    for task in tasks {
        let state = task.get_state();
        let state_str = match state {
            TaskStatus::Idle => "IDLE",
            TaskStatus::Staged => "STAGED",
            TaskStatus::InProgress => "INPROG",
            TaskStatus::Finished => "DONE",
        };
        // Tags need to be sorted to look consistent (HashSet is random!)
        let state_lines = wrap(state_str, W_STATE);
        let name_lines = wrap(task.get_name(), w_name);
        let desc_lines = wrap(task.get_description(), w_description);
        let max_lines = name_lines
            .len()
            .max(desc_lines.len())
            .max(state_lines.len());
        for i in 0..max_lines {
            let name_part = name_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let desc_part = desc_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let state_part = state_lines.get(i).map(|s| s.as_ref()).unwrap_or("");

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
                state_part,
                n = w_name,
                d = w_description,
                w = W_WEIGHT,
                t = W_STATE
            );
        }
        //println!("{}", "-".repeat(term_width));
    }
}

fn print_heap_headers(heaps: Vec<&TaskHeap>) {
    if heaps.is_empty() {
        println!("No task stacks to display.");
        return;
    }
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    const W_WEIGHT: usize = 8;
    const W_STATE_SUM: usize = 15;
    const BORDER_OVERHEAD: usize = 13;

    let remaining_width = term_width.saturating_sub(W_WEIGHT + W_STATE_SUM + BORDER_OVERHEAD);
    const RATIO_NAME_TAGS: f64 = 0.5;
    let w_name = (remaining_width as f64 * RATIO_NAME_TAGS) as usize;
    let w_tags = (remaining_width as f64 * (1. - RATIO_NAME_TAGS)) as usize;
    let w_name = w_name.max(5);
    let w_tags = w_tags.max(10);
    //println!("+{}", "-".repeat(term_width - 5));
    println!(
        "{:<n$} | {:>w$} | {:<t$} | {:<s$}",
        "STACK",
        "WEIGHT",
        "TAGS",
        "I/S/P/D",
        n = w_name,
        t = w_tags,
        w = W_WEIGHT,
        s = W_STATE_SUM,
    );
    //println!("{}", "-".repeat(term_width));
    // Tags need to be sorted to look consistent (HashSet is random!)
    for heap in heaps {
        let mut tags = heap.get_tags();
        tags.sort();
        let tags_string = tags
            .into_iter()
            .map(|s| s.to_owned())
            .collect::<Vec<_>>()
            .join(",");
        let tag_lines = wrap(&tags_string, w_tags);
        let name_lines = wrap(heap.get_name(), w_name);
        let state_sums = heap.get_states_sum();
        let order = vec![
            TaskStatus::Idle,
            TaskStatus::Staged,
            TaskStatus::InProgress,
            TaskStatus::Finished,
        ];
        let state_str = order
            .into_iter()
            .map(|variant| state_sums.get(&variant).unwrap_or(&0u32).to_string())
            .collect::<Vec<_>>()
            .join("/");
        let state_lines = wrap(&state_str, w_tags);
        let max_lines = name_lines.len().max(state_lines.len()).max(tag_lines.len());
        for i in 0..max_lines {
            let name_part = name_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let state_part = state_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let tags_part = tag_lines.get(i).map(|s| s.as_ref()).unwrap_or("");

            // Only print Weight/State on the FIRST line of the row
            let weight_part = if i == 0 {
                truncate(&heap.get_weight().to_string(), W_WEIGHT)
            } else {
                "".to_owned()
            };

            println!(
                "{:<n$} | {:<w$} | {:>t$} | {:<s$}",
                name_part,
                weight_part,
                tags_part,
                state_part,
                n = w_name,
                t = w_tags,
                w = W_WEIGHT,
                s = W_STATE_SUM
            );
        }
    }
    //rintln!("{}", "-".repeat(term_width));
}
//fn print_single_header(heap: &TaskHeap) {
//    let term_width = if let Some((Width(w), _)) = terminal_size() {
//        w as usize
//    } else {
//        80
//    };
//
//    const W_WEIGHT: usize = 8;
//    const W_STATE_SUM: usize = 15;
//    const BORDER_OVERHEAD: usize = 19;
//
//    let remaining_width = term_width.saturating_sub(W_WEIGHT + W_STATE_SUM + BORDER_OVERHEAD);
//    const RATIO_NAME_TAGS: f64 = 0.5;
//    let w_name = (remaining_width as f64 * RATIO_NAME_TAGS) as usize;
//    let w_tags = (remaining_width as f64 * (1. - RATIO_NAME_TAGS)) as usize;
//    let w_name = w_name.max(5);
//    let w_tags = w_tags.max(10);
//    //println!("+{}", "-".repeat(term_width - 5));
//    //println!(
//    //    "{:<n$} | {:>w$} | {:<t$} | {:<s$}",
//    //    "HEAP NAME",
//    //    "WEIGHT",
//    //    "TAGS",
//    //    "STATE SUM",
//    //    n = w_name,
//    //    t = w_tags,
//    //    w = W_WEIGHT,
//    //    s = W_STATE_SUM,
//    //);
//    //println!("{}", "-".repeat(term_width));
//    // Tags need to be sorted to look consistent (HashSet is random!)
//    let mut tags = heap.get_tags();
//    tags.sort();
//    let tags_string = tags
//        .into_iter()
//        .map(|s| s.to_owned())
//        .collect::<Vec<_>>()
//        .join(",");
//    let tag_lines = wrap(&tags_string, w_tags);
//    let name_lines = wrap(heap.get_name(), w_name);
//    let state_sums = heap.get_states_sum();
//    let order = vec![
//        TaskStatus::Idle,
//        TaskStatus::Staged,
//        TaskStatus::InProgress,
//        TaskStatus::Finished,
//    ];
//    let state_str = order
//        .into_iter()
//        .map(|variant| state_sums.get(&variant).unwrap_or(&0u32).to_string())
//        .collect::<Vec<_>>()
//        .join("/");
//    let state_lines = wrap(&state_str, w_tags);
//    let max_lines = name_lines.len().max(state_lines.len()).max(tag_lines.len());
//    for i in 0..max_lines {
//        let name_part = name_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
//        let state_part = state_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
//        let tags_part = tag_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
//
//        // Only print Weight/State on the FIRST line of the row
//        let weight_part = if i == 0 {
//            truncate(&heap.get_weight().to_string(), W_WEIGHT)
//        } else {
//            "".to_owned()
//        };
//
//        println!(
//            "Heap: {:<n$} | {:<w$} | {:>t$} | {:<s$}",
//            name_part,
//            weight_part,
//            tags_part,
//            state_part,
//            n = w_name,
//            t = w_tags,
//            w = W_WEIGHT,
//            s = W_STATE_SUM
//        );
//    }
//    //rintln!("{}", "-".repeat(term_width));
//}

pub fn print_tasks_standalone<W: Write>(tasks: Vec<&Task>, str: &mut W) -> Result<(), HeapError> {
    if tasks.is_empty() {
        writeln!(str, "No tasks to display.")?;
        return Ok(());
    }
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    const W_WEIGHT: usize = 8;
    const W_STATE: usize = 8;
    const BORDER_OVERHEAD: usize = 13;

    let remaining_width = term_width.saturating_sub(W_WEIGHT + W_STATE + BORDER_OVERHEAD);
    const RATIO_NAME_DESC: f64 = 0.4;
    let w_name = (remaining_width as f64 * RATIO_NAME_DESC) as usize;
    let w_description = (remaining_width as f64 * (1. - RATIO_NAME_DESC)) as usize;
    let w_name = w_name.max(5);
    let w_description = w_description.max(10);
    //println!("{}", "-".repeat(term_width));
    writeln!(
        str,
        "{:<n$} | {:<d$} | {:>w$} | {:<t$}",
        "STACK.TASK",
        "DESCRIPTION",
        "WEIGHT",
        "STATE",
        n = w_name,
        d = w_description,
        w = W_WEIGHT,
        t = W_STATE
    )?;
    for task in tasks {
        let state = match task.get_state() {
            TaskStatus::Idle => "IDLE",
            TaskStatus::Staged => "STAGED",
            TaskStatus::InProgress => "PROG",
            TaskStatus::Finished => "DONE",
        }; // Tags need to be sorted to look consistent (HashSet is random!)
        let state_lines = wrap(state, W_STATE);
        let full_name = task.get_full_name();
        let name_lines = wrap(&full_name, w_name);
        let desc_lines = wrap(task.get_description(), w_description);
        let max_lines = name_lines
            .len()
            .max(desc_lines.len())
            .max(state_lines.len());
        for i in 0..max_lines {
            let name_part = name_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let desc_part = desc_lines.get(i).map(|s| s.as_ref()).unwrap_or("");
            let state_part = state_lines.get(i).map(|s| s.as_ref()).unwrap_or("");

            // Only print Weight/Tags on the FIRST line of the row
            let weight_part = if i == 0 {
                truncate(&task.get_weight().to_string(), W_WEIGHT)
            } else {
                "".to_owned()
            };

            writeln!(
                str,
                "{:<n$} | {:<d$} | {:>w$} | {:<t$}",
                name_part,
                desc_part,
                weight_part,
                state_part,
                n = w_name,
                d = w_description,
                w = W_WEIGHT,
                t = W_STATE
            )?;
        }
        //println!("{}", "-".repeat(term_width));
    }
    Ok(())
}

pub fn print_single_heap(heap: &TaskHeap, staged_only: bool) {
    print_heap_headers(vec![heap]);
    let tasks = if staged_only {
        heap.get_staged_tasks()
    } else {
        heap.get_all_tasks()
    };
    print_task_table(&tasks);
}

pub fn print_all_tasks_flat(
    heapmap: &HeapMap,
    staged_only: bool,
    tags: &[String],
) -> Result<(), HeapError> {
    let mut heaps = heapmap
        .values()
        .filter(|heap| heap.has_tags(tags) && (!staged_only || !heap.is_all_complete()))
        .collect::<Vec<_>>();
    heaps.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    let all_tasks = heaps
        .iter()
        .flat_map(|heap| {
            if staged_only {
                heap.get_staged_tasks()
            } else {
                heap.get_all_tasks()
            }
        })
        .collect::<Vec<_>>();
    //print_heap_headers(heaps);
    {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        print_tasks_standalone(all_tasks, &mut handle)?;
    }
    Ok(())
}

pub fn print_all_tasks(heapmap: &HeapMap, staged_only: bool, tags: &[String]) {
    let mut stacks = heapmap
        .values()
        .filter(|heap| heap.has_tags(tags) && (!staged_only || !heap.is_all_complete()))
        .collect::<Vec<_>>();
    stacks.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    if stacks.is_empty() {
        println!("No task stacks to display.");
        return;
    }
    for heap in stacks {
        print_single_heap(heap, staged_only);
    }
}

pub fn print_heaps_only(heapmap: &HeapMap, tags: &[String]) {
    let mut heaps = heapmap
        .values()
        .filter(|heap| heap.has_tags(tags))
        .collect::<Vec<_>>();
    heaps.sort_by(|a, b| a.get_name().cmp(b.get_name()));
    print_heap_headers(heaps);
}

pub fn get_yes_no() -> Result<String, HeapError> {
    print!(" [y/n]:");
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

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}..", &s[..max_width - 2])
    } else {
        s.to_string()
    }
}
