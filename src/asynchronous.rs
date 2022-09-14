use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{BudgetContext, BudgetContextData, BudgetResult, Budgetable, Container};

fn run_with_budget<Budget, Backing, F>(
    future: impl FnOnce(BudgetContext<Backing, Budget>) -> F,
    initial_budget: Budget,
) -> BudgetedFuture<Budget, Backing, F>
where
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
    F: Future,
{
    let context = BudgetContext {
        data: Backing::new(BudgetContextData {
            budget: initial_budget,
            future_waker: None,
        }),
        _budget: PhantomData,
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
struct BudgetedFuture<Budget, Backing, F>
where
    Backing: Container<BudgetContextData<Budget>>,
{
    state: Option<BudgetedFutureState<Budget, Backing, F>>,
}

struct BudgetedFutureState<Budget, Backing, F>
where
    Backing: Container<BudgetContextData<Budget>>,
{
    future: Pin<Box<F>>,
    context: BudgetContext<Backing, Budget>,
    _budget: PhantomData<Budget>,
}

impl<Budget, Backing, F> Future for BudgetedFuture<Budget, Backing, F>
where
    F: Future,
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
{
    type Output = Progress<Budget, Backing, F>;

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

enum BudgetPoll<Budget, Backing, F>
where
    F: Future,
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
{
    Ready(Progress<Budget, Backing, F>),
    Pending {
        future: Pin<Box<F>>,
        context: BudgetContext<Backing, Budget>,
    },
}

fn poll_async_future_with_budget<Budget, Backing, F>(
    mut future: Pin<Box<F>>,
    cx: &mut Context<'_>,
    budget_context: BudgetContext<Backing, Budget>,
) -> BudgetPoll<Budget, Backing, F>
where
    Budget: Budgetable,
    F: Future,
    Backing: Container<BudgetContextData<Budget>>,
{
    budget_context.data.map_locked(|data| {
        data.future_waker = None;
    });

    let pinned_future = Pin::new(&mut future);
    let future_result = pinned_future.poll(cx);

    match future_result {
        Poll::Ready(output) => BudgetPoll::Ready(Progress::Complete(BudgetResult {
            output,
            balance: budget_context.data.map_locked(|data| data.budget.clone()),
        })),
        Poll::Pending => {
            let waker = budget_context
                .data
                .map_locked(|data| data.future_waker.take());
            if let Some(waker) = waker {
                BudgetPoll::Ready(Progress::NoBudget(IncompleteFuture {
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
enum Progress<Budget, Backing, F>
where
    Budget: Budgetable,
    F: Future,
    Backing: Container<BudgetContextData<Budget>>,
{
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteFuture<Budget, Backing, F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output, Budget>),
}

/// A future that was budgeted using [`run_with_budget()`] that has
/// not yet completed.
struct IncompleteFuture<Budget, Backing, F>
where
    F: Future,
    Backing: Container<BudgetContextData<Budget>>,
{
    future: Pin<Box<F>>,
    waker: Waker,
    context: BudgetContext<Backing, Budget>,
}

impl<Budget, Backing, F> IncompleteFuture<Budget, Backing, F>
where
    F: Future,
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
{
    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    ///
    /// This function returns a future that must be awaited for anything to happen.
    pub fn continue_with_additional_budget(
        self,
        additional_budget: usize,
    ) -> BudgetedFuture<Budget, Backing, F> {
        let Self {
            future,
            waker,
            context,
            ..
        } = self;
        waker.wake();
        context
            .data
            .map_locked(|data| data.budget.replenish(additional_budget));

        BudgetedFuture {
            state: Some(BudgetedFutureState {
                future,
                context,
                _budget: PhantomData,
            }),
        }
    }
    /// Waits for additional budget to be allocated through
    /// [`ReplenishableBudget::replenish()`].
    pub fn wait_for_budget(self) -> WaitForBudgetFuture<Budget, Backing, F> {
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
struct WaitForBudgetFuture<Budget, Backing, F>
where
    F: Future,
    Backing: Container<BudgetContextData<Budget>>,
{
    has_returned_pending: bool,
    waker: Option<Waker>,
    future: BudgetedFuture<Budget, Backing, F>,
}

impl<Budget, Backing, F> Future for WaitForBudgetFuture<Budget, Backing, F>
where
    F: Future,
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
{
    type Output = Progress<Budget, Backing, F>;

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
            state
                .context
                .data
                .map_locked(|data| data.budget.add_waker(cx.waker()));
            Poll::Pending
        }
    }
}

macro_rules! define_public_interface {
    ($modulename:ident, $backing:ident, $moduledocs:literal) => {
        #[doc = $moduledocs]
        pub mod $modulename {
            use std::{
                future::Future,
                pin::Pin,
                task::{self, Poll},
            };

            use crate::{BudgetResult, Budgetable, spend::$modulename::SpendBudget};

            type Backing<Budget> = crate::$backing<crate::BudgetContextData<Budget>>;
            type BudgetContext<Budget> = crate::BudgetContext<Backing<Budget>, Budget>;

            /// A budget-limited asynchronous context.
            pub struct Context<Budget>(BudgetContext<Budget>)
            where
                Budget: Budgetable;

            impl<Budget> Context<Budget>
            where
                Budget: Budgetable,
            {

                /// Executes `future` with the provided budget. The future will run until it
                /// completes or until it has invoked [`spend()`](Self::spend) enough to
                /// exhaust the budget provided. If the future never called
                /// [`spend()`](Self::spend), this function will not return until the future
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
                pub fn run_with_budget<F>(
                    future: impl FnOnce(Context<Budget>) -> F,
                    initial_budget: Budget,
                ) -> BudgetedFuture<Budget, F>
                where
                    F: Future,
                {
                    BudgetedFuture(super::run_with_budget(
                        |context| future(Context(context)),
                        initial_budget,
                    ))
                }

                /// Retrieves the current budget.
                ///
                /// This function should only be called by code that is guaranteed to be running
                /// by this executor. When called outside of code run by this executor, this function will.
                #[must_use]
                pub fn budget(&self) -> usize {
                    self.0.budget()
                }

                /// Spends `amount` from the curent budget.
                ///
                /// This function returns a future which must be awaited.
                pub fn spend(&self, amount: usize) -> SpendBudget<'_, Budget> {
                    // How do we re-export SpendBudget since it's sahrd with async too. crate-level module?
                    SpendBudget::from(self.0.spend(amount))
                }
            }

            /// Executes a future with a given budget when awaited.
            ///
            /// This future is returned from [`Context::run_with_budget()`].
            #[must_use = "the future must be awaited to be executed"]
            pub struct BudgetedFuture<Budget, F>(super::BudgetedFuture<Budget, Backing<Budget>, F>)
            where
                Budget: Budgetable;

            impl<Budget, F> Future for BudgetedFuture<Budget, F>
            where
                Budget: Budgetable,
                F: Future,
            {
                type Output = Progress<Budget, F>;

                fn poll(
                    mut self: Pin<&mut Self>,
                    cx: &mut task::Context<'_>,
                ) -> Poll<Self::Output> {
                    let inner = Pin::new(&mut self.0);
                    match inner.poll(cx) {
                        Poll::Ready(output) => Poll::Ready(Progress::from(output)),
                        Poll::Pending => Poll::Pending,
                    }
                }
            }
            /// The progress of a future's execution.
            pub enum Progress<Budget, F>
            where
                Budget: Budgetable,
                F: Future,
            {
                /// The future was interrupted because it requested to spend more budget
                /// than was available.
                NoBudget(IncompleteFuture<Budget, F>),
                /// The future has completed.
                Complete(BudgetResult<F::Output, Budget>),
            }

            impl<Budget, F> From<super::Progress<Budget, Backing<Budget>, F>> for Progress<Budget, F>
            where
                Budget: Budgetable,
                F: Future, {
                fn from(progress: super::Progress<Budget, Backing<Budget>, F>) -> Self {
                    match progress {
                        super::Progress::NoBudget(incomplete) => Progress::NoBudget(IncompleteFuture(incomplete)),
                        super::Progress::Complete(result) => Progress::Complete(result)
                    }
                }
            }

            /// A future that was budgeted using [`Context::run_with_budget()`]
            /// that has not yet completed.
            pub struct IncompleteFuture<Budget, F>(
                pub(super) super::IncompleteFuture<Budget, Backing<Budget>, F>,
            )
            where
                F: Future,
                Budget: Budgetable;

            impl<Budget, F> IncompleteFuture<Budget, F>
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
                    BudgetedFuture(self.0.continue_with_additional_budget(additional_budget))
                }

                /// Waits for additional budget to be allocated through
                /// [`ReplenishableBudget::replenish()`](crate::ReplenishableBudget::replenish).
                pub fn wait_for_budget(self) -> WaitForBudgetFuture<Budget, F> {
                    WaitForBudgetFuture(self.0.wait_for_budget())
                }

            }

            /// A future that waits for additional budget to be allocated
            /// through
            /// [`ReplenishableBudget::replenish()`](crate::ReplenishableBudget::replenish).
            ///
            /// This must be awaited to be executed.
            #[must_use = "the future must be awaited to be executed"]
            pub struct WaitForBudgetFuture<Budget, F>(
                pub(super) super::WaitForBudgetFuture<Budget, Backing<Budget>, F>,
            )
            where
                F: Future,
                Budget: Budgetable;

            impl<Budget, F> Future for WaitForBudgetFuture<Budget, F>
            where
                F: Future,
                Budget: Budgetable,
            {
                type Output = Progress<Budget, F>;

                fn poll(
                    mut self: Pin<&mut Self>,
                    cx: &mut task::Context<'_>,
                ) -> Poll<Self::Output> {
                    let inner = Pin::new(&mut self.0);
                    match inner.poll(cx) {
                        Poll::Ready(output) => Poll::Ready(Progress::from(output)),
                        Poll::Pending => Poll::Pending,
                    }
                }
            }
        }
    };
}

define_public_interface!(
    threadsafe,
    SyncContainer,
    "A threadsafe (`Send + Sync`), asynchronous budgeting implementation that is runtime agnostic.\n\nThe only difference between this module and the [`singlethreaded`] module is that this one uses [`std::sync::Arc`] and [`std::sync::Mutex`] instead of [`std::rc::Rc`] and [`std::cell::RefCell`]."
);

define_public_interface!(
    singlethreaded,
    NotSyncContainer,
    "A single-threaded (`!Send + !Sync`), asynchronous budgeting implementation that is runtime agnostic.\n\nThe only difference between this module and the [`threadsafe`] module is that this one uses [`std::rc::Rc`] and [`std::cell::RefCell`] instead of [`std::sync::Arc`] and [`std::sync::Mutex`]."
);
