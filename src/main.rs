//TODO: store hash
//TODO: Move to stack?
//TODO: Multiple stacks, the stacks are in a heap
//TODO: Single message for each command via help
//Change storage to use a different separator/filter for non-separator chars.
//Wrap errors in another error type to be able to get "push": "Missing argument"
//Maybe move? within edit?
//Uses for -a flag
//For now, rigid args (req)
mod error;
use error::HeapError;
mod io;
mod task;
use io::{print_single_task, print_task_table, read_task_heap, write_task_heap};
mod commands;
use crate::{commands::Commands, io::get_yes_no, utils::TaskHeap};
use commands::Commands::*;
mod stack;
mod utils;
use utils::{Hash, Weight, extract_array_by_tag, get_non_opt_arg};

use rand::{distributions::WeightedIndex, prelude::*};
use std::env;

fn print_help() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("task-heap v{VERSION} prints tasks");
    println!("Usage: task-heap ([--action] [--options])*");
    println!();
    println!("Actions:");
    println!("\t-i, --push              Push a task by name onto the task heap,");
    println!("                          with optional tags and description.");
    println!();
    println!("\t-o, --pop               Pop a task at random from the task heap");
    println!("                          by weight, with optional tag filter.");
    println!();
    println!("\t-d, --delete            Delete a task by name or by tag.");
    println!();
    println!("\t-r, --reset             Delete all tasks.");
    println!();
    println!("\t-e, --edit              Edit a task's name, description, tags or");
    println!("                          weight.");
    println!();
    println!("\t-ct, --clear-tags       Clear all tags from a task by name.");
    println!();
    println!("\t-l, --list              List all tasks or tasks filtered by tag.");
    println!();
    println!("\t-h, --help              Print this message.");
    println!();
    println!("Options:");
    println!("\t-n, --name              Specify a new name when editing a task.");
    println!();
    println!("\t-p, --description       Specify a description when creating or");
    println!("                          editing a task.");
    println!();
    println!("\t-t, --tag               Specify a number of single-word tags to");
    println!("                          add to a task, or to filter tasks by.");
    println!();
    println!("\t-ut, --untag            Specify a number of single-word tags to");
    println!("                          remove from a task when editing.");
    println!();
    println!("\t-n, --name              Specify a new name when editing a task.");
    println!();
    //Add create, destroy (stack), delete is indexed (if no index it is a pop), insert is indexed (if no index it is a push) as well, and per stack.
    //Two states, staged and unstaged.
    //By default staged is the first task only, to stage more you need to specify
    //Randomly popping is from staged piles of each stack
    //When listing, show the stacks and their staged tasks, if list --stack name print the whole
    //info about stack. list --all prints everything.
    //Clear stack of finished tasks
    //Push pop arg are for the stacks
    //Stage unstage
    //Do I move away from -- actions? No more chaining arguments?
    //. action name --option aasdw daw daw d --option
    //naming: stack:task because of edit? or edit-task vs edit-stack? Same problem.  edit vs
    //edit stack --task name --options
    //Undo with a Reverse action stack (prevact.csv)
    //Settings:
    //Only one task possible in progress (settings)?
    //To use tag filter, just use the tag as a stack name
}

fn run_commands(commands: Vec<Commands>) -> Result<(), HeapError> {
    //-> Result<(), HeapError> {
    let mut task_heap = read_task_heap().unwrap_or_else(|err| {
        println!("Error reading the task heap: {err}.\nCreating a new heap...");
        HashMap::new()
    });
    let mut command_iter = commands.into_iter().peekable();
    while let Some(command) = command_iter.next() {
        match command {
            Push(ref argument) => {
                if task_heap.contains_key(&Task::hash_fn(argument)) {
                    return Err(HeapError::TaskAlreadyExists(argument.to_owned()));
                };
                let mut new_task = Task::from_arg(argument);
                while let Some(qualifier) = command_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
                    match qualifier {
                        Description(desc) => {
                            new_task.set_desc(desc);
                        }
                        Weight(weight_str) => {
                            new_task.set_weight(weight_str);
                        }
                        Tag(tags) => {
                            new_task.add_tags(tags);
                        }
                        //Cannot be a non-qualifier
                        _ => unreachable!(),
                    };
                }
                task_heap.insert(new_task.get_hash(), new_task);
            }
            Pop => {
                let tags = command_iter
                    .next_if(|cmd| matches!(cmd, Tag(_)))
                    .map(|cmd| match cmd {
                        Tag(names) => names,
                        _ => unreachable!(),
                    })
                    .unwrap_or_default();
                let tasks = extract_array_by_tag(&task_heap, &tags, |tuple| tuple.1);
                if tasks.is_empty() {
                    if !tags.is_empty() {
                        return Err(HeapError::NoTaggedElements(tags.join(",")));
                    } else {
                        return Err(HeapError::NoTasksOnHeap);
                    }
                }
                let weights: Vec<u32> = tasks.iter().map(|task| task.get_weight()).collect();
                let hashes: Vec<Hash> = tasks.into_iter().map(|task| task.get_hash()).collect();
                let distribution = WeightedIndex::new(&weights)
                    .expect("The set of tasks to choose from should not be empty");
                let mut rng = thread_rng();

                let selected_hash = hashes[distribution.sample(&mut rng)];
                let selected_task = &task_heap
                    .get(&selected_hash)
                    .expect("Error with random number generation or elements selection");
                println!("The selected task for completion is:");
                print_single_task(selected_task);
                print!("Are you certain you can complete it? Are you a chicken or a penguin?");
                let input = get_yes_no()?;
                if input.to_lowercase() == "y" {
                    task_heap.remove(&selected_hash);
                    println!("Task was popped. Penguin wishes you good luck!");
                } else {
                    println!("You gave up on the task. *Chicken noises*");
                }
            }
            Delete(argument) => {
                let tags = command_iter
                    .next_if(|cmd| matches!(cmd, Tag(_)))
                    .map(|cmd| match cmd {
                        Tag(names) => names,
                        _ => unreachable!(),
                    })
                    .unwrap_or_default();
                let tasks = match tags {
                    tags if !tags.is_empty() => {
                        let task_vec = extract_array_by_tag(&task_heap, &tags, |tuple| tuple.1);
                        if task_vec.is_empty() {
                            return Err(HeapError::NoTaggedElements(tags.join(",")));
                        } else {
                            task_vec
                        }
                    }
                    _ => {
                        if argument.is_empty() {
                            return Err(HeapError::MissingArgument((
                                "name or tag".to_owned(),
                                "delete".to_owned(),
                            )));
                        }
                        let hash = Task::hash_fn(&argument);
                        let Some(task_ref) = task_heap.get(&hash) else {
                            return Err(HeapError::TaskNotFound(argument));
                        };
                        vec![task_ref; 1]
                    }
                };
                println!("To be deleted:");
                print_task_table(&tasks);
                print!("Are you sure you want to delete?");
                let answer = get_yes_no()?;
                if answer.to_lowercase() == "y" {
                    let hashes_to_remove: Vec<Hash> =
                        tasks.into_iter().map(|task| task.get_hash()).collect();
                    for hash in hashes_to_remove {
                        task_heap.remove(&hash);
                    }
                    println!("Tasks deleted. *Chicken noises*?");
                }
            }
            Edit(ref argument) => {
                let Some(task) = task_heap.get_mut(&Task::hash_fn(argument)) else {
                    return Err(HeapError::TaskNotFound(argument.to_owned()));
                };
                let mut rehash = false;
                while let Some(qualifier) = command_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
                    match qualifier {
                        Name(name) => {
                            task.set_name(name);
                            rehash = true;
                        }
                        Description(desc) => {
                            task.set_desc(desc);
                        }
                        Weight(weight_str) => {
                            task.set_weight(weight_str);
                        }
                        Tag(tags) => {
                            task.add_tags(tags);
                        }
                        Untag(tags) => {
                            task.remove_tags(tags);
                        }
                        //Cannot be a non-qualifier
                        _ => unreachable!(),
                    };
                }
                if rehash {
                    let Some(updated_task) = task_heap.remove(&Task::hash_fn(argument)) else {
                        unreachable!();
                    };
                    task_heap.insert(updated_task.get_hash(), updated_task);
                }
            }
            ClearTags(argument) => {
                let Some(task) = task_heap.get_mut(&Task::hash_fn(&argument)) else {
                    return Err(HeapError::TaskNotFound(argument.to_owned()));
                };
                task.clear_tags();
            }
            List => {
                let tags = command_iter
                    .next_if(|cmd| matches!(cmd, Tag(_)))
                    .map(|cmd| match cmd {
                        Tag(names) => names,
                        _ => unreachable!(),
                    })
                    .unwrap_or_default();
                let tasks = extract_array_by_tag(&task_heap, &tags, |tuple| tuple.1);
                if tasks.is_empty() {
                    if tags.is_empty() {
                        return Err(HeapError::NoTaggedElements(tags.join(",")));
                    } else {
                        return Err(HeapError::NoTasksOnHeap);
                    }
                }
                print_task_table(&tasks);
            }
            Reset => {
                print!("Are you sure you want to erase your task heap?");
                let answer = get_yes_no()?;
                if answer.to_lowercase() == "y" {
                    task_heap.clear();
                }
            }
            Help => {
                print_help();
            }

            Name(argument) | Description(argument) | Weight(argument) => {
                println!("Standalone task qualifiers are ignored: {argument}")
            }
            Tag(argument) | Untag(argument) => {
                println!("Standalone task qualifiers are ignored: {argument:?}")
            }
        }
    }
    match write_task_heap(task_heap) {
        Ok(_) => Ok(()),
        Err(e) => Err(HeapError::FileError(e)),
    }
}

fn main() -> Result<(), HeapError> {
    let args: Vec<String> = env::args().collect();
    let mut args_iterator = args.into_iter().skip(1).peekable();

    let mut commands: Vec<Commands> = Vec::new();

    while let Some(arg) = args_iterator.next() {
        let contents: String = join_args(&mut args_iterator);
        commands.push(match arg.as_str() {
            "-i" | "--push" => {
                if contents.is_empty() {
                    return Err(HeapError::MissingArgument((
                        "name".to_owned(),
                        "push".to_owned(),
                    )));
                }
                Push(contents)
            }
            "-p" | "--description" => {
                if contents.is_empty() {
                    return Err(HeapError::MissingArgument((
                        "description".to_owned(),
                        "description".to_owned(),
                    )));
                }
                Description(contents)
            }
            "-n" | "--name" => {
                if contents.is_empty() {
                    return Err(HeapError::MissingArgument((
                        "name".to_owned(),
                        "name".to_owned(),
                    )));
                }
                Name(contents)
            }
            "-at" | "--tag" => {
                let tags: Vec<String> = contents
                    .split(",")
                    .map(|str| str.trim().to_owned())
                    .filter(|s| !s.is_empty() || !s.contains(""))
                    .collect();
                if tags.is_empty() {
                    return Err(HeapError::TagCannotBeEmpty);
                }
                Tag(tags)
            }
            "-ut" | "--untag" => {
                let tags: Vec<String> = contents
                    .split(",")
                    .map(|str| str.trim().to_owned())
                    .filter(|s| !s.is_empty() || !s.contains(""))
                    .collect();
                if tags.is_empty() {
                    return Err(HeapError::TagCannotBeEmpty);
                }
                Untag(tags)
            }
            "-w" | "--weight" => {
                if contents.is_empty() {
                    return Err(HeapError::MissingArgument((
                        "weight number".to_owned(),
                        "weight".to_owned(),
                    )));
                }
                Weight(contents)
            }
            "-o" | "--pop" => {
                if !contents.is_empty() {
                    return Err(HeapError::DoesNotTakeArg("pop".to_owned()));
                }
                Pop
            }
            "-d" | "--delete" => Delete(contents),
            // Needs to know if there are tags to consider the arg is incomplete
            "-r" | "--reset" => {
                if !contents.is_empty() {
                    return Err(HeapError::DoesNotTakeArg("reset".to_owned()));
                }
                Reset
            }
            "-e" | "--edit" => {
                if contents.is_empty() {
                    return Err(HeapError::MissingArgument((
                        "name".to_owned(),
                        "edit".to_owned(),
                    )));
                }
                Edit(contents)
            }
            "-ct" | "--clear-tags" => {
                if contents.is_empty() {
                    return Err(HeapError::MissingArgument((
                        "name".to_owned(),
                        "clear-tags".to_owned(),
                    )));
                }
                ClearTags(contents)
            }
            "-l" | "--list" => {
                if !contents.is_empty() {
                    return Err(HeapError::DoesNotTakeArg("list".to_owned()));
                }
                List
            }
            "-h" | "--help" => {
                if !contents.is_empty() {
                    return Err(HeapError::DoesNotTakeArg("help".to_owned()));
                }
                Help
            }
            unknown_arg => {
                println!("{unknown_arg} is not an argument.");
                continue;
            }
        });
    }
    run_commands(commands)
}
