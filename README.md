# budget-executor

An approach to "throttling" async tasks in Rust using manual instrumentation.

[![crate version](https://img.shields.io/crates/v/budget-executor.svg)](https://crates.io/crates/budget-executor)
[![Live Build Status](https://img.shields.io/github/workflow/status/khonsulabs/budget-executor/Tests/main)](https://github.com/khonsulabs/budget-executor/actions?query=workflow:Tests)
[![HTML Coverage Report for `main` branch](https://khonsulabs.github.io/budget-executor/coverage/badge.svg)](https://khonsulabs.github.io/budget-executor/coverage/)
[![Documentation](https://img.shields.io/badge/docs-main-informational)](https://khonsulabs.github.io/budget-executor/main/budget_executor)

This crate implements a manual task throttling approach using a simple `usize` to
track available budget. Both leverage Rust's approach to async tasks to enable
pausing the task when it requests to spend more budget than is remaining.

One use-case/inspiration for this crate is a scripting language built in Rust.
By defining parts of the interpeter using `async` functions, untrusted scripts
can be executed with this crate with limited budget to protect against infinite
loops or too-intensive scripts slowing down a server. The interpreter can assign
different costs to various operations.

This crate has two implementations of this approach: one that blocks the current
thread and one that works with any async runtime.

## Using from non-async code (Blocking)

This example of the [blocking](https://khonsulabs.github.io/budget-executor/main/budget_executor/blocking/index.html) implementation is from
`examples/simple.rs` in the repository:

```rust
use std::time::Duration;

use budget_executor::blocking::{run_with_budget, Progress};

fn main() {
    // Run a task with no initial budget. The first time the task asks to spend
    // any budget, it will be paused.
    let mut progress = run_with_budget(some_task_to_limit(), 0);

    // At this point, the task has run until the first call to
    // budget_executor::spend. Because we gave an initial_budget of 0, the future
    // is now paused. Let's loop until it's finished, feeding it 5 budget at a
    // time.
    loop {
        progress = match progress {
            Progress::NoBudget(incomplete_task) => {
                // Resume the task, allowing for 5 more budget to be spent.
                println!("+5 budget");
                incomplete_task.continue_with_additional_budget(5)
            }
            Progress::Complete(result) => {
                // The task has completed. `result.output` contains the output
                // of the task itself. We can also inspect the balance of the
                // budget:
                println!(
                    "Task completed with balance: {:?}, output: {:?}",
                    result.balance, result.output
                );
                break;
            }
        };
    }
}

async fn some_task_to_limit() -> bool {
    do_some_operation(1).await;
    do_some_operation(5).await;
    do_some_operation(1).await;
    do_some_operation(25).await;
    true
}

async fn do_some_operation(times: u8) {
    println!("> Asking to spend {times} from the budget");
    budget_executor::spend(usize::from(times)).await;

    // Despite being async code, because we know we're running in a
    // single-threaded environment, we can still call blocking operations.
    std::thread::sleep(Duration::from_millis(u64::from(times) * 100));
}

```

When run, it produces this output:

```sh
~/p/budget-executor (main)> cargo run --example simple
   Compiling budget-executor v0.1.0 (/home/ecton/projects/budget-executor)
    Finished dev [unoptimized + debuginfo] target(s) in 0.31s
     Running `target/debug/examples/simple`
> Asking to spend 1 from the budget
+5 budget
> Asking to spend 5 from the budget
+5 budget
> Asking to spend 1 from the budget
> Asking to spend 25 from the budget
+5 budget
+5 budget
+5 budget
+5 budget
+5 budget
Task completed with balance: Remaining(3), output: true
```

### How does this work?

At the start of the example, [run_with_budget()](https://khonsulabs.github.io/budget-executor/main/budget_executor/blocking/fn.run_with_budget.html) is called with
an initial balance of 0. This will cause the future (`some_task_to_limit()`) to
execute until it executes [`spend(amount).await`](https://khonsulabs.github.io/budget-executor/main/budget_executor/fn.spend.html). When the future
attempts to spend any budget, because the initial balance was 0, the future will
be paused until budget made available. `run_with_budget()` returns
`Progress::NoBudget` which contains the incomplete task.

The example now loops until `progress` contains `Progress::Complete`. When
`Progress::NoBudget` is returned instead, the task is resumed using
[`continue_with_additional_budget()`](https://khonsulabs.github.io/budget-executor/main/budget_executor/blocking/struct.IncompleteFuture.html#method.continue_with_additional_budget). This resumes
executing the future, which will re-awaken inside of `spend().await`. The budget
will be checked again. If there is enough budget, `spend().await` will deduct
the spent amount and return. If there isn't enough budget, the future will pause again and `continue_with_additional_budget` returns `Progress::NoBudget`.

Upon completion, the remaining balance is returned along with the task's output
in `Progress::Complete`. In this example, the task spends a total of 32. Because
the budget is always allocated in increments of 5, 35 budget was allocated which
left a remaining budget of 3 when the task completed.

### Blocking Implementation Warnings

If you invoke `.await` on anything other than [`spend()`](https://khonsulabs.github.io/budget-executor/main/budget_executor/fn.spend.html), the
executing thread will be parked until that future is completed. This means care
must be taken if you attempt to use the blocking implementation within another
async context: you must ensure that any future awaited by the task will be
completed on a separate thread. If you can't guarantee this, you should use the
asynchronous implementation.

## Using from async code

This example is `examples/simple-async.rs` in the repository:

```rust
use std::time::Duration;

use budget_executor::asynchronous::{run_with_budget, Progress};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Run a task with no initial budget. The first time the task asks to spend
    // any budget, it will be paused.
    let mut progress = run_with_budget(some_task_to_limit(), 0).await;

    // At this point, the task has run until the first call to
    // budget_executor::spend. Because we gave an initial_budget of 0, the future
    // is now paused. Let's loop until it's finished, feeding it 5 budget at a
    // time.
    loop {
        progress = match progress {
            Progress::NoBudget(incomplete_task) => {
                // Resume the task, allowing for 5 more budget to be spent.
                println!("+5 budget");
                incomplete_task.continue_with_additional_budget(5).await
            }
            Progress::Complete(result) => {
                // The task has completed. `result.output` contains the output
                // of the task itself. We can also inspect the balance of the
                // budget:
                println!(
                    "Task completed with balance: {:?}, output: {:?}",
                    result.balance, result.output
                );
                break;
            }
        };
    }
}

async fn some_task_to_limit() -> bool {
    do_some_operation(1).await;
    do_some_operation(5).await;
    do_some_operation(1).await;
    do_some_operation(25).await;
    true
}

async fn do_some_operation(times: u8) {
    println!("> Asking to spend {times} from the budget");
    budget_executor::spend(usize::from(times)).await;
    tokio::time::sleep(Duration::from_millis(u64::from(times) * 100)).await;
}

```

When run, it produces the same output as displayed in the blocking section.

### How does this work?

This example is identical to the blocking example, but instead uses the
[`asynchronous`](https://khonsulabs.github.io/budget-executor/main/budget_executor/asynchronous/index.html) module's APIs: [`run_with_budget().await`](https://khonsulabs.github.io/budget-executor/main/budget_executor/asynchronous/fn.run_with_budget.html)
and [`continue_with_additional_budget().await`](https://khonsulabs.github.io/budget-executor/main/budget_executor/asynchronous/struct.IncompleteFuture.html#method.continue_with_additional_budget).

This implementation is runtime agnostic and is actively tested against tokio.

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), are
open-source. This repository is available under the [MIT License](./LICENSE-MIT)
or the [Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
