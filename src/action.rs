use crate::commands::{Command, Options};
use crate::error::HeapError;
use crate::heap::{HeapBuilder, TaskHeap};
use crate::io::{
    get_yes_no, print_all_tasks, print_all_tasks_flat, print_heaps_only, print_help,
    print_single_heap, print_tasks_standalone,
};
use crate::task::{Task, TaskBuilder};
use crate::utils::{
    HeapMap, NumOrStr, PeekIntoIter, TaskID, Weight, get_heap_from_arg, get_task_from_arg,
    get_task_from_id,
};
use Command::*;
use Options::*;
use rand::{distributions::WeightedIndex, prelude::*};

pub type Action<'a> = (Command<'a>, Vec<Options>);
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
            Staged => {
                option_string.push_str("-s; ");
            }
        }
    }
    if !option_string.is_empty() {
        println!("Ignoring options: \"{}\"", option_string);
    }
}
fn build_heap(
    name: impl Into<String>,
    options_iter: &mut PeekIntoIter<Options>,
    command: &Command,
) -> TaskHeap {
    let mut builder = HeapBuilder::new(name.into());
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(command)) {
        match qualifier {
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
    options_iter: &mut PeekIntoIter<Options>,
    command: &Command,
) -> Task {
    let mut builder = TaskBuilder::new(name.into(), heap_name.into());
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(command)) {
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
fn edit_heap(heap: &mut TaskHeap, options_iter: &mut PeekIntoIter<Options>, command: &Command) {
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(command)) {
        match qualifier {
            Name(name) => {
                heap.set_name(name);
            }
            Description(desc) => {
                println!(
                    "Heaps cannot have descriptions. Ignoring description: \"{}\"",
                    desc
                );
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
fn edit_task(task: &mut Task, options_iter: &mut PeekIntoIter<Options>, command: &Command) {
    while let Some(qualifier) = options_iter.next_if(|cmd| cmd.is_valid_for(command)) {
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
fn pop_task(tags: Vec<String>, heapmap: &mut HeapMap) -> Result<&mut Task, HeapError> {
    let candidates: Vec<(&String, Weight)> = heapmap
        .iter()
        .filter(|(_, heap)| !heap.is_empty() && heap.has_tags(&tags) && !heap.is_all_complete())
        .map(|(key, heap)| (key, heap.get_weight()))
        .collect();
    if candidates.is_empty() {
        return Err(HeapError::NoHeapsFound);
    }
    let distribution = WeightedIndex::new(candidates.iter().map(|t| t.1))
        .map_err(|_| HeapError::InvalidWeights)?;
    let mut rng = thread_rng();
    let (key, _) = candidates[distribution.sample(&mut rng)];
    let key = key.to_owned();
    // For simplicity, just pick the first heap that has tasks.
    let heap = heapmap.get_mut(&key).ok_or(HeapError::HeapNotFound(key))?;
    let (staged_weights, staged_tasks): (Vec<Weight>, Vec<usize>) = heap.get_staged();

    let selected_task: usize = if let Ok(dist) = WeightedIndex::new(&staged_weights) {
        // We know this index is valid because dist was built from staged_weights
        staged_tasks[dist.sample(&mut rng)]
    } else {
        heap.get_first_unfinished_task()
            .ok_or(HeapError::SomeHeapsAreFinished)?
    };
    let (_, selected_task) = heap
        .get_task_mut(&NumOrStr::Num(selected_task))
        .ok_or(HeapError::IndexError)?;
    println!("The selected task for completion is:");
    print_tasks_standalone(vec![selected_task], false);
    print!("Are you certain you can complete it? Are you a chicken or a penguin?");
    let input = get_yes_no()?;
    if input.to_lowercase() == "y" {
        println!("Task is in progress. Penguin wishes you good luck!");
    } else {
        println!("You gave up on the task. *Chicken noises*");
        return Err(HeapError::UserSaidNo);
    }
    selected_task.in_progress();
    //let task_name = selected_task.get_name().to_owned();
    //println!("Popped {}.{} task", &selected_heap.get_name(), task_name);
    Ok(selected_task)
}
pub fn run_action(
    action: Action,
    heapmap: &mut HeapMap,
    active_task: &mut Option<TaskID>,
) -> Result<(), HeapError> {
    let (command, options) = action;
    let mut options_iter = options.into_iter().peekable();
    match command {
        CreateHeap(ref heap_name) => {
            if heapmap.contains_key(heap_name) {
                return Err(HeapError::HeapAlreadyExists(heap_name.to_owned()));
            }
            let heap = build_heap(heap_name, &mut options_iter, &command);
            heapmap.insert(heap.get_name().to_owned(), heap);
        }
        DestroyHeap(heap_name) => {
            print!("Are you sure you want to delete {heap_name}?");
            let input = get_yes_no()?;
            if input.to_lowercase() == "y" {
                println!("Task heap deleted.");
            } else {
                return Err(HeapError::UserSaidNo);
            }

            match heapmap.remove(&heap_name) {
                Some(heap) => {
                    let _ = active_task.take_if(|t| t.0 == heap.get_name());
                }
                None => return Err(HeapError::HeapNotFound(heap_name.to_owned())),
            }
        }
        PushTask((ref heap_name, ref task_name)) => {
            let heap = get_heap_from_arg(heap_name, heapmap)?;
            if heap.get_task(&NumOrStr::Str(task_name)).is_some() {
                return Err(HeapError::TaskAlreadyExists(format!(
                    "{heap_name}.{task_name}"
                )));
            }
            heap.push(build_task(
                task_name,
                heap_name,
                &mut options_iter,
                &command,
            ));
        }
        PopTask(tag_list) => {
            if let Some(task) = active_task.as_ref() {
                return Err(HeapError::ATaskIsAlreadyInProgress(
                    get_task_from_id(heapmap, task)?.get_full_name(),
                ));
            }
            match pop_task(tag_list, heapmap) {
                Ok(task) => {
                    active_task.replace((task.get_heap_name().into(), task.get_name().into()));
                }
                Err(e) => {
                    let _ = active_task.take();
                    return Err(e);
                }
            };
        }
        InsertTask((ref heap_name, ref task_name, index)) => {
            let heap = get_heap_from_arg(heap_name, heapmap)?;
            if heap.get_task(&NumOrStr::Str(task_name)).is_some() {
                return Err(HeapError::TaskAlreadyExists(format!(
                    "{heap_name}.{task_name}"
                )));
            }
            heap.insert_task(
                build_task(task_name, heap_name, &mut options_iter, &command),
                index,
            );
        }
        RemoveTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            let index = {
                let (idx, task) = get_task_from_arg(heap, &task_idx_name)?;
                print!("Are you sure you want to delete {}?", task.get_full_name());
                let input = get_yes_no()?;
                if input.to_lowercase() == "y" {
                    println!("Task deleted.");
                } else {
                    return Err(HeapError::UserSaidNo);
                }
                let _ = active_task.take_if(|t| format!("{}.{}", t.0, t.1) == task.get_full_name());
                idx
            };
            heap.remove_task(index);
        }
        Edit((ref heap_name, ref task_idx_name_opt)) => {
            let heap = get_heap_from_arg(heap_name, heapmap)?;
            if let Some(task_idx_name) = task_idx_name_opt {
                let (_, task) = get_task_from_arg(heap, task_idx_name)?;
                edit_task(task, &mut options_iter, &command);
            } else {
                edit_heap(heap, &mut options_iter, &command);
            }
        }
        FinishTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.finish();
            let _ = active_task.take_if(|t| format!("{}.{}", t.0, t.1) == task.get_full_name());
            print_tasks_standalone(vec![task], false);
        }
        StartTask((heap_name, task_idx_name)) => {
            if let Some(task) = active_task.as_ref() {
                return Err(HeapError::ATaskIsAlreadyInProgress(
                    get_task_from_id(heapmap, task)?.get_full_name(),
                ));
            }
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.in_progress();
            active_task.replace((task.get_heap_name().into(), task.get_name().into()));
            print_tasks_standalone(vec![task], false);
        }
        StageTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.stage();
            let _ = active_task.take_if(|t| format!("{}.{}", t.0, t.1) == task.get_full_name());
            print_tasks_standalone(vec![task], false);
        }
        UnstageTask((heap_name, task_idx_name)) => {
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            let (_, task) = get_task_from_arg(heap, &task_idx_name)?;
            task.unstage();
            let _ = active_task.take_if(|t| format!("{}.{}", t.0, t.1) == task.get_full_name());
            print_tasks_standalone(vec![task], false);
        }
        StagedTasks => {
            let staged_tasks: Vec<&Task> = heapmap
                .values()
                .filter(|heap| !heap.is_empty())
                .flat_map(|heap| heap.get_staged_tasks())
                .collect();
            print_tasks_standalone(staged_tasks, true);
        }
        CurrentTasks => {
            let current_tasks: Vec<&Task> = heapmap
                .values()
                .filter(|heap| !heap.is_empty())
                .flat_map(|heap| heap.get_current_tasks())
                .collect();
            print_tasks_standalone(current_tasks, true);
        }
        CompleteCurrent => {
            let Some(task) = active_task.take() else {
                return Err(HeapError::NoTaskInProgress);
            };
            let task = get_task_from_id(heapmap, &task)?;
            task.finish();
            println!("Completed {}", task.get_full_name())
        }
        ClearDone(heap_name) => {
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            print!("Are you sure you want to delete all done tasks in {heap_name}?");
            let input = get_yes_no()?;
            if input.to_lowercase() == "y" {
                println!("Tasks deleted.");
            } else {
                return Err(HeapError::UserSaidNo);
            }
            heap.clear_done();
        }
        ClearAllTasks(heap_name) => {
            let heap = get_heap_from_arg(&heap_name, heapmap)?;
            print!("Are you sure you want to delete all tasks in {heap_name}?");
            let input = get_yes_no()?;
            if input.to_lowercase() == "y" {
                println!("Tasks deleted.");
            } else {
                return Err(HeapError::UserSaidNo);
            }
            heap.clear_all();
        }
        List(heap_name_opt) => {
            //Either list all heaps with their tasks or a single heap
            let tags = match options_iter.next().unwrap_or(Tags(vec![])) {
                Tags(t) => t,
                _ => vec![],
            };
            if let Some(heap_name) = heap_name_opt {
                let heap = get_heap_from_arg(&heap_name, heapmap)?;
                print_single_heap(heap, false);
            } else {
                print_all_tasks(heapmap, true, &tags);
            }
        }
        FlatList => {
            //Either list all heaps with their tasks or a single heap
            let tags = match options_iter.next().unwrap_or(Tags(vec![])) {
                Tags(t) => t,
                _ => vec![],
            };
            print_all_tasks_flat(heapmap, false, &tags);
        }
        Heaps => {
            //Print only heap headers and staged
            let tags = match options_iter.next().unwrap_or(Tags(vec![])) {
                Tags(t) => t,
                _ => vec![],
            };
            print_heaps_only(heapmap, &tags);
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
    Ok(())
}
