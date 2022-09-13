use std::{
    cell::Cell,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{
    set_budget, unbox_any_budget, AnyBudget, BudgetResult, Budgetable, ReplenishableBudget,
    FUTURE_WAKER,
};

/// Executes `future` with the provided budget. The future will run until it
/// completes or until it has invoked [`spend()`](crate::spend) enough to
/// exhaust the budget provided. If the future never called
/// [`spend()`](crate::spend), this function will not return until the future
/// has completed.
///
/// This function returns a [`Future`] which must be awaited to execute the
/// function.
///
/// This implementation is runtime agnostic.
///
/// # Panics
///
/// Panics when called within from within `future` or any code invoked by
/// `future`.
pub fn run_with_budget<Budget: Budgetable, F: Future>(
    future: F,
    initial_budget: Budget,
) -> BudgetedFuture<Budget, F> {
    let initial_budget = Box::new(Some(initial_budget));
    BudgetedFuture {
        state: Some(BudgetedFutureState {
            future: Box::pin(future),
            balance: initial_budget,
            _budget: PhantomData,
        }),
    }
}

/// Executes a future with a given budget when awaited.
///
/// This future is returned from [`run_with_budget()`].
#[must_use = "the future must be awaited to be executed"]
pub struct BudgetedFuture<B, F> {
    state: Option<BudgetedFutureState<B, F>>,
}

struct BudgetedFutureState<B, F> {
    future: Pin<Box<F>>,
    balance: Box<dyn AnyBudget>,
    _budget: PhantomData<B>,
}

impl<B, F> Future for BudgetedFuture<B, F>
where
    F: Future,
    B: Budgetable,
{
    type Output = Progress<B, F>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self
            .state
            .take()
            .expect("poll called after future was complete");
        match poll_async_future_with_budget(state.future, cx, state.balance) {
            BudgetPoll::Ready(result) => Poll::Ready(result),
            BudgetPoll::Pending { future, balance } => {
                self.state = Some(BudgetedFutureState {
                    future,
                    balance,
                    _budget: PhantomData,
                });
                Poll::Pending
            }
        }
    }
}

enum BudgetPoll<B, F>
where
    F: Future,
    B: Budgetable,
{
    Ready(Progress<B, F>),
    Pending {
        future: Pin<Box<F>>,
        balance: Box<dyn AnyBudget>,
    },
}

fn poll_async_future_with_budget<B: Budgetable, F: Future>(
    mut future: Pin<Box<F>>,
    cx: &mut Context<'_>,
    current_budget: Box<dyn AnyBudget>,
) -> BudgetPoll<B, F> {
    let budget_guard = set_budget(current_budget);
    FUTURE_WAKER.with(|waker| waker.set(None));

    let pinned_future = Pin::new(&mut future);
    let future_result = pinned_future.poll(cx);
    let balance = budget_guard.take_budget();

    match future_result {
        Poll::Ready(output) => BudgetPoll::Ready(Progress::Complete(BudgetResult {
            output,
            balance: unbox_any_budget(balance),
        })),
        Poll::Pending => {
            let waker = FUTURE_WAKER.with(Cell::take);
            if let Some(waker) = waker {
                BudgetPoll::Ready(Progress::NoBudget(IncompleteAsyncFuture {
                    future,
                    waker,
                    balance,
                    _budget: PhantomData,
                }))
            } else {
                BudgetPoll::Pending { future, balance }
            }
        }
    }
}

/// The progress of a future's execution.
pub enum Progress<Budget: Budgetable, F: Future> {
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteAsyncFuture<Budget, F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output, Budget>),
}

/// A future that was budgeted using [`run_with_budget()`] that has
/// not yet completed.
pub struct IncompleteAsyncFuture<B, F>
where
    F: Future,
{
    future: Pin<Box<F>>,
    waker: Waker,
    balance: Box<dyn AnyBudget>,
    _budget: PhantomData<B>,
}

impl<B, F> IncompleteAsyncFuture<B, F>
where
    F: Future,
    B: Budgetable,
{
    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    ///
    /// This function returns a future that must be awaited for anything to happen.
    pub fn continue_with_additional_budget(self, additional_budget: usize) -> BudgetedFuture<B, F> {
        let Self {
            future,
            waker,
            mut balance,
            ..
        } = self;
        waker.wake();
        balance.replenish(additional_budget);

        BudgetedFuture {
            state: Some(BudgetedFutureState {
                future,
                balance,
                _budget: PhantomData,
            }),
        }
    }
}

impl<F> IncompleteAsyncFuture<ReplenishableBudget, F>
where
    F: Future,
{
    /// Waits for additional budget to be allocated through
    /// [`ReplenishableBudget::replenish()`].
    pub fn wait_for_budget(self) -> WaitForBudgetFuture<ReplenishableBudget, F> {
        let Self {
            future,
            waker,
            balance,
            ..
        } = self;
        WaitForBudgetFuture {
            has_returned_pending: false,
            waker: Some(waker),
            future: BudgetedFuture {
                state: Some(BudgetedFutureState {
                    future,
                    balance,
                    _budget: PhantomData,
                }),
            },
        }
    }
}

/// A future that waits for additional budget to be allocated through
/// [`ReplenishableBudget::replenish()`].
///
/// This must be awaited to be executed.
#[must_use = "the future must be awaited to be executed"]
pub struct WaitForBudgetFuture<B, F>
where
    F: Future,
{
    has_returned_pending: bool,
    waker: Option<Waker>,
    future: BudgetedFuture<B, F>,
}

impl<B, F> Future for WaitForBudgetFuture<B, F>
where
    F: Future,
    B: Budgetable,
{
    type Output = Progress<B, F>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.has_returned_pending {
            if let Some(waker) = self.waker.take() {
                waker.wake();
            }

            let state = self
                .future
                .state
                .take()
                .expect("poll called after future was complete");
            match poll_async_future_with_budget(state.future, cx, state.balance) {
                BudgetPoll::Ready(result) => Poll::Ready(result),
                BudgetPoll::Pending { future, balance } => {
                    self.future.state = Some(BudgetedFutureState {
                        future,
                        balance,
                        _budget: PhantomData,
                    });
                    Poll::Pending
                }
            }
        } else {
            self.has_returned_pending = true;
            let state = self.future.state.as_ref().expect("always present");
            state.balance.add_waker(cx.waker());
            Poll::Pending
        }
    }
}
