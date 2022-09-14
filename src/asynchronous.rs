use std::{
    cell::RefCell,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use crate::{BudgetContext, BudgetContextData, BudgetResult, Budgetable, ReplenishableBudget};

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
    future: impl FnOnce(BudgetContext<Budget>) -> F,
    initial_budget: Budget,
) -> BudgetedFuture<Budget, F> {
    let context = BudgetContext {
        data: Rc::new(BudgetContextData {
            budget: RefCell::new(initial_budget),
            future_waker: RefCell::new(None),
        }),
    };
    BudgetedFuture {
        state: Some(BudgetedFutureState {
            future: Box::pin(future(context.clone())),
            context,
            _budget: PhantomData,
        }),
    }
}

/// Executes a future with a given budget when awaited.
///
/// This future is returned from [`run_with_budget()`].
#[must_use = "the future must be awaited to be executed"]
pub struct BudgetedFuture<Budget, F> {
    state: Option<BudgetedFutureState<Budget, F>>,
}

struct BudgetedFutureState<Budget, F> {
    future: Pin<Box<F>>,
    context: BudgetContext<Budget>,
    _budget: PhantomData<Budget>,
}

impl<Budget, F> Future for BudgetedFuture<Budget, F>
where
    F: Future,
    Budget: Budgetable,
{
    type Output = Progress<Budget, F>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self
            .state
            .take()
            .expect("poll called after future was complete");
        match poll_async_future_with_budget(state.future, cx, state.context) {
            BudgetPoll::Ready(result) => Poll::Ready(result),
            BudgetPoll::Pending { future, context } => {
                self.state = Some(BudgetedFutureState {
                    future,
                    context,
                    _budget: PhantomData,
                });
                Poll::Pending
            }
        }
    }
}

enum BudgetPoll<Budget, F>
where
    F: Future,
    Budget: Budgetable,
{
    Ready(Progress<Budget, F>),
    Pending {
        future: Pin<Box<F>>,
        context: BudgetContext<Budget>,
    },
}

fn poll_async_future_with_budget<Budget: Budgetable, F: Future>(
    mut future: Pin<Box<F>>,
    cx: &mut Context<'_>,
    budget_context: BudgetContext<Budget>,
) -> BudgetPoll<Budget, F> {
    *budget_context.data.future_waker.borrow_mut() = None;

    let pinned_future = Pin::new(&mut future);
    let future_result = pinned_future.poll(cx);

    match future_result {
        Poll::Ready(output) => BudgetPoll::Ready(Progress::Complete(BudgetResult {
            output,
            balance: budget_context.data.budget.borrow().clone(),
        })),
        Poll::Pending => {
            let waker = budget_context.data.future_waker.take();
            if let Some(waker) = waker {
                BudgetPoll::Ready(Progress::NoBudget(IncompleteAsyncFuture {
                    future,
                    waker,
                    context: budget_context,
                }))
            } else {
                BudgetPoll::Pending {
                    future,
                    context: budget_context,
                }
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
pub struct IncompleteAsyncFuture<Budget, F>
where
    F: Future,
{
    future: Pin<Box<F>>,
    waker: Waker,
    context: BudgetContext<Budget>,
}

impl<Budget, F> IncompleteAsyncFuture<Budget, F>
where
    F: Future,
    Budget: Budgetable,
{
    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    ///
    /// This function returns a future that must be awaited for anything to happen.
    pub fn continue_with_additional_budget(
        self,
        additional_budget: usize,
    ) -> BudgetedFuture<Budget, F> {
        let Self {
            future,
            waker,
            context,
            ..
        } = self;
        waker.wake();
        context
            .data
            .budget
            .borrow_mut()
            .replenish(additional_budget);

        BudgetedFuture {
            state: Some(BudgetedFutureState {
                future,
                context,
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
            context,
            ..
        } = self;
        WaitForBudgetFuture {
            has_returned_pending: false,
            waker: Some(waker),
            future: BudgetedFuture {
                state: Some(BudgetedFutureState {
                    future,
                    context,
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
pub struct WaitForBudgetFuture<Budget, F>
where
    F: Future,
{
    has_returned_pending: bool,
    waker: Option<Waker>,
    future: BudgetedFuture<Budget, F>,
}

impl<Budget, F> Future for WaitForBudgetFuture<Budget, F>
where
    F: Future,
    Budget: Budgetable,
{
    type Output = Progress<Budget, F>;

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
            match poll_async_future_with_budget(state.future, cx, state.context) {
                BudgetPoll::Ready(result) => Poll::Ready(result),
                BudgetPoll::Pending { future, context } => {
                    self.future.state = Some(BudgetedFutureState {
                        future,
                        context,
                        _budget: PhantomData,
                    });
                    Poll::Pending
                }
            }
        } else {
            self.has_returned_pending = true;
            let state = self.future.state.as_ref().expect("always present");
            state.context.data.budget.borrow_mut().add_waker(cx.waker());
            Poll::Pending
        }
    }
}
