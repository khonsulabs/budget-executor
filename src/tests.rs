mod blocking {
    use std::{
        cell::Cell,
        rc::Rc,
        time::{Duration, Instant},
    };

    use crate::{
        blocking::{run_with_budget, Progress},
        spend, ReplenishableBudget,
    };

    #[test]
    fn basic() {
        let counter = Rc::new(Cell::new(0));
        let future_counter = counter.clone();
        let future = async move {
            future_counter.set(0);
            spend(1).await;
            future_counter.set(1);
            spend(1).await;
            future_counter.set(2);
        };

        let incomplete = match run_with_budget(future, 0) {
            Progress::NoBudget(incomplete) => {
                assert_eq!(counter.get(), 0);
                incomplete
            }
            Progress::Complete(result) => unreachable!("future completed: {result:?}"),
        };
        let incomplete = match incomplete.continue_with_additional_budget(1) {
            Progress::NoBudget(incomplete) => {
                assert_eq!(counter.get(), 1);
                incomplete
            }
            Progress::Complete(result) => unreachable!("future completed: {result:?}"),
        };
        let result = match incomplete.continue_with_additional_budget(1) {
            Progress::Complete(result) => result,
            Progress::NoBudget(_incomplete) => {
                unreachable!("future didn't complete");
            }
        };
        assert_eq!(result.balance, 0);
    }

    #[test]
    fn external_budget() {
        let budget = ReplenishableBudget::default();
        let future = async move {
            for _ in 0..100 {
                println!("F> Spend 1");
                spend(1).await;
            }
            println!("Done");
        };

        let thread_budget = budget.clone();
        std::thread::spawn(move || {
            for _ in 0..100 {
                println!("T> Replenish 1");
                thread_budget.replenish(1);
                std::thread::sleep(Duration::from_micros(1));
            }
            println!("T> Done");
        });

        let mut incomplete = match run_with_budget(future, budget) {
            Progress::NoBudget(incomplete) => incomplete,
            Progress::Complete(result) => unreachable!("future completed: {result:?}"),
        };

        while let Progress::NoBudget(new_incomplete_task) = incomplete.wait_for_budget() {
            println!("M> Waiting to complete");
            incomplete = new_incomplete_task;
        }
    }

    #[test]
    fn non_budget_parking() {
        // This test uses flume's bounded channel mixed with a thread sleep to cause
        // the async task to wait if it sends messages too quickly. This means this
        // test should take the (sleep_duration * number_of_iterations - 1) time to
        // complete. As written currently, this is 1.4 seconds with an assertion of
        // greater than 1s.
        let (sender, receiver) = flume::bounded(1);
        std::thread::spawn(move || {
            while let Ok(message) = receiver.recv() {
                println!("R: received {message}");
                std::thread::sleep(Duration::from_millis(100));
                println!("R: done 'processing'");
            }
        });

        let task = async move {
            for message in 0..=15 {
                println!("S: requesting budget");
                spend(1).await;
                println!("S: sending {message}");
                sender.send_async(message).await.unwrap();
                println!("S: message sent");
            }
        };

        let started_at = Instant::now();
        let mut progress = run_with_budget(task, 0);
        while let Progress::NoBudget(incomplete) = progress {
            println!("E: providing budget");
            progress = incomplete.continue_with_additional_budget(1);
        }
        let elapsed_time = started_at.elapsed();
        assert!(elapsed_time > Duration::from_secs(1));
    }

    #[test]
    #[should_panic]
    fn reentrant_panic() {
        drop(run_with_budget(async { run_with_budget(async {}, 0) }, 0));
    }
}

mod asynchronous {
    use std::time::{Duration, Instant};

    use crate::{
        asynchronous::{run_with_budget, Progress},
        spend, ReplenishableBudget,
    };

    #[tokio::test]
    async fn external_runtime_compatability() {
        // This test uses flume's bounded channel mixed with a thread sleep to cause
        // the async task to wait if it sends messages too quickly. This means this
        // test should take the (sleep_duration * number_of_iterations - 1) time to
        // complete. As written currently, this is 1.4 seconds with an assertion of
        // greater than 1s.
        let (sender, receiver) = flume::bounded(1);
        tokio::task::spawn(async move {
            while let Ok(message) = receiver.recv_async().await {
                println!("R: received {message}");
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("R: done 'processing'");
            }
        });

        let task = async move {
            for message in 0..=15 {
                println!("S: requesting budget");
                spend(1).await;
                println!("S: sending {message}");
                sender.send_async(message).await.unwrap();
                println!("S: message sent");
            }
        };

        let started_at = Instant::now();
        let mut progress = run_with_budget(task, 0).await;
        while let Progress::NoBudget(incomplete) = progress {
            println!("E: providing budget");
            progress = incomplete.continue_with_additional_budget(1).await;
        }
        let elapsed_time = started_at.elapsed();
        assert!(elapsed_time > Duration::from_secs(1));
    }

    #[tokio::test]
    #[should_panic]
    async fn reentrant_panic_async() {
        drop(run_with_budget(async { run_with_budget(async {}, 0).await }, 0).await);
    }

    #[tokio::test]
    async fn external_budget() {
        let budget = ReplenishableBudget::default();
        let future = async move {
            for _ in 0..100 {
                println!("F> Spend 1");
                spend(1).await;
            }
            println!("Done");
        };

        let thread_budget = budget.clone();
        std::thread::spawn(move || {
            for _ in 0..100 {
                println!("T> Replenish 1");
                thread_budget.replenish(1);
                std::thread::sleep(Duration::from_micros(1));
            }
            println!("T> Done");
        });

        let mut incomplete = match run_with_budget(future, budget).await {
            Progress::NoBudget(incomplete) => incomplete,
            Progress::Complete(result) => unreachable!("future completed: {result:?}"),
        };

        while let Progress::NoBudget(new_incomplete_task) = incomplete.wait_for_budget().await {
            println!("M> Waiting to complete");
            incomplete = new_incomplete_task;
        }
    }
}
