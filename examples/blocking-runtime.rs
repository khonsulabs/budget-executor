use std::time::Duration;

use budget_executor::{blocking::threadsafe::Runtime, ReplenishableBudget};

const THREADS: usize = 10;
const TASKS_PER_THREAD: usize = 10;
const OPS_PER_TASK: usize = 10;

fn main() {
    let shared_budget = ReplenishableBudget::new(0);

    // Spawn multiple worker threads that all run tasks using a shared budget.
    let mut workers = Vec::new();
    for worker_id in 0..THREADS {
        let shared_budget = shared_budget.clone();
        workers.push(std::thread::spawn(move || worker(worker_id, shared_budget)));
    }

    // trickle-charge the tasks.
    for _ in 0..=THREADS * TASKS_PER_THREAD * OPS_PER_TASK {
        shared_budget.replenish(1);
        std::thread::sleep(Duration::from_millis(1));
    }

    // Wait for the threads to finish
    for worker in workers {
        worker.join().unwrap();
    }
}

fn worker(worker_id: usize, shared_budget: ReplenishableBudget) {
    Runtime::run_with_budget(
        |runtime| async move { some_task_to_limit(worker_id, runtime).await },
        shared_budget,
    )
    .wait_until_complete();
}

async fn some_task_to_limit(worker_id: usize, runtime: Runtime<ReplenishableBudget>) {
    let mut tasks = Vec::new();
    for task_id in 0..TASKS_PER_THREAD {
        let task_runtime = runtime.clone();
        tasks.push(runtime.spawn(async move {
            for op in 0..OPS_PER_TASK {
                task_runtime.spend(1).await;
                println!("Worker {worker_id}, Task {task_id}, Operation {op}");
            }
        }));
    }

    for task in tasks {
        task.await;
    }
}

#[test]
fn runs() {
    main()
}
