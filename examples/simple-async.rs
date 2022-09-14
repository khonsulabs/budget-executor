use std::time::Duration;

use budget_executor::asynchronous::singlethreaded::{Context, Progress};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Run a task with no initial budget. The first time the task asks to spend
    // any budget, it will be paused.
    let mut progress = Context::run_with_budget(some_task_to_limit, 0).await;

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

async fn some_task_to_limit(context: Context<usize>) -> bool {
    do_some_operation(1, &context).await;
    do_some_operation(5, &context).await;
    do_some_operation(1, &context).await;
    do_some_operation(25, &context).await;
    true
}

async fn do_some_operation(times: u8, context: &Context<usize>) {
    println!("> Asking to spend {times} from the budget");
    context.spend(usize::from(times)).await;
    tokio::time::sleep(Duration::from_millis(u64::from(times) * 100)).await;
}

#[test]
fn runs() {
    main()
}
