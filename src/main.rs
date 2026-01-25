mod error;
use error::HeapError;
mod task;
use task::Task;
mod io;
use io::{read_task_heap, write_task_heap};
mod commands;
use commands::Commands::*;

use rand::{distributions::WeightedError, prelude::*};
use std::{
    collections::HashMap,
    env,
    iter::{Peekable, Skip},
    vec::IntoIter,
};

use crate::commands::Commands;
type ArgsIter = Peekable<Skip<IntoIter<String>>>;
type TaskHeap = HashMap<[u8; 32], Task>;
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
fn run_commands(command: Vec<Commands>) -> Result<(), HeapError> {
    //-> Result<(), HeapError> {
    let mut task_heap = read_task_heap().unwrap_or_else(|err| {
        println!("Error reading the task heap:{err}.\nCreating a new heap...");
        HashMap::new()
    });
    let mut command_iter = command.into_iter().peekable();
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
            Pop => {}
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
}
