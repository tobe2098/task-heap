use crate::action::Action;
use crate::error::HeapError;
use crate::utils::{self, NumOrStr, Weight, get_description, get_heap_and_task, get_name};
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
const UNSTAGE_TASK_CMD: &str = "unstage";
const CURRENT_TASKS_CMD: &str = "current";
const STAGED_TASK_CMD: &str = "selected";
const COMPLETE_CURRENT_CMD: &str = "complete";
const CLEAR_DONE_CMD: &str = "clear-done";
const CLEAR_ALL_TASKS_CMD: &str = "clear-all";
const LIST_CMD: &str = "list";
const FLATLIST_CMD: &str = "flatlist";
const HEAPS_CMD: &str = "heaps";
const HELP_CMD: &str = "help";
const NAME_OPT: &str = "--name";
const NAME_OPT2: &str = "-n";
const DESCRIPTION_OPT: &str = "--description";
const DESCRIPTION_OPT2: &str = "-d";
const WEIGHT_OPT: &str = "--weight";
const WEIGHT_OPT2: &str = "-w";
const TAG_OPT: &str = "--tag";
const UNTAG_OPT: &str = "--untag";
const STAGED_OPT1: &str = "--staged";
const STAGED_OPT2: &str = "-s";
pub enum Command<'a> {
    CreateHeap(String),
    DestroyHeap(String),
    PushTask((String, String)),           //Push task onto stack.
    PopTask(Vec<String>),                 //Pop task from staged tasks.
    InsertTask((String, String, usize)),  //Indexed or by name.
    RemoveTask((String, NumOrStr<'a>)),   //||
    Edit((String, Option<NumOrStr<'a>>)), //Both stack or task and the stack is the argument.
    FinishTask((String, NumOrStr<'a>)),
    StageTask((String, NumOrStr<'a>)),   //Arg is stack always
    UnstageTask((String, NumOrStr<'a>)), //Stage or unstage
    StartTask((String, NumOrStr<'a>)),   //Stage or unstage
    CurrentTasks,
    StagedTasks,
    CompleteCurrent,
    ClearDone(String),     //Arg is stack
    ClearAllTasks(String), //Arg is stack
    List(Option<String>),  //Either a specific stack, or stacks only
    FlatList,
    Heaps,
    Help,
}
pub enum Options {
    Name(String),
    Description(String),
    Weight(Weight),
    Tags(Vec<String>),
    Untag(Vec<String>),
    Staged,
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
            (Command::List(_), Self::Tags(_) | Self::Staged) => true,
            (Command::FlatList, Self::Tags(_) | Self::Staged) => true,
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
        UNSTAGE_TASK_CMD => {
            let option_pair = get_heap_and_task(&mut args_iterator)?;
            let Some(heap_name) = option_pair.0 else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    UNSTAGE_TASK_CMD.to_owned(),
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
                        UNSTAGE_TASK_CMD.to_owned(),
                    )));
                }
            };
            Command::UnstageTask((heap_name, task_index_or_name))
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
        FLATLIST_CMD => Command::FlatList,
        HEAPS_CMD => Command::Heaps,
        HELP_CMD => Command::Help,
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
            STAGED_OPT1 | STAGED_OPT2 => Options::Staged,
            unknown_opt => {
                println!("{unknown_opt} ignored, it is not an argument or option.");
                continue;
            }
        };
        option_args.push(option);
    }
    Ok((cmd, option_args))
}
