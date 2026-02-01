use crate::{error::HeapError, stack::TaskStack};
use std::{collections::HashMap, iter::Peekable, iter::Skip, vec::IntoIter};

pub type ArgsIter = Peekable<Skip<IntoIter<String>>>;
pub type TaskHeap = HashMap<String, TaskStack>;
pub type Weight = u32;
pub const DEFAULT_WEIGHT: Weight = 100;
pub const SEPARATOR: &str = "||";

pub enum NumOrStr {
    Num(usize),
    Str(String),
}
pub fn extract_array_by_tag<'a, F, R>(map: &'a TaskHeap, tags: &[String], closure: F) -> Vec<R>
where
    F: FnMut((&'a String, &'a TaskStack)) -> R,
{
    map.iter()
        .filter(|tuple| tuple.1.has_tags(tags))
        .map(closure)
        .collect()
}

pub fn get_heap_and_task(
    args_iterator: &mut ArgsIter,
) -> Result<(Option<String>, Option<String>), HeapError> {
    let mut name_vec = match get_non_opt_arg(args_iterator) {
        Some(name_str) => name_str
            .split(".")
            .map(|s| s.to_owned())
            .collect::<Vec<String>>()
            .into_iter(),
        None => {
            return Ok((None, None));
        }
    };
    let heap_name = is_filename_safe(name_vec.next())?;
    let task_name = cannot_have_whitespace(cannot_have_separator(name_vec.next()))?;
    Ok((heap_name, task_name))
}

pub fn get_name(args_iterator: &mut ArgsIter) -> Result<Option<String>, HeapError> {
    cannot_have_whitespace(cannot_have_separator(get_non_opt_arg(args_iterator)))
}

pub fn get_heap_name(args_iterator: &mut ArgsIter) -> Result<Option<String>, HeapError> {
    is_filename_safe(get_non_opt_arg(args_iterator))
}

pub fn get_description(args_iterator: &mut ArgsIter) -> Result<Option<String>, HeapError> {
    cannot_have_separator(get_non_opt_arg(args_iterator))
}

fn get_non_opt_arg(args_iterator: &mut ArgsIter) -> Option<String> {
    if let Some(next_arg) = args_iterator.peek()
        && !next_arg.starts_with("-")
    {
        // We proved 'Some' exists with peek(), so unwrap() is safe.
        Some(args_iterator.next().unwrap().trim().to_owned())
    } else {
        None
    }
}

fn cannot_have_whitespace(
    argument: Result<Option<String>, HeapError>,
) -> Result<Option<String>, HeapError> {
    let argument = argument?;
    match argument {
        Some(argument) => {
            if !argument.contains(char::is_whitespace) {
                Ok(Some(argument))
            } else {
                Err(HeapError::ArgumentCannotHaveWhitespace)
            }
        }
        None => Ok(argument),
    }
}
fn cannot_have_separator(argument: Option<String>) -> Result<Option<String>, HeapError> {
    match argument {
        Some(argument) => {
            if !argument.contains(SEPARATOR) {
                Ok(Some(argument))
            } else {
                Err(HeapError::ArgumentCannotHaveSeparator)
            }
        }
        None => Ok(argument),
    }
}

fn is_filename_safe(argument: Option<String>) -> Result<Option<String>, HeapError> {
    match argument {
        Some(argument) => {
            if !argument
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                Ok(Some(argument))
            } else {
                Err(HeapError::InvalidHeapName)
            }
        }
        None => Ok(argument),
    }
}
