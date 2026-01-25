mod error;
use error::HeapError;
mod task;
use task::Task;
mod io;
use io::{print_task_table, read_task_heap, write_task_heap};
mod commands;
use commands::Commands::*;

use rand::{distributions::WeightedIndex, prelude::*};
use std::{
    collections::HashMap,
    env,
    io::{Write, stdin, stdout},
    iter::{Peekable, Skip},
    vec::IntoIter,
};

use crate::commands::Commands;
type ArgsIter = Peekable<Skip<IntoIter<String>>>;
type Hash = [u8; 32];
type TaskHeap = HashMap<Hash, Task>;
fn print_help() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("task-heap v{VERSION} prints tasks");
    println!("Usage:");
    println!("task-heap --push | push task description");
    println!("task-heap --pull | pull");
    println!("task-heap --list | ls");
    println!("task-heap --help | -h");
}

fn join_args(args_iterator: &mut ArgsIter) -> String {
    let mut param = Vec::new();

    while let Some(next_arg) = args_iterator.peek() {
        if next_arg.starts_with("-") {
            break;
        }
        // We proved 'Some' exists with peek(), so unwrap() is safe.
        let word = args_iterator.next().unwrap();

        param.push(word.trim().to_owned());
    }
    param.join(" ")
}
fn get_weights(map: &TaskHeap, tag: Option<&str>) -> Vec<u32> {
    map.iter()
        .filter(|tuple| {
            if let Some(tag_name) = tag {
                tuple.1.has_tag(tag_name)
            } else {
                true
            }
        })
        .map(|tuple| tuple.1.get_weight())
        .collect()
}

fn get_hashes(map: &TaskHeap, tag: Option<&str>) -> Vec<Hash> {
    map.iter()
        .filter(|tuple| {
            if let Some(tag_name) = tag {
                tuple.1.has_tag(tag_name)
            } else {
                true
            }
        })
        .map(|tuple| tuple.1.get_hash())
        .collect()
}
fn run_commands(commands: Vec<Commands>) -> Result<(), HeapError> {
    //-> Result<(), HeapError> {
    let mut task_heap = read_task_heap().unwrap_or_else(|err| {
        println!("Error reading the task heap:{err}.\nCreating a new heap...");
        HashMap::new()
    });
    let mut command_iter = commands.into_iter().peekable();
    while let Some(command) = command_iter.next() {
        match command {
            Push(ref argument) => {
                let mut new_task = Task::from_arg(argument);
                while let Some(qualifier) = command_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
                    match qualifier {
                        Description(desc) => {
                            new_task.set_desc(desc);
                        }
                        Weight(weight_str) => {
                            new_task.set_weight(weight_str);
                        }
                        Tag(tag) => {
                            if tag.is_empty() {
                                return Err(HeapError::TagCannotBeEmpty);
                            }
                            new_task.add_tag(tag);
                        }
                        //Cannot be a non-qualifier
                        _ => unreachable!(),
                    };
                }
                task_heap.insert(new_task.get_hash(), new_task);
            }
            Pop => {
                let tag = command_iter
                    .next_if(|cmd| matches!(cmd, Tag(_)))
                    .map(|cmd| match cmd {
                        Tag(name) => name,
                        _ => unreachable!(),
                    });
                let tag_option: Option<&str> = tag.as_deref();
                let weights = get_weights(&task_heap, tag_option);
                let hashes = get_hashes(&task_heap, tag_option);
                let distribution = WeightedIndex::new(&weights)
                    .expect("The set of tasks to choose from should not be empty");
                let mut rng = thread_rng();

                let selected_hash = hashes[distribution.sample(&mut rng)];
                let selected_task = &task_heap[&selected_hash];
                println!("The selected task for completion is:");
                print_task_table(vec![selected_task; 1]);
                print!("Are you certain you can complete it? *Chicken noises* [y/n]:");
                stdout().flush().unwrap(); //Flush so prompt appears before user input.

                let mut input = String::new();

                match stdin().read_line(&mut input) {
                    Ok(_) => {
                        input = input.trim().to_owned();
                    }
                    Err(e) => {
                        return Err(HeapError::FileError(e));
                    }
                }
                if input.to_lowercase() == "y" {
                    task_heap.remove(&selected_hash);
                    println!("Task was popped. Penguin wishes you good luck!");
                } else {
                    println!("You gave up on the task. *Chicken noises*");
                }
            }
            Delete(ref argument) => {}
            Edit(ref argument) => {}
            ClearTags(ref argument) => {}
            List => {}
            Reset => {}
            Help => {}

            Description(argument) | Weight(argument) | Tag(argument) | Untag(argument) => {
                println!("Standalone task qualifiers are ignored: {argument}")
            }
        }
    }
    match write_task_heap(task_heap) {
        Ok(_) => Ok(()),
        Err(e) => Err(HeapError::FileError(e)),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut args_iterator = args.into_iter().skip(1).peekable();

    let mut commands: Vec<Commands> = Vec::new();

    while let Some(arg) = args_iterator.next() {
        let contents = join_args(&mut args_iterator);
        commands.push(match arg.as_str() {
            "-u" | "--push" => Push(contents),
            "-n" | "--description" => Description(contents),
            "-at" | "--tag" => Tag(contents),
            "-ut" | "--untag" => Untag(contents),
            "-w" | "--weight" => Weight(contents),
            "-o" | "--pop" => {
                println!(">{contents} were ignored.");
                Pop
            }
            "-d" | "--delete" => Delete(contents),
            "-r" | "--reset" => {
                println!(">{contents} were ignored.");
                Reset
            }
            "-e" | "--edit" => Edit(contents),
            "-ct" | "--clear-tags" => ClearTags(contents),
            "-l" | "--list" => {
                println!(">{contents} were ignored.");
                List
            }
            "-h" | "--help" => {
                println!(">{contents} were ignored.");
                Help
            }
            unknown_arg => {
                println!("{unknown_arg} is not an argument.");
                continue;
            }
        });
    }
    run_commands(commands).unwrap();
}
