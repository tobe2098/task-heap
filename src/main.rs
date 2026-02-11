//TODO: store hash
//TODO: Move to stack?
//TODO: Multiple stacks, the stacks are in a heap
//TODO: Single message for each command via help
//Change storage to use a different separator/filter for non-separator chars.
//Wrap errors in another error type to be able to get "push": "Missing argument"
//Maybe move? within edit?
//Uses for -a flag
//For now, rigid args (req)
mod commands;
mod error;
use error::HeapError;
mod io;
mod task;
use crate::{
    action::run_action,
    commands::parse_command,
    io::{read_meta_file, read_task_heap, write_meta_file, write_task_heap},
    utils::HeapMap,
};
mod action;
mod heap;
mod utils;

use std::env;

fn main() -> Result<(), HeapError> {
    let args: Vec<String> = env::args().collect();
    let args_iterator = args.into_iter().skip(1).peekable();

    let action = parse_command(args_iterator)?;
    let mut heapmap = read_task_heap()?;
    let mut active_task = read_meta_file(&heapmap)?;
    run_action(action, &mut heapmap, &mut active_task)?;

    write_meta_file(active_task)?;
    write_task_heap(heapmap)?;
    Ok(())
}
