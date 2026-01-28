# Task Heap

**Task Heap** is a command-line interface (CLI) task manager written in Rust. Unlike standard to-do lists, Task Heap is designed to cure "analysis paralysis." Instead of manually picking a task, you assign tasks a **weight**, and the program randomly "pops" a task for you to complete based on those weights.

## Features

* **Weighted Random Selection:** Assign importance (weight) to tasks. Higher weight = higher probability of being selected.
* **Gamified Workflow:** The "Pop" mechanic challenges you to complete the task immediately (Penguin) or back out (Chicken).
* **Tagging System:** Organize tasks with tags and filter the "Pop" selection by specific tags.
* **Persistent Storage:** Automatically saves and loads your heap (currently via local file I/O).
* **Detailed Metadata:** Tasks support names, descriptions, weights, and arbitrary tags.

## Installation

Ensure you have [Rust and Cargo installed](https://www.rust-lang.org/tools/install).

1. Clone the repository:
```bash
git clone https://github.com/yourusername/task-heap.git
cd task-heap

```


2. Build the project:
```bash
cargo build --release

```


3. Run the binary:
```bash
./target/release/task-heap --help

```

You can add the binary to a path in PATH or add the folder to PATH to use without relative paths.

---

## Usage

The general syntax for Task Heap is:

```bash
task-heap [ACTION] [OPTIONS/QUALIFIERS]

```

### 1. Adding Tasks (`--push` / `-i`)

Push a new task onto the heap. You can chain qualifiers like description, weight, and tags immediately after the push command.

**Arguments:** `Task Name`

```bash
# Simple push
task-heap -i Clean the Garage

# Detailed push with weight (priority), description, and tags
task-heap -i Finish Rust Project -w 50 -p Write the README and fix bugs -at coding,school

```

### 2. Popping Tasks (`--pop` / `-o`)

This is the core feature. The program selects a task for you.

* **How it works:** It creates a weighted distribution of all tasks.
* **Filtering:** You can limit the selection pool by providing tags.

**The Penguin vs. Chicken Mechanic:**
Once a task is selected, you are prompted:

> *Are you certain you can complete it? Are you a chicken or a penguin?*

* **Yes (y):** You are a **Penguin**. You accept the challenge, and the task is **removed** (completed) from the heap.
* **No (n):** You are a **Chicken**. The task remains in the heap to be picked another day.

```bash
# Pop any task from the heap
task-heap -o

# Pop a task only from the coding category
task-heap -o -at coding

```

### 3. Listing Tasks (`--list` / `-l`)

View all current tasks or filter them by tag.

```bash
# List all
task-heap -l

# List only tasks tagged 'household'
task-heap -l -at household

```

### 4. Editing Tasks (`--edit` / `-e`)

Modify an existing task. You identify the task by its original name, then apply qualifiers to change it.

**Arguments:** `Original Task Name`

```bash
# Change weight and add a tag
task-heap -e Clean the Garage -w 100 -at urgent

# Rename a task
task-heap -e Clean the Garage -n Clean the Entire House

# Remove specific tags
task-heap -e Finish Rust Project -ut school

```

### 5. Deleting Tasks (`--delete` / `-d`)

Permanently remove a task or a group of tasks.

```bash
# Delete a specific task by name
task-heap -d Old Task

# Delete ALL tasks that have a specific tag
task-heap -d -at deprecated

```

### 6. Reset (`--reset` / `-r`)

Deletes **all** tasks from the heap. Requires confirmation.

```bash
task-heap -r

```

### 7. Chain commands!

You can chain commands for ease of use. All operations will be cancelled if there is at least one error, so the task heap's state will never be corrupted.

```
task-heap -i task1 -i task2 -p sth -w 2 -i task3 -o -l
```

---

## Command Reference

| Flag | Long Flag | Description |
| --- | --- | --- |
| **Actions** |  |  |
| `-i` | `--push` | Insert a new task. |
| `-o` | `--pop` | Select a task to do. |
| `-d` | `--delete` | Delete a task or group of tasks. |
| `-e` | `--edit` | Update task details. |
| `-l` | `--list` | Display tasks. |
| `-r` | `--reset` | Wipe the heap. |
| `-ct` | `--clear-tags` | Remove all tags from a specific task. |
| **Qualifiers** |  |  |
| `-n` | `--name` | Specify a new name (used in edit). |
| `-p` | `--description` | Add/Change description. |
| `-w` | `--weight` | Set integer weight (probability). |
| `-at` | `--tag` | Add tags (comma-separated). |
| `-ut` | `--untag` | Remove tags (comma-separated). |

---

## Using different storage locations

By default, `task-heap` stores the task data on a default folder depending on your OS. If you want to override this to use, access or share your task heap, you can define the environment variable TASK_HEAP_DBPATH, for example:
```
TASK_HEAP_DBPATH=$HOME/Dropbox task-heap --push
```
