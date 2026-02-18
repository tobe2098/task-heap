# task-heap

`task-heap` is a command-line task management tool written in Rust designed to help you prioritize and select tasks using a weighted probabilistic system. Unlike standard todo lists that are static, `task-heap` allows you to assign weights to task stacks (categories) and individual tasks to randomize your workflow based on priority.

It features a "commit" mechanic (Chicken vs. Penguin) to encourage accountability when a task is selected.

## Installation

To install the package from the tarball:
```bash
cargo install --path .
```

To install directly:
```bash
cargo install task-heap
```

## Core Concepts

### Stacks and Weights

A **task stack** is a stack of tasks (like a project or topic). Both Stacks and Tasks have **Weights**.

* **Stack Weight:** Determines how likely the tool is to select a task from this category.
* **Task Weight:** Determines how likely a specific task is to be chosen compared to others in the same stack.

### The Workflow

1. **Push** tasks into Task Stacks.
2. **Stage** the tasks you are willing to do now.
3. **Pop** a task. The tool rolls the dice (respecting your weights) and presents a task.
4. **Commit:** You must confirm if you will do the task (Penguin) or back out (Chicken).

## Usage

### Stack Management

Commands for managing task categories.

* `create <stack_name> [options]`
* Creates a new task stack.
* *Options:* `Weight`, `Tags`.


* `destroy <stack_name>`
* Deletes a stack and all its contents (requires confirmation).


* `stacks [options]`
* Lists all available stacks.
* *Options:* Filter by `Tags`.


* `edit <stack_name> [options]`
* Modifies stack properties.
* *Options:* `Weight`, `Tags`, `Untag`.



### Task Management

Commands for adding and modifying individual tasks.

* `push <stack_name>.<task_name|index> [options]`
* Adds a new task to a specific stack.
* *Options:* `Description`, `Weight`.


* `insert <stack_name>.<index> <task_name>  [options]`
* Inserts a task at a specific position in the list.


* `remove <stack_name>.<task_name|index>`
* Deletes a specific task.


* `edit <stack_name>.<task_name|index> [options]`
* Edits an existing task.
* *Options:* `Name`, `Description`, `Weight`.



### Workflow & Execution

Commands to progress through your work.

* `stage <stack_name>.<task_name|index>`
* Marks an idle task as "Staged" (ready to be picked).


* `pop [tags]`
* Randomly selects a task from your staged tasks based on weights. If a stack has no staged tasks, the first task in the stack is considered as staged.
* *Note:* Can filter candidates by Tags.


* `start <stack_name>.<task_name|index>`
* Manually marks a specific task as "In Progress" (bypassing the random pop). Only one task can be "In Progress"!


* `finish <stack_name>.<task_name|index>`
* Marks a specific task as Done.


* `complete`
* Marks the currently active task as Done.


* `current`
* Displays the task currently in progress.


* `staged`
* Lists all tasks currently staged or considered staged across all stacks.


* `reset <stack_name>.<task_name|index>`
* Resets a task's status back to Idle.



### Maintenance & Views

* `list [stack_name]`
* Lists tasks. If a stack is specified, lists only that stack. Otherwise, lists all.


* `tasks`
* Prints a list of all tasks across all stacks.


* `clear-done <stack_name>`
* Removes all completed tasks from a stack.


* `clear-all <stack_name>`
* Removes **all** tasks from a stack.



## Options & Flags

When running commands, you can use the following qualifiers:

* **Weight:** Sets the probability weight (e.g., `-w 10`).
* **Tags:** Adds tags for filtering (e.g., `--tags work`, `--tags work,urgent`).
* **Untag:** Removes tags (e.g., `--untag work`).
* **Description:** Adds a text description to a task (e.g., `-d "Fix the login bug"`).

## Examples

### Creating Stacks and Tasks

[Examples here]

### working with Weights

[Examples here]

### The Pop Workflow

[Examples here]

## License

[GPL](./LICENSE)
