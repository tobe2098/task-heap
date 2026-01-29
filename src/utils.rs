use crate::stack::TaskStack;
use std::{collections::HashMap, iter::Peekable, iter::Skip, vec::IntoIter};

pub type ArgsIter = Peekable<Skip<IntoIter<String>>>;
pub type TaskHeap = HashMap<String, TaskStack>;
pub type Weight = u32;
pub const DEFAULT_WEIGHT: Weight = 100;
pub fn extract_array_by_tag<'a, F, R>(map: &'a TaskHeap, tags: &[String], closure: F) -> Vec<R>
where
    F: FnMut((&'a String, &'a TaskStack)) -> R,
{
    map.iter()
        .filter(|tuple| tuple.1.has_tags(tags))
        .map(closure)
        .collect()
}

pub fn join_args(args_iterator: &mut ArgsIter) -> String {
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
