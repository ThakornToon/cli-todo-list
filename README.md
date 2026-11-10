# Rust Todo List

A command-line todo list application written in Rust. Tasks are stored as JSON in `todos.json` in the directory where the application is run.

## Features

- Add tasks with a title and due date and time.
- List tasks with calendar or date-range filters, status filters, and sorting by due date, status, or title.
- Toggle the completion status of multiple tasks by ID.
- Delete multiple tasks by ID, with a confirmation prompt.
- Edit a task's title or due date and time.
- Renumber task IDs.
- Show notifications for overdue tasks and tasks due within the next 24 hours when the application starts.
- Enter `b` at a prompt to cancel the current operation and return to the main menu.

## Requirements

- Rust and Cargo

## Build and run

From the project directory:

```sh
cargo run
```

## Usage

Follow the numbered menu shown when the application starts. Task due dates and times use the format `YYYY-MM-DD HH:MM` (for example, `2026-11-08 19:30`). Date filters use `YYYY-MM-DD`.

The application creates `todos.json` when tasks are first saved. Keep this file to preserve your task list between runs.
