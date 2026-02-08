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
use crate::{
    commands::{Command, parse_command},
    io::get_yes_no,
    utils::HeapMap,
};
mod action;
mod heap;
mod utils;
use utils::{Weight, extract_array_by_tag};

use rand::{distributions::WeightedIndex, prelude::*};
use std::env;

fn main() -> Result<(), HeapError> {
    let args: Vec<String> = env::args().collect();
    let mut args_iterator = args.into_iter().skip(1).peekable();

    let action = parse_command(args_iterator)?;

    let mut task_heap = read_task_heap().unwrap_or_else(|err| {
        println!("Error reading the task heap: {err}.\nCreating a new heap...");
        TaskHeap::new()
    });
    run_commands(commands)
}
