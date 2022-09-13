use std::{
    cell::Cell,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{
    set_budget, unbox_any_budget, AnyBudget, BudgetGuard, BudgetResult, Budgetable,
    ReplenishableBudget, FUTURE_WAKER,
};

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
pub fn run_with_budget<Budget: Budgetable, F: Future>(
    future: F,
    initial_budget: Budget,
) -> Progress<Budget, F> {
    let initial_budget = Box::new(Some(initial_budget));
    let budget_guard = set_budget(initial_budget);

    let waker = budget_waker::for_current_thread();
    execute_future(Box::pin(future), waker, budget_guard)
}

fn execute_future<Budget: Budgetable, F: Future>(
    mut future: Pin<Box<F>>,
    waker: Waker,
    mut budget: BudgetGuard,
) -> Progress<Budget, F> {
    let mut pinned_future = Pin::new(&mut future);
    let mut cx = Context::from_waker(&waker);
    loop {
        let poll_result = pinned_future.poll(&mut cx);
        let balance = budget.take_budget();
        balance.remove_waker(cx.waker());
        let ran_out_of_budget = FUTURE_WAKER.with(Cell::take).is_some();

        if let Poll::Ready(output) = poll_result {
            return Progress::Complete(BudgetResult {
                output,
                balance: unbox_any_budget(balance),
            });
        }

        if ran_out_of_budget {
            return Progress::NoBudget(IncompleteFuture {
                future,
                waker,
                balance,
                _budget: PhantomData,
            });
        }

        balance.add_waker(cx.waker());
        budget = set_budget(balance);
        pinned_future = Pin::new(&mut future);
        std::thread::park();
    }
}

/// A future that was budgeted with [`run_with_budget()`] that has not yet
/// completed.
pub struct IncompleteFuture<Budget, F>
where
    F: Future,
{
    future: Pin<Box<F>>,
    waker: Waker,
    balance: Box<dyn AnyBudget>,
    _budget: PhantomData<Budget>,
}

impl<Budget, F> IncompleteFuture<Budget, F>
where
    F: Future,
    Budget: Budgetable,
{
    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    pub fn continue_with_additional_budget(self, additional_budget: usize) -> Progress<Budget, F> {
        let Self {
            future,
            waker,
            mut balance,
            ..
        } = self;
        balance.replenish(additional_budget);
        let budget_guard = set_budget(balance);

        execute_future(future, waker, budget_guard)
    }
}

impl<F> IncompleteFuture<ReplenishableBudget, F>
where
    F: Future,
{
    /// Waits for additional budget to be allocated through
    /// [`ReplenishableBudget::replenish()`].
    pub fn wait_for_budget(self) -> Progress<ReplenishableBudget, F> {
        let Self {
            future,
            waker,
            balance,
            ..
        } = self;

        balance.add_waker(&waker);
        std::thread::park();
        balance.remove_waker(&waker);

        let budget_guard = set_budget(balance);

        execute_future(future, waker, budget_guard)
    }
}

/// The progress of a future's execution.
#[must_use]
pub enum Progress<Budget, F: Future> {
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteFuture<Budget, F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output, Budget>),
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
