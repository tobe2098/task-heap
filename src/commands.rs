use crate::action::Action;
use crate::error::HeapError;
use crate::utils::{self, NumOrStr, Weight, get_description, get_heap_and_task, get_name};
use terminal_size::{Width, terminal_size};
use textwrap::wrap;

const MK_HEAP_CMD: &str = "create";
const RM_HEAP_CMD: &str = "destroy";
const PUSH_TASK_CMD: &str = "push";
const POP_TASK_CMD: &str = "pop";
const INSERT_TASK_CMD: &str = "insert";
const REMOVE_TASK_CMD: &str = "remove";
const EDIT_CMD: &str = "edit";
const FINISH_TASK_CMD: &str = "finish";
const START_TASK_CMD: &str = "start";
const STAGE_TASK_CMD: &str = "stage";
const RESET_TASK_CMD: &str = "reset";
const CURRENT_TASKS_CMD: &str = "current";
const STAGED_TASK_CMD: &str = "selected";
const COMPLETE_CURRENT_CMD: &str = "complete";
const CLEAR_DONE_CMD: &str = "clear-done";
const CLEAR_ALL_TASKS_CMD: &str = "clear-all";
const LIST_CMD: &str = "list";
const TASKLIST_CMD: &str = "tasks";
const HEAPS_CMD: &str = "stacks";
const HELP_CMD: &str = "help";
const NAME_OPT: &str = "--name";
const NAME_OPT2: &str = "-n";
const DESCRIPTION_OPT: &str = "--description";
const DESCRIPTION_OPT2: &str = "-d";
const WEIGHT_OPT: &str = "--weight";
const WEIGHT_OPT2: &str = "-w";
const TAG_OPT: &str = "--tag";
const UNTAG_OPT: &str = "--untag";
pub fn print_help_cmd(command: &str) {
    const WEIGHT_DESC: &str = "Numerical weight used to modify the likelihood of selection on 'pop'. Default weight is 100.";
    const TAGS_ADD_DESC: &str = "Add comma separated tags to the heap for filtering.";
    const TAGS_FILTER_DESC: &str =
        "Optional, filter heaps by the intersection of comma separated tags.";
    const TAGS_REMOVE_DESC: &str = "Remove comma separated tags from the heap.";
    const DESCRIPTION_DESC: &str = "Set the task's description.";
    const NAME_DESC: &str = "Change the task's name.";
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    const OVERHEAD: usize = 2;
    let term_width = term_width.saturating_sub(OVERHEAD);
    const CMD_WIDTH: usize = 15;
    const RATIO_NAME_DESC: f64 = 0.2;
    let w_cmd = (term_width as f64 * RATIO_NAME_DESC) as usize;
    let w_cmd = w_cmd.max(CMD_WIDTH);
    let w_description = (term_width as f64 * (1. - RATIO_NAME_DESC)) as usize;
    let (func_strs, arguments, options) = match command {
        //: (_, _, Vec<&str, &str>, _): (_, _, Vec<&str, &str>, _)
        MK_HEAP_CMD => (
            vec![
                "Create an empty task stack to store related tasks.".to_owned(),
                format!("Usage: task-heap {} <name> [options]", MK_HEAP_CMD),
            ],
            vec![(
                "name",
                "Immutable name of the task stack to be created. Valid names must consist of letters, numbers, or hyphens, must start with a letter, and must end with an alphanumeric character.",
            )],
            vec![(
                format!("{} | {} <weight>", WEIGHT_OPT, WEIGHT_OPT2),
                WEIGHT_DESC,
            )],
        ),
        RM_HEAP_CMD => (
            vec![
                "Destroy a task stack and all contained tasks.".to_owned(),
                format!("Usage: task-heap {} <name>", RM_HEAP_CMD),
            ],
            vec![("name", "Name of the task stack to be deleted.")],
            vec![],
        ),
        PUSH_TASK_CMD => (
            vec![
                "Push a new task by name onto the last slot of a task stack.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<name> [--options]",
                    PUSH_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack onto which to push the newly created task."),
                ("name", "Name of the task to be created. Valid names must consist of letters, numbers, or hyphens, must start with a letter, and must end with an alphanumeric character."),
            ],
            vec![
                (
                    format!("{} | {} <desc>", DESCRIPTION_OPT, DESCRIPTION_OPT2),
                    DESCRIPTION_DESC,
                ),
                (
                    format!("{} | {} <weight>", WEIGHT_OPT, WEIGHT_OPT2),
                    WEIGHT_DESC,
                ),
            ],
        ),
        POP_TASK_CMD => (
            vec![
                "Pop a task at random by selecting a non-empty task stack from the set of task stacks and then randomly selecting a staged task from that stack. If a task stack has no staged tasks, the first idle task of that stack is considered as staged."
                    .to_owned(),
                format!("Usage: task-heap {} <tags>", POP_TASK_CMD),
            ],
            vec![(
                "tags",
TAGS_FILTER_DESC
            )],
            vec![],
        ),
        INSERT_TASK_CMD => (
            vec![
                "Create a new task and insert it in a task stack by index.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<index> <name>  [--options] args",
                    INSERT_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack onto which to insert the newly created task."),
                ("index", "0-based index where the task will be inserted. The index must be an integer between 0 and the length of the task stack."),
                ("name", "Name of the task to be created. Valid names must consist of letters, numbers, or hyphens, must start with a letter, and must end with an alphanumeric character."),
            ],
            vec![
                (
                    format!("{} | {} <desc>", DESCRIPTION_OPT, DESCRIPTION_OPT2),
                    DESCRIPTION_DESC,
                ),
                (
                    format!("{} | {} <weight>", WEIGHT_OPT, WEIGHT_OPT2),
                    WEIGHT_DESC,
                ),
            ],
        ),
        REMOVE_TASK_CMD => (
            vec![
                "Delete a task from a task stack by name or by index.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<name|index> [--options]",
                    REMOVE_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack where the task to delete is stored."),
                ("index", "0-based index of the task to delete. The index must be an integer between 0 and the length of the task stack minus 1."),
                ("name", "Name of the task to delete."),
            ],
            vec![],
        ),
        EDIT_CMD => (
            vec![
                "Edit the properties of a task stack or of a task. Changing the name of a task stack is not supported.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>(.<name/index>) [--options]",
                    EDIT_CMD
                ),
            ],
            vec![
                ("stack", "Stack where the task to edit is stored."),
                ("index", "Optional 0-based index of the task to edit. The index must be an integer between 0 and the length of the task stack minus 1."),
                ("name", "Optional, name of the task to edit."),
            ],
            vec![
(format!("{} | {} <name>", NAME_OPT, NAME_OPT2), NAME_DESC),
                (
                    format!("{} | {} <desc>", DESCRIPTION_OPT, DESCRIPTION_OPT2),
                    DESCRIPTION_DESC,
                ),
                (
                    format!("{} | {} <weight>", WEIGHT_OPT, WEIGHT_OPT2),
                    WEIGHT_DESC,
                ),
                (format!("{} <tags>", TAG_OPT), TAGS_ADD_DESC),
                (
                    format!("{} <tags>", UNTAG_OPT),
                    TAGS_REMOVE_DESC,
                ),
            ],
        ),
        FINISH_TASK_CMD => (
            vec![
                "Set a task's status to 'done'.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<name/index> [--options]",
                    FINISH_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack where the task is stored."),
                ("index", "0-based index of the task to finish. The index must be an integer between 0 and the length of the task stack minus 1."),
                ("name", "Name of the task to finish."),
            ],
            vec![],
        ),
        START_TASK_CMD => (
            vec![
                "Set a task's status to 'in progress'.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<name/index> [--options]",
                    START_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack where the task is stored."),
                ("index", "0-based index of the task to finish. The index must be an integer between 0 and the length of the task stack minus 1."),
                ("name", "Name of the task to finish."),
            ],
            vec![],
        ),
        STAGE_TASK_CMD => (
            vec![
                "Stage a task for selection through `task-heap pop`. If a task stack has no staged tasks, the first idle task of that stack is considered as staged.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<name/index> [--options]",
                    STAGE_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack where the task is stored."),
                ("index", "0-based index of the task to finish. The index must be an integer between 0 and the length of the task stack minus 1."),
                ("name", "Name of the task to finish."),
            ],
            vec![],
        ),
        RESET_TASK_CMD => (
            vec![
                "Reset a task's status to 'idle'.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>.<name/index> [--options]",
                    RESET_TASK_CMD
                ),
            ],
            vec![
                ("stack", "Stack where the task is stored."),
                ("index", "0-based index of the task to finish. The index must be an integer between 0 and the length of the task stack minus 1."),
                ("name", "Name of the task to finish."),
            ],
            vec![],
        ),
        CURRENT_TASKS_CMD => (
            vec![
                "List the task that is currently in progress.".to_owned(),
                format!(
                    "Usage: task-heap {}",
                    CURRENT_TASKS_CMD
                ),
            ],
            vec![],
            vec![],
        ),
        STAGED_TASK_CMD => (
            vec![
                "List tasks that are staged for selection. If a task stack has no staged tasks, the first idle task of that stack is considered as staged.".to_owned(),
                format!(
                    "Usage: task-heap {}",
                    STAGED_TASK_CMD
                ),
            ],
            vec![],
            vec![],
        ),
        COMPLETE_CURRENT_CMD => (
            vec![
                "Complete the task that is currently in progress.".to_owned(),
                format!(
                    "Usage: task-heap {}",
                    COMPLETE_CURRENT_CMD
                ),
            ],
            vec![],
            vec![],
        ),
        CLEAR_DONE_CMD => (
            vec![
                "Erase all completed tasks in a task stack.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>",
                    CLEAR_DONE_CMD
                ),
            ],
            vec![
                ("stack", "Task stack to clear of completed tasks."),
            ],
            vec![],
        ),
        CLEAR_ALL_TASKS_CMD => (
            vec![
                "Erase all tasks in a task stack.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>",
                    CLEAR_ALL_TASKS_CMD
                ),
            ],
            vec![
                ("stack", "Task stack to clear of all tasks."),
            ],
            vec![],
        ),
        LIST_CMD => (
            vec![
                "List existing task stacks with each task stack's tasks or a task stack's tasks.".to_owned(),
                format!(
                    "Usage: task-heap {} <stack>",
                    LIST_CMD
                ),
            ],
            vec![
                ("stack", "Optional, task stack to list the tasks from."),
            ],
            vec![],
        ),
        TASKLIST_CMD => (
            vec![
                "List task stacks and then tasks.".to_owned(),
                format!(
                    "Usage: task-heap {}",
                    TASKLIST_CMD
                ),
            ],
            vec![],
            vec![],
        ),
        HEAPS_CMD => (
            vec![
                "List all task stacks.".to_owned(),
                format!(
                    "Usage: task-heap {}",
                    HEAPS_CMD
                ),
            ],
            vec![],
            vec![],
        ),
        HELP_CMD => (
            vec![
                "Print this message.".to_owned(),
                format!(
                    "Usage: task-heap {} <command>",
                    HELP_CMD
                ),
            ],
            vec![
                ("command", "Command to print information on."),
            ],
            vec![],
        ),
        command => (
            vec![
                format!("Unknown command: {command}"),
                format!(
                    "Usage: task-heap {} <command>",
                    HELP_CMD
                ),
            ],
            vec![
                ("command", "Command to print information on."),
            ],
            vec![],
        ),
    };
    for func_str in func_strs {
        let w_lines = wrap(&func_str, term_width);
        for w_line in w_lines {
            println!("{w_line}");
        }
    }
    if !arguments.is_empty() {
        println!();
        println!("Arguments:");
        for (arg, desc) in arguments {
            let desc_lines = wrap(desc, w_description);
            for (i, line) in desc_lines.iter().enumerate() {
                if i == 0 {
                    println!("  {:<w$} {}", arg, line, w = w_cmd);
                } else {
                    println!("  {:<w$} {}", "", line, w = w_cmd);
                }
            }
        }
    }
    if !options.is_empty() {
        println!();
        println!("Options:");
        for (arg, desc) in options {
            let desc_lines = wrap(desc, w_description);
            for (i, line) in desc_lines.iter().enumerate() {
                if i == 0 {
                    println!("  {:<w$} {}", arg, line, w = w_cmd);
                } else {
                    println!("  {:<w$} {}", "", line, w = w_cmd);
                }
            }
        }
    }
}
pub fn print_help() {
    let term_width = if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80
    };

    const OVERHEAD: usize = 2;
    let term_width = term_width.saturating_sub(OVERHEAD);
    const CMD_WIDTH: usize = 15;
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const RATIO_NAME_DESC: f64 = 0.2;
    let w_cmd = (term_width as f64 * RATIO_NAME_DESC) as usize;
    let w_cmd = w_cmd.max(CMD_WIDTH);
    let w_description = (term_width as f64 * (1. - RATIO_NAME_DESC)) as usize;

    let lines = vec![
        "Usage: task-heap [command] heap(.task/index) [--options] args".to_owned(),
        "task-heap v{VERSION} helps organize your tasks and choose what to do next from the CLI."
            .replace("{VERSION}", VERSION),
    ];
    for line in lines {
        let w_lines = wrap(&line, term_width);
        for w_line in w_lines {
            println!("{w_line}");
        }
    }
    println!();
    println!("Available commands:");
    let commands = vec![
        (MK_HEAP_CMD, "Create an empty heap to store related tasks."),
        (RM_HEAP_CMD, "Destroy a task heap and all contained tasks."),
        (
            PUSH_TASK_CMD,
            "Push a new task by name onto the last slot of a task heap.",
        ),
        (
            POP_TASK_CMD,
            "Pop a task at random from the set of heaps and start working.",
        ),
        (
            INSERT_TASK_CMD,
            "Create a new task and insert it in a heap by index.",
        ),
        (REMOVE_TASK_CMD, "Delete a task by name or by index."),
        (EDIT_CMD, "Edit a task's name, description, tags or weight."),
        (FINISH_TASK_CMD, "Set a task's status to 'done'."),
        (START_TASK_CMD, "Set a task's status to 'in progress'."),
        (STAGE_TASK_CMD, "Stage a task for selection."),
        (RESET_TASK_CMD, "Reset a task's status."),
        (
            COMPLETE_CURRENT_CMD,
            "Complete the task that is currently in progress.",
        ),
        (CLEAR_DONE_CMD, "Erase all dones tasks in a heap."),
        (CLEAR_ALL_TASKS_CMD, "Erase all tasks in a heap."),
        (LIST_CMD, "List existing heaps and each heap's tasks."),
        (TASKLIST_CMD, "List heaps and tasks."),
        (
            CURRENT_TASKS_CMD,
            "List the task that is currently in progress.",
        ),
        (STAGED_TASK_CMD, "List tasks that are staged for selection."),
        (HEAPS_CMD, "List existing heaps."),
        (HELP_CMD, "Print this message."),
    ];
    for (cmd, desc) in commands {
        let desc_lines = wrap(desc, w_description);
        for (i, line) in desc_lines.iter().enumerate() {
            if i == 0 {
                println!("  {:<w$} {}", cmd, line, w = w_cmd);
            } else {
                println!("  {:<w$} {}", "", line, w = w_cmd);
            }
        }
    }
    println!();
}
pub enum Command<'a> {
    CreateHeap(String),
    DestroyHeap(String),
    PushTask((String, String)),           //Push task onto stack.
    PopTask(Vec<String>),                 //Pop task from staged tasks.
    InsertTask((String, String, usize)),  //Indexed or by name.
    RemoveTask((String, NumOrStr<'a>)),   //||
    Edit((String, Option<NumOrStr<'a>>)), //Both stack or task and the stack is the argument.
    FinishTask((String, NumOrStr<'a>)),
    StageTask((String, NumOrStr<'a>)), //Arg is stack always
    ResetTask((String, NumOrStr<'a>)), //Stage or unstage
    StartTask((String, NumOrStr<'a>)), //Stage or unstage
    CurrentTasks,
    StagedTasks,
    CompleteCurrent,
    ClearDone(String),     //Arg is stack
    ClearAllTasks(String), //Arg is stack
    List(Option<String>),  //Either a specific stack, or stacks only
    FlatList,
    Heaps,
    Help(Option<String>), //Either a specific command or general help
}
pub enum Options {
    Name(String),
    Description(String),
    Weight(Weight),
    Tags(Vec<String>),
    Untag(Vec<String>),
}
impl Options {
    pub fn is_valid_for(&self, command: &Command) -> bool {
        match (command, self) {
            // Push accepts everything except Untag
            (Command::CreateHeap(_), Self::Weight(_) | Self::Tags(_)) => true,
            //Nmae in args
            (Command::PushTask(_), Self::Description(_) | Self::Weight(_)) => true,

            // Pop/Delete ONLY accept filtering tags
            //Pop takes the tag as an optional arg
            //Remove should take heapname.taskname
            (Command::InsertTask(_), Self::Description(_) | Self::Weight(_) | Self::Tags(_)) => {
                true
            }

            // Edit accepts specific fields
            (
                Command::Edit(_),
                Self::Name(_)
                | Self::Description(_)
                | Self::Weight(_)
                | Self::Tags(_)
                | Self::Untag(_),
            ) => true,

            //List accepts tag and weight (for now equal, but <> in future)
            (Command::List(_), Self::Tags(_)) => true,
            (Command::FlatList, Self::Tags(_)) => true,
            // Default to false for everything else
            _ => false,
        }
    }
}
pub fn parse_command<'a>(mut args_iterator: utils::ArgsIter) -> Result<Action<'a>, HeapError> {
    //Only one command, so:
    let Some(cmd_str) = args_iterator.next() else {
        return Err(HeapError::MissingCommand);
    };
    let cmd = match cmd_str.as_str() {
        MK_HEAP_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    MK_HEAP_CMD.to_owned(),
                )));
            };
            Command::CreateHeap(heap_name)
        }
        RM_HEAP_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    RM_HEAP_CMD.to_owned(),
                )));
            };
            Command::DestroyHeap(heap_name)
        }
        PUSH_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    PUSH_TASK_CMD.to_owned(),
                )));
            };
            let Some(task_name) = option_pair.1 else {
                return Err(HeapError::MissingOption((
                    "task name".to_owned(),
                    PUSH_TASK_CMD.to_owned(),
                )));
            };
            Command::PushTask((heap_name, task_name))
        }
        POP_TASK_CMD => {
            let tag_list = match utils::get_heap_name(&mut args_iterator)? {
                Some(tag_str) => tag_str
                    .split(",")
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty() || !s.contains(" "))
                    .collect(),
                None => Vec::new(),
            };
            Command::PopTask(tag_list)
        }
        INSERT_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    INSERT_TASK_CMD.to_owned(),
                )));
            };
            let task_index = match option_pair.1.map(|s| s.parse::<usize>()) {
                Some(Ok(task_index)) => task_index,
                None | Some(Err(_)) => {
                    return Err(HeapError::MissingOption((
                        "valid heap index".to_owned(),
                        INSERT_TASK_CMD.to_owned(),
                    )));
                }
            };
            let Some(task_name) = get_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "task name".to_owned(),
                    INSERT_TASK_CMD.to_owned(),
                )));
            };
            Command::InsertTask((heap_name, task_name, task_index))
        }
        REMOVE_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    REMOVE_TASK_CMD.to_owned(),
                )));
            };
            let task_index_or_name = match option_pair.1 {
                Some(task_index) => match task_index.parse::<usize>() {
                    Ok(index) => NumOrStr::Num(index),
                    Err(_) => NumOrStr::String(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        REMOVE_TASK_CMD.to_owned(),
                    )));
                }
            };
            Command::RemoveTask((heap_name, task_index_or_name))
        }
        EDIT_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    EDIT_CMD.to_owned(),
                )));
            };
            let task_index_or_name = match option_pair.1 {
                Some(task_index) => match task_index.parse::<usize>() {
                    Ok(index) => Some(NumOrStr::Num(index)),
                    Err(_) => Some(NumOrStr::String(task_index)),
                },
                None => None,
            };
            Command::Edit((heap_name, task_index_or_name))
        }
        FINISH_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    FINISH_TASK_CMD.to_owned(),
                )));
            };
            let task_index_or_name = match option_pair.1 {
                Some(task_index) => match task_index.parse::<usize>() {
                    Ok(index) => NumOrStr::Num(index),
                    Err(_) => NumOrStr::String(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        FINISH_TASK_CMD.to_owned(),
                    )));
                }
            };
            Command::FinishTask((heap_name, task_index_or_name))
        }
        START_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    START_TASK_CMD.to_owned(),
                )));
            };
            let task_index_or_name = match option_pair.1 {
                Some(task_index) => match task_index.parse::<usize>() {
                    Ok(index) => NumOrStr::Num(index),
                    Err(_) => NumOrStr::String(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        START_TASK_CMD.to_owned(),
                    )));
                }
            };
            Command::StartTask((heap_name, task_index_or_name))
        }
        STAGE_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    STAGE_TASK_CMD.to_owned(),
                )));
            };
            let task_index_or_name = match option_pair.1 {
                Some(task_index) => match task_index.parse::<usize>() {
                    Ok(index) => NumOrStr::Num(index),
                    Err(_) => NumOrStr::String(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        STAGE_TASK_CMD.to_owned(),
                    )));
                }
            };
            Command::StageTask((heap_name, task_index_or_name))
        }
        RESET_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    RESET_TASK_CMD.to_owned(),
                )));
            };
            let task_index_or_name = match option_pair.1 {
                Some(task_index) => match task_index.parse::<usize>() {
                    Ok(index) => NumOrStr::Num(index),
                    Err(_) => NumOrStr::String(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        RESET_TASK_CMD.to_owned(),
                    )));
                }
            };
            Command::ResetTask((heap_name, task_index_or_name))
        }
        STAGED_TASK_CMD => Command::StagedTasks,
        CURRENT_TASKS_CMD => Command::CurrentTasks,
        COMPLETE_CURRENT_CMD => Command::CompleteCurrent,
        CLEAR_DONE_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    CLEAR_DONE_CMD.to_owned(),
                )));
            };
            Command::ClearDone(heap_name)
        }
        CLEAR_ALL_TASKS_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    CLEAR_ALL_TASKS_CMD.to_owned(),
                )));
            };
            Command::ClearAllTasks(heap_name)
        }
        LIST_CMD => {
            let heap_name: Option<String> = utils::get_heap_name(&mut args_iterator)?;
            Command::List(heap_name)
        }
        TASKLIST_CMD => Command::FlatList,
        HEAPS_CMD => Command::Heaps,
        HELP_CMD => {
            let command_name: Option<String> = args_iterator.next();
            Command::Help(command_name)
        }
        unknown_cmd => {
            return Err(HeapError::UnknownCommand(unknown_cmd.to_owned()));
        }
    };

    let mut option_args: Vec<Options> = Vec::new();
    while let Some(opt) = args_iterator.next() {
        let option = match opt.as_str() {
            DESCRIPTION_OPT | DESCRIPTION_OPT2 => {
                let Some(contents) = get_description(&mut args_iterator)? else {
                    return Err(HeapError::MissingOption((
                        "description".to_owned(),
                        DESCRIPTION_OPT.to_owned(),
                    )));
                };
                Options::Description(contents)
            }
            NAME_OPT2 | NAME_OPT => {
                let Some(contents) = get_name(&mut args_iterator)? else {
                    return Err(HeapError::MissingOption((
                        "name".to_owned(),
                        NAME_OPT.to_owned(),
                    )));
                };
                Options::Name(contents)
            }
            TAG_OPT => {
                let Some(contents) = get_name(&mut args_iterator)? else {
                    return Err(HeapError::MissingOption((
                        "tag name".to_owned(),
                        TAG_OPT.to_owned(),
                    )));
                };
                let tags: Vec<String> = contents
                    .split(",")
                    .map(|str| str.trim().to_owned())
                    .filter(|s| !s.is_empty() || !s.contains(""))
                    .collect();
                if tags.iter().any(|s| s.is_empty()) {
                    return Err(HeapError::TagCannotBeEmpty);
                }
                Options::Tags(tags)
            }
            UNTAG_OPT => {
                let Some(contents) = get_name(&mut args_iterator)? else {
                    return Err(HeapError::MissingOption((
                        "tag name".to_owned(),
                        TAG_OPT.to_owned(),
                    )));
                };
                let tags: Vec<String> = contents
                    .split(",")
                    .map(|str| str.trim().to_owned())
                    .filter(|s| !s.is_empty() || !s.contains(""))
                    .collect();
                if tags.iter().any(|s| s.is_empty()) {
                    return Err(HeapError::TagCannotBeEmpty);
                }
                Options::Untag(tags)
            }
            WEIGHT_OPT | WEIGHT_OPT2 => {
                let Some(contents) = get_name(&mut args_iterator)? else {
                    return Err(HeapError::MissingOption((
                        "weight number".to_owned(),
                        WEIGHT_OPT.to_owned(),
                    )));
                };
                Options::Weight(contents.parse()?)
            }
            unknown_opt => {
                println!("{unknown_opt} ignored, it is not an argument or option.");
                continue;
            }
        };
        option_args.push(option);
    }
    Ok((cmd, option_args))
}
