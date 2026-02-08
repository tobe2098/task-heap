use crate::commands::{Command, Options};
use crate::error::HeapError;
use crate::heap::{HeapBuilder, TaskHeap};
use crate::io::{print_help, print_single_task, print_task_table};
use crate::task::{Task, TaskBuilder};
use crate::utils::{HeapMap, NumOrStr, PeekIntoIter, Weight};
use Command::*;
use Options::*;
use rand::{distributions::WeightedIndex, prelude::*};

pub type Action = (Command, Vec<Options>);
fn ignore_options(options: PeekIntoIter<Options>) {
    let mut option_string = String::new();
    for option in options {
        match option {
            Name(name) => {
                option_string.push_str(&format!("Name: {}; ", name));
            }
            Description(desc) => {
                option_string.push_str(&format!("Description: {}; ", desc));
            }
            Weight(weight_str) => {
                option_string.push_str(&format!("Weight: {}; ", weight_str));
            }
            Tags(tags) => {
                option_string.push_str(&format!("Tag(s): {:?}; ", tags));
            }
            Untag(tags) => {
                option_string.push_str(&format!("Untag(s): {:?}; ", tags));
            }
            All => {
                option_string.push_str("-a; ");
            }
        }
    }
    if !option_string.is_empty() {
        println!("Ignoring options: \"{}\"", option_string);
    }
}
fn build_heap(
    name: String,
    mut options_iter: PeekIntoIter<Options>,
    command: &Command,
) -> TaskHeap {
    let mut builder = HeapBuilder::new(name);
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
        match qualifier {
            Description(desc) => {
                builder.description(desc);
            }
            Weight(weight_str) => {
                builder.weight(weight_str);
            }
            Tags(tags) => {
                for tag in tags {
                    builder.add_tag(tag);
                }
            }
            //Cannot be a non-qualifier
            _ => unreachable!(),
        };
    }
    TaskHeap::from(builder)
}
fn build_task(
    name: impl Into<String>,
    heap_name: impl Into<String>,
    mut options_iter: PeekIntoIter<Options>,
    command: &Command,
) -> Task {
    let mut builder = TaskBuilder::new(name.into(), heap_name.into());
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
        match qualifier {
            Description(desc) => {
                builder.description(desc);
            }
            Weight(weight_str) => {
                builder.weight(weight_str);
            }
            //Cannot be a non-qualifier
            _ => unreachable!(),
        };
    }
    Task::from(builder)
}
fn edit_heap(heap: &mut TaskHeap, mut options_iter: PeekIntoIter<Options>, command: &Command) {
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
        match qualifier {
            Name(name) => {
                heap.set_name(name);
            }
            Description(desc) => {
                heap.set_description(desc);
            }
            Weight(weight_str) => {
                heap.set_weight(weight_str);
            }
            Tags(tags) => {
                for tag in tags {
                    heap.add_tag(tag);
                }
            }
            Untag(tags) => {
                for tag in tags {
                    heap.remove_tag(tag);
                }
            }
            //Cannot be a non-qualifier
            _ => unreachable!(),
        };
    }
}
fn edit_task(task: &mut Task, mut options_iter: PeekIntoIter<Options>, command: &Command) {
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(&command)) {
        match qualifier {
            Name(name) => {
                task.set_name(name);
            }
            Description(desc) => {
                task.set_desc(desc);
            }
            Weight(weight_str) => {
                task.set_weight(weight_str);
            }
            //Cannot be a non-qualifier
            _ => unreachable!(),
        };
    }
}
fn pop_task(tag: Vec<String>, heapmap: &mut HeapMap) -> Result<(), HeapError> {
    let mut heaps: Vec<&mut TaskHeap> = heapmap
        .iter_mut()
        .map(|tuple| tuple.1)
        .filter(|heap| !heap.is_empty() && heap.has_tags(tag.as_slice()) && !heap.is_all_complete())
        .collect();
    if heaps.is_empty() {
        return Err(HeapError::NoHeapsFound);
    }
    let heap_weights = heaps.iter().map(|heap| heap.get_weight()).collect();
    let distribution = WeightedIndex::new(&heap_weights).unwrap();
    let mut rng = thread_rng();
    let selected_heap = heaps.get_mut(distribution.sample(&mut rng)).unwrap();
    // For simplicity, just pick the first heap that has tasks.
    let (staged_weights, mut staged_tasks): (Vec<Weight>, Vec<&mut Task>) =
        selected_heap.get_staged();

    let selected_task = if let Ok(dist) = WeightedIndex::new(&staged_weights) {
        // We know this index is valid because dist was built from staged_weights
        &mut staged_tasks[dist.sample(&mut rng)]
    } else {
        selected_heap
            .get_first_unfinished_task()
            .ok_or(HeapError::SomeHeapsAreFinished)?
    };
    selected_task.in_progress();
    //let task_name = selected_task.get_name().to_owned();
    //println!("Popped {}.{} task", &selected_heap.get_name(), task_name);
    print_single_task(selected_task);
    Ok(())
}
fn get_heap_from_arg(heap_name: String, heapmap: &mut HeapMap) -> Result<&mut TaskHeap, HeapError> {
    let Some(heap) = heapmap.get_mut(&heap_name) else {
        return Err(HeapError::HeapNotFound(heap_name.to_owned()));
    };
    Ok(heap)
}
fn get_task_from_arg<'a>(
    heap: &'a mut TaskHeap,
    task_idx_name: &NumOrStr,
) -> Result<(usize, &'a mut Task), HeapError> {
    let Some(task) = heap.get_task_mut(task_idx_name) else {
        return Err(HeapError::TaskNotFound(match task_idx_name {
            NumOrStr::Num(idx) => format!("in index {idx}"),
            NumOrStr::Str(name) => name.to_owned(),
        }));
    };
    Ok(task)
}
pub fn run_action(
    action: Action,
    mut heapmap: HeapMap,
    mut active_task: Option<&mut Task>,
) -> Result<HeapMap, HeapError> {
    let (command, options) = action;
    let options_iter = options.into_iter().peekable();
    match command {
        CreateHeap(heap_name) => {
            if heapmap.contains_key(&heap_name) {
                return Err(HeapError::HeapAlreadyExists(heap_name.to_owned()));
            }
            let heap = build_heap(heap_name, options_iter, &command);
            heapmap.insert(heap.get_name().to_owned(), heap);
        }
        DestroyHeap(heap_name) => match heapmap.remove(&heap_name) {
            Some(heap) => {
                let _ = active_task.take_if(|t| t.get_heap_name() == heap.get_name());
            }
            None => return Err(HeapError::HeapNotFound(heap_name.to_owned())),
        },
        PushTask((heap_name, task_name)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            if heap.get_task(&NumOrStr::Str(task_name)).is_some() {
                return Err(HeapError::TaskAlreadyExists(format!(
                    "{heap_name}.{task_name}"
                )));
            }
            heap.push(build_task(task_name, heap_name, options_iter, &command));
        }
        PopTask(tag_list) => {
            if let Some(task) = active_task.as_ref() {
                return Err(HeapError::ATaskIsAlreadyInProgress(task.get_full_name()));
            }
            pop_task(tag_list, &mut heapmap)?;
        }
        InsertTask((heap_name, task_name, index)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            if heap.get_task(&NumOrStr::Str(task_name)).is_some() {
                return Err(HeapError::TaskAlreadyExists(format!(
                    "{heap_name}.{task_name}"
                )));
            }
            heap.insert_task(
                build_task(task_name, heap_name, options_iter, &command),
                index,
            );
        }
        RemoveTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            let (idx, task) = get_task_from_arg(heap, &task_idx_name)?;
            heap.remove_task(idx);
            let _ = active_task.take_if(|t| t.get_full_name() == task.get_full_name());
        }
        Edit((heap_name, task_idx_name_opt)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            if let Some(task_idx_name) = task_idx_name_opt {
                let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
                edit_task(task, options_iter, &command);
            } else {
                edit_heap(heap, options_iter, &command);
            }
        }
        FinishTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.finish();
            let _ = active_task.take_if(|t| t.get_full_name() == task.get_full_name());
            print_single_task(task);
        }
        StageTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.stage();
            let _ = active_task.take_if(|t| t.get_full_name() == task.get_full_name());
            print_single_task(task);
        }
        UnstageTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.unstage();
            let _ = active_task.take_if(|t| t.get_full_name() == task.get_full_name());
            print_single_task(task);
        }
        StagedTasks => {
            let staged_tasks: Vec<&Task> = heapmap
                .values()
                .filter(|heap| !heap.is_empty())
                .flat_map(|heap| heap.get_staged_tasks())
                .collect();
            print_task_table(&staged_tasks);
        }
        CurrentTasks => {
            let current_tasks: Vec<&Task> = heapmap
                .values()
                .filter(|heap| !heap.is_empty())
                .flat_map(|heap| heap.get_current_tasks())
                .collect();
            print_task_table(&current_tasks);
        }
        CompleteCurrent => {
            let Some(task) = active_task.take() else {
                return Err(HeapError::NoTaskInProgress);
            };
            task.finish();
            println!("Completed {}", task.get_full_name())
        }
        ClearDone(heap_name) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            heap.clear_done();
        }
        ClearAllTasks(heap_name) => {
            let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
            heap.clear_all();
        }
        List(heap_name_opt) => {
            //Either list all heaps with their tasks or a single heap
            if let Some(heap_name) = heap_name_opt {
                let heap = get_heap_from_arg(heap_name, &mut heapmap)?;
                print_heaps(&vec![heap]);
            } else {
                let heaps: Vec<&TaskHeap> = heapmap.values().collect();
                print_heaps(&heaps);
            }
        }
        Heaps => {
            //Print only heap headers
            let heaps: Vec<&TaskHeap> = heapmap.values().collect();
            print_heaps(&heaps);
        }
        Help => {
            print_help();
        }
    };
    ignore_options(options_iter);
    //match write_task_heap(task_heap) {
    //    Ok(_) => Ok(()),
    //    Err(e) => Err(HeapError::FileError(e)),
    //}
    Ok(heapmap)
}
