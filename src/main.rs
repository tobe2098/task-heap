//TODO: Single message for each command via help
//TODO: undo command. Undo with a Reverse action stack (prevact.csv)

mod commands;
mod error;
use error::HeapError;
mod io;
mod task;
use crate::{
    action::run_action,
    commands::parse_command,
    io::{read_meta_file, read_task_heap, write_meta_file, write_task_heap},
};
mod action;
mod heap;
mod utils;

use std::{env, io::Write};

fn main() -> Result<(), HeapError> {
    let args: Vec<String> = env::args().collect();
    let args_iterator = args.into_iter().skip(1).peekable();

    let action = parse_command(args_iterator)?;
    let mut heapmap = read_task_heap()?;
    let mut active_task = read_meta_file(&heapmap)?;
    let result_messages = run_action(action, &mut heapmap, &mut active_task)?;
    match write_meta_file(active_task) {
        Ok(_) => (),
        Err(e) => eprintln!(
            "Warning: could not write meta file, some data may have been corrupted: {}",
            e
        ),
    };
    match write_task_heap(heapmap) {
        Ok(_) => (),
        Err(e) => eprintln!(
            "Warning: could not write task heap file, some data may have been
corrupted: {}",
            e
        ),
    };
    std::io::stdout().write_all(&result_messages)?;
    Ok(())
}
