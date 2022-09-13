use std::{
    cell::Cell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{compute_overage, set_budget, Budget, BudgetResult, FUTURE_WAKER};

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
pub fn run_with_budget<F: Future>(future: F, initial_budget: usize) -> BudgetedFuture<F> {
    BudgetedFuture {
        future: Some(Box::pin(future)),
        budget: Budget::Remaining(initial_budget),
    }
}

/// Executes a future with a given budget when awaited.
///
/// This future is returned from [`run_with_budget()`].
#[must_use = "the future must be awaited to be executed"]
pub struct BudgetedFuture<F> {
    future: Option<Pin<Box<F>>>,
    budget: Budget,
}

impl<F> Future for BudgetedFuture<F>
where
    F: Future,
{
    type Output = Progress<F>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match poll_async_future_with_budget(
            self.future
                .take()
                .expect("poll called after future was complete"),
            cx,
            self.budget,
        ) {
            BudgetPoll::Ready(result) => Poll::Ready(result),
            BudgetPoll::Pending { future, balance } => {
                self.future = Some(future);
                self.budget = balance;
                Poll::Pending
            }
        }
    }
}

enum BudgetPoll<F>
where
    F: Future,
{
    Ready(Progress<F>),
    Pending {
        future: Pin<Box<F>>,
        balance: Budget,
    },
}

fn poll_async_future_with_budget<F: Future>(
    mut future: Pin<Box<F>>,
    cx: &mut Context<'_>,
    current_budget: Budget,
) -> BudgetPoll<F> {
    let budget_guard = set_budget(current_budget);
    FUTURE_WAKER.with(|waker| waker.set(None));

    let pinned_future = Pin::new(&mut future);
    let future_result = pinned_future.poll(cx);
    let balance = budget_guard.take_budget();

    match future_result {
        Poll::Ready(output) => {
            BudgetPoll::Ready(Progress::Complete(BudgetResult { output, balance }))
        }
        Poll::Pending => {
            let waker = FUTURE_WAKER.with(Cell::take);
            if let Some(waker) = waker {
                BudgetPoll::Ready(Progress::NoBudget(IncompleteAsyncFuture {
                    future,
                    waker,
                    remaining_budget: match balance {
                        Budget::Remaining(budget) => budget,
                        Budget::Overage(_) => unreachable!("invalid state"),
                    },
                }))
            } else {
                BudgetPoll::Pending { future, balance }
            }
        }
    }
}

/// The progress of a future's execution.
pub enum Progress<F: Future> {
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteAsyncFuture<F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output>),
}

/// A future that was budgeted using [`run_with_budget()`] that has
/// not yet completed.
pub struct IncompleteAsyncFuture<F>
where
    F: Future,
{
    future: Pin<Box<F>>,
    waker: Waker,
    remaining_budget: usize,
}

impl<F> IncompleteAsyncFuture<F>
where
    F: Future,
{
    /// When awaited, continues executing the provided future until it completes. Once
    /// completed, the future's output and the final budget will be returned.
    ///
    /// This function returns a future that must be awaited for anything to happen.
    pub fn continue_to_completion(self) -> ContinueToCompletion<F> {
        let Self {
            future,
            waker,
            remaining_budget,
        } = self;
        waker.wake();
        ContinueToCompletion {
            future: Some(future),
            remaining_budget,
        }
    }

    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    ///
    /// This function returns a future that must be awaited for anything to happen.
    pub fn continue_with_additional_budget(self, additional_budget: usize) -> BudgetedFuture<F> {
        let Self {
            future,
            waker,
            remaining_budget,
        } = self;
        waker.wake();
        let budget = remaining_budget.saturating_add(additional_budget);

        BudgetedFuture {
            future: Some(future),
            budget: Budget::Remaining(budget),
        }
    }
}

/// A future that continues executing a budgeted future until it completes,
/// tracking any budget overages incurred while doing so.
///
/// This future is returned from
/// [`IncompleteAsyncFuture::continue_to_completion()`].
#[must_use = "the future must be awaited to be executed"]
pub struct ContinueToCompletion<F> {
    future: Option<Pin<Box<F>>>,
    remaining_budget: usize,
}

impl<F> Future for ContinueToCompletion<F>
where
    F: Future,
{
    type Output = (F::Output, Budget);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match poll_async_future_with_budget(
            self.future.take().expect("polled after completion"),
            cx,
            Budget::Overage(0),
        ) {
            BudgetPoll::Ready(Progress::Complete(BudgetResult { output, balance })) => {
                Poll::Ready((output, compute_overage(self.remaining_budget, balance)))
            }
            BudgetPoll::Ready(Progress::NoBudget(_)) => {
                unreachable!("impossible in overage mode")
            }
            BudgetPoll::Pending { future, balance } => {
                self.future = Some(future);
                self.remaining_budget = match balance {
                    Budget::Remaining(balance) => balance,
                    Budget::Overage(_) => unreachable!("invalid state"),
                };
                Poll::Pending
            }
        }
    }
}
