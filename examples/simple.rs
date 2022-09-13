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
