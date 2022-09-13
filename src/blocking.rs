use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{compute_overage, set_budget, Budget, BudgetGuard, BudgetResult, FUTURE_WAKER};

/// Executes `future` with the provided budget. The future will run until it
/// completes or until it has invoked [`spend()`](crate::spend) enough to
/// exhaust the budget provided. If the future never called
/// [`spend()`](crate::spend), this function will block the current thread until
/// the future completes.
///
/// This method should not be used within async methods. For running a future
/// inside of an async context, use
/// [`asynchronous::run_with_budget()`](crate::asynchronous::run_with_budget).
///
/// # Panics
///
/// Panics when called within from within `future` or any code invoked by
/// `future`.
pub fn run_with_budget<F: Future>(future: F, initial_budget: usize) -> Progress<F> {
    let budget_guard = set_budget(Budget::Remaining(initial_budget));

    let waker = budget_waker::for_current_thread();
    execute_future(Box::pin(future), waker, budget_guard)
}

fn execute_future<F: Future>(
    mut future: Pin<Box<F>>,
    waker: Waker,
    mut budget: BudgetGuard,
) -> Progress<F> {
    let mut pinned_future = Pin::new(&mut future);
    let mut cx = Context::from_waker(&waker);
    loop {
        let poll_result = pinned_future.poll(&mut cx);
        let balance = budget.take_budget();
        let ran_out_of_budget = FUTURE_WAKER.with(Cell::take).is_some();

        if let Poll::Ready(output) = poll_result {
            return Progress::Complete(BudgetResult { output, balance });
        }

        if ran_out_of_budget {
            return Progress::NoBudget(IncompleteFuture {
                future,
                waker,
                remaining_budget: match balance {
                    Budget::Remaining(remaining) => remaining,
                    Budget::Overage(_) => unreachable!("invalid state"),
                },
            });
        }

        // TODO need to re-establish guard
        budget = set_budget(balance);
        pinned_future = Pin::new(&mut future);
        std::thread::park();
    }
}

/// A future that was budgeted with [`run_with_budget()`] that has not yet
/// completed.
pub struct IncompleteFuture<F>
where
    F: Future,
{
    future: Pin<Box<F>>,
    waker: Waker,
    remaining_budget: usize,
}

impl<F> IncompleteFuture<F>
where
    F: Future,
{
    /// Continues executing the provided future until it completes. Once
    /// completed, the future's output and the final budget will be returned.
    #[must_use]
    pub fn continue_to_completion(self) -> (F::Output, Budget) {
        let Self {
            future,
            waker,
            remaining_budget,
        } = self;
        let budget_guard = set_budget(Budget::Overage(0));

        match execute_future(future, waker, budget_guard) {
            Progress::Complete(BudgetResult { output, balance }) => {
                (output, compute_overage(remaining_budget, balance))
            }
            Progress::NoBudget(_) => {
                unreachable!("impossible in overage mode")
            }
        }
    }

    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    pub fn continue_with_additional_budget(self, additional_budget: usize) -> Progress<F> {
        let Self {
            future,
            waker,
            remaining_budget,
        } = self;
        let budget_guard = set_budget(Budget::Remaining(
            remaining_budget.saturating_add(additional_budget),
        ));

        execute_future(future, waker, budget_guard)
    }
}

/// The progress of a future's execution.
#[must_use]
pub enum Progress<F: Future> {
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteFuture<F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output>),
}

mod budget_waker {
    use std::{
        sync::Arc,
        task::{RawWaker, RawWakerVTable, Waker},
        thread::Thread,
    };

    pub fn for_current_thread() -> Waker {
        let arc_thread = Arc::new(std::thread::current());
        let arc_thread = Arc::into_raw(arc_thread);

        unsafe { Waker::from_raw(RawWaker::new(arc_thread.cast::<()>(), &BUDGET_WAKER_VTABLE)) }
    }

    unsafe fn clone(arc_thread: *const ()) -> RawWaker {
        let arc_thread: Arc<Thread> = Arc::from_raw(arc_thread.cast::<std::thread::Thread>());
        let cloned = arc_thread.clone();
        let cloned = Arc::into_raw(cloned);

        let _ = Arc::into_raw(arc_thread);

        RawWaker::new(cloned.cast::<()>(), &BUDGET_WAKER_VTABLE)
    }

    unsafe fn wake_consuming(arc_thread: *const ()) {
        let arc_thread: Arc<Thread> = Arc::from_raw(arc_thread as *mut Thread);
        arc_thread.unpark();
    }

    unsafe fn wake_by_ref(arc_thread: *const ()) {
        let arc_thread: Arc<Thread> = Arc::from_raw(arc_thread as *mut Thread);
        arc_thread.unpark();

        let _ = Arc::into_raw(arc_thread);
    }

    unsafe fn drop_waker(arc_thread: *const ()) {
        let arc_thread: Arc<Thread> = Arc::from_raw(arc_thread as *mut Thread);
        drop(arc_thread);
    }

    const BUDGET_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone, wake_consuming, wake_by_ref, drop_waker);
}
