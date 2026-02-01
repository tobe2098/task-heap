use crate::action::Action;
use crate::error::HeapError;
use crate::utils::{self, NumOrStr, get_description, get_heap_and_task, get_name};
const MK_HEAP_CMD: &str = "create";
const RM_HEAP_CMD: &str = "destroy";
const PUSH_TASK_CMD: &str = "push";
const POP_TASK_CMD: &str = "pop";
const INSERT_TASK_CMD: &str = "insert";
const REMOVE_TASK_CMD: &str = "remove";
const EDIT_CMD: &str = "edit";
const FINISH_TASK_CMD: &str = "finish";
const STAGE_TASK_CMD: &str = "stage";
const UNSTAGE_TASK_CMD: &str = "unstage";
const CURRENT_TASKS_CMD: &str = "current";
const COMPLETE_CURRENT_CMD: &str = "complete";
const CLEAR_DONE_CMD: &str = "clear-done";
const CLEAR_ALL_TASKS_CMD: &str = "clear-all";
const LIST_CMD: &str = "list";
const HELP_CMD: &str = "help";
const NAME_OPT: &str = "--name";
const NAME_OPT2: &str = "-n";
const DESCRIPTION_OPT: &str = "--description";
const DESCRIPTION_OPT2: &str = "-d";
const WEIGHT_OPT: &str = "--weight";
const WEIGHT_OPT2: &str = "-w";
const TAG_OPT: &str = "--tag";
const UNTAG_OPT: &str = "--untag";
const ALL_OPT: &str = "-a";
pub enum Commands {
    CreateHeap(String),
    DestroyHeap(String),
    PushTask((String, String)),          //Push task onto stack.
    PopTask(Option<String>),             //Pop task from staged tasks.
    InsertTask((String, String, usize)), //Indexed or by name.
    RemoveTask((String, NumOrStr)),      //||
    Edit((String, Option<NumOrStr>)),    //Both stack or task and the stack is the argument.
    FinishTask((String, NumOrStr)),
    StageTask((String, NumOrStr)),   //Arg is stack always
    UnstageTask((String, NumOrStr)), //Stage or unstage
    CurrentTasks,
    CompleteCurrent,
    ClearDone(String),     //Arg is stack
    ClearAllTasks(String), //Arg is stack
    List(Option<String>),  //Either a specific stack, or stacks only
    Help,
}
pub enum Options {
    Name(String),
    Description(String),
    Weight(String),
    Tags(Vec<String>),
    Untag(Vec<String>),
    All,
}
impl Options {
    pub fn is_valid_for(&self, command: &Commands) -> bool {
        match (command, self) {
            // Push accepts everything except Untag
            (Commands::CreateHeap(_), Self::Description(_) | Self::Weight(_) | Self::Tags(_)) => {
                true
            }
            (Commands::DestroyHeap(_), Self::Tags(_)) => true,
            (
                Commands::PushTask(_),
                Self::Name(_) | Self::Description(_) | Self::Weight(_) | Self::Tags(_),
            ) => true,

            // Pop/Delete ONLY accept filtering tags
            //Pop takes the tag as an optional arg
            (
                Commands::InsertTask(_),
                Self::Name(_) | Self::Description(_) | Self::Weight(_) | Self::Tags(_),
            ) => true,

            (Commands::RemoveTask(_), Self::Name(_)) => true,

            // Edit accepts specific fields
            (
                Commands::Edit(_),
                Self::Name(_)
                | Self::Description(_)
                | Self::Weight(_)
                | Self::Tags(_)
                | Self::Untag(_),
            ) => true,

            //List accepts tag and weight (for now equal, but <> in future)
            (Commands::List(_), Self::Tags(_)) => true,

            // Default to false for everything else
            _ => false,
        }
    }
}
pub fn parse_command(mut args_iterator: utils::ArgsIter) -> Result<Action, HeapError> {
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
            Commands::CreateHeap(heap_name)
        }
        RM_HEAP_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    RM_HEAP_CMD.to_owned(),
                )));
            };
            Commands::DestroyHeap(heap_name)
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
            Commands::PushTask((heap_name, task_name))
        }
        POP_TASK_CMD => Commands::PopTask(utils::get_heap_name(&mut args_iterator)?),
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
            Commands::InsertTask((heap_name, task_name, task_index))
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
                    Err(_) => NumOrStr::Str(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        REMOVE_TASK_CMD.to_owned(),
                    )));
                }
            };
            Commands::RemoveTask((heap_name, task_index_or_name))
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
                    Err(_) => Some(NumOrStr::Str(task_index)),
                },
                None => None,
            };
            Commands::Edit((heap_name, task_index_or_name))
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
                    Err(_) => NumOrStr::Str(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        FINISH_TASK_CMD.to_owned(),
                    )));
                }
            };
            Commands::FinishTask((heap_name, task_index_or_name))
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
                    Err(_) => NumOrStr::Str(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        STAGE_TASK_CMD.to_owned(),
                    )));
                }
            };
            Commands::StageTask((heap_name, task_index_or_name))
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
                    Err(_) => NumOrStr::Str(task_index),
                },
                None => {
                    return Err(HeapError::MissingOption((
                        "valid heap index or task name".to_owned(),
                        UNSTAGE_TASK_CMD.to_owned(),
                    )));
                }
            };
            Commands::UnstageTask((heap_name, task_index_or_name))
        }
        CURRENT_TASKS_CMD => Commands::CurrentTasks,
        COMPLETE_CURRENT_CMD => Commands::CompleteCurrent,
        CLEAR_DONE_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    CLEAR_DONE_CMD.to_owned(),
                )));
            };
            Commands::ClearDone(heap_name)
        }
        CLEAR_ALL_TASKS_CMD => {
            let Some(heap_name) = utils::get_heap_name(&mut args_iterator)? else {
                return Err(HeapError::MissingOption((
                    "heap name".to_owned(),
                    CLEAR_ALL_TASKS_CMD.to_owned(),
                )));
            };
            Commands::ClearAllTasks(heap_name)
        }
        LIST_CMD => {
            let heap_name = utils::get_heap_name(&mut args_iterator)?;
            Commands::List(heap_name)
        }
        HELP_CMD => Commands::Help,
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
                Options::Weight(contents)
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
