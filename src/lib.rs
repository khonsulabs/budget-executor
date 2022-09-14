#![doc = include_str!(".crate-docs.md")]
#![warn(
    clippy::cargo,
    missing_docs,
    clippy::pedantic,
    future_incompatible,
    rust_2018_idioms
)]
#![allow(
    clippy::option_if_let_else,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc
)]

use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::{Context, Poll, Waker},
};

use crate::sealed::BudgetableSealed;

/// A budget implementation compatible with any async executor.
pub mod asynchronous;
/// A standalone implementation does not require another async executor and
/// blocks the current thread while executing.
pub mod blocking;

#[derive(Debug, Clone)]
pub struct BudgetContext<Budget> {
    data: Rc<BudgetContextData<Budget>>,
}

#[derive(Debug)]
struct BudgetContextData<Budget> {
    budget: RefCell<Budget>,
    future_waker: RefCell<Option<Waker>>,
}

/// The result of a completed future.
#[derive(Debug)]
pub struct BudgetResult<T, Budget> {
    /// The output of the future.
    pub output: T,
    /// The budget after completing the future.
    pub balance: Budget,
}

/// Spends `amount` from the curent budget.
///
/// This is a future that must be awaited. This future is created by `spend()`.
#[derive(Debug)]
#[must_use = "budget is not spent until this future is awaited"]
pub struct SpendBudget<'a, Budget> {
    context: &'a BudgetContext<Budget>,
    amount: usize,
}

impl<'a, Budget> Future for SpendBudget<'a, Budget>
where
    Budget: Budgetable,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut budget = self.context.data.budget.borrow_mut();
        if budget.spend(self.amount) {
            budget.remove_waker(cx.waker());
            Poll::Ready(())
        } else {
            // Not enough budget

            match &mut *self.context.data.future_waker.borrow_mut() {
                Some(existing_waker) if existing_waker.will_wake(cx.waker()) => {
                    *existing_waker = cx.waker().clone();
                }
                waker => {
                    *waker = Some(cx.waker().clone());
                }
            }

            budget.add_waker(cx.waker());

            Poll::Pending
        }
    }
}

impl<Budget> BudgetContext<Budget>
where
    Budget: Budgetable,
{
    /// Retrieves the current budget.
    ///
    /// This function should only be called by code that is guaranteed to be running
    /// by this executor. When called outside of code run by this executor, this function will.
    #[must_use]
    pub fn budget(&self) -> usize {
        let budget = self.data.budget.borrow();
        (&*budget).get()
    }

    /// Spends `amount` from the curent budget.
    ///
    /// This function returns a future which must be awaited.
    ///
    /// ```rust
    /// use budget_executor::spend;
    ///
    /// async fn some_task() {
    ///     // Attempt to spend 5 budget. This will pause the
    ///     // async task (Future) until enough budget is available.
    ///     spend(5).await;
    ///     // The budget was spent, proceed with the operation.
    /// }
    /// ```
    pub fn spend(&self, amount: usize) -> SpendBudget<'_, Budget> {
        SpendBudget {
            context: self,
            amount,
        }
    }
}

// #[must_use]
// struct BudgetGuard {
//     needs_reset: bool,
// }

// fn set_budget(new_budget: Box<dyn AnyBudget>) -> BudgetGuard {
//     BUDGET.with(|budget| {
//         let mut budget = budget.borrow_mut();

//         assert!(
//             (&*budget).is_none(),
//             "blocking-executor is not able to be nested"
//         );
//         *budget = Some(new_budget);
//     });
//     BudgetGuard { needs_reset: true }
// }

// impl BudgetGuard {
//     pub fn take_budget(mut self) -> Box<dyn AnyBudget> {
//         self.needs_reset = false;

//         BUDGET.with(RefCell::take).expect("should still be present")
//         // boxed_budget
//         //     .as_any_mut()
//         //     .downcast_mut::<Option<Budget>>()
//         //     .expect("types should match")
//         //     .take()
//         //     .expect("should always be Some")
//     }
// }

// impl Drop for BudgetGuard {
//     fn drop(&mut self) {
//         if self.needs_reset {
//             BUDGET.with(RefCell::take);
//         }
//     }
// }

/// A type that can be used as a budget.
///
/// Current implementors are:
///
/// - [`usize`]
/// - [`ReplenishableBudget`]
pub trait Budgetable: sealed::BudgetableSealed {}

mod sealed {
    pub trait BudgetableSealed: Clone + std::fmt::Debug + Unpin + 'static {
        fn get(&self) -> usize;
        fn spend(&mut self, amount: usize) -> bool;
        fn replenish(&mut self, amount: usize);
        fn add_waker(&self, waker: &std::task::Waker);
        fn remove_waker(&self, waker: &std::task::Waker);
    }
}

impl Budgetable for usize {}

impl BudgetableSealed for usize {
    fn spend(&mut self, amount: usize) -> bool {
        if let Some(remaining) = self.checked_sub(amount) {
            *self = remaining;
            true
        } else {
            false
        }
    }

    fn get(&self) -> usize {
        *self
    }

    fn replenish(&mut self, amount: usize) {
        *self = self.saturating_add(amount);
    }

    fn add_waker(&self, _waker: &std::task::Waker) {}

    fn remove_waker(&self, _waker: &std::task::Waker) {}
}

/// An atomic budget storage that can be replenished by other threads or tasks
/// than the one driving the budgeted task.
#[derive(Clone, Debug, Default)]
pub struct ReplenishableBudget {
    data: Arc<ReplenishableBudgetData>,
}

impl ReplenishableBudget {
    /// Adds `amount` to the budget. This will wake up the task if it is
    /// currently waiting for additional budget.
    pub fn replenish(&self, amount: usize) {
        let mut budget = self.data.budget.load(Ordering::Acquire);
        budget = budget.saturating_add(amount);
        self.data.budget.store(budget, Ordering::Release);
        let mut waker = self.data.waker.lock().expect("panics should be impossible");
        for waker in waker.drain(..) {
            waker.wake();
        }
    }
}

#[derive(Debug, Default)]
struct ReplenishableBudgetData {
    budget: AtomicUsize,
    waker: Mutex<Vec<Waker>>,
}

impl ReplenishableBudget {
    /// Returns a new instance with the intiial budget provided.
    #[must_use]
    pub fn new(initial_budget: usize) -> Self {
        Self {
            data: Arc::new(ReplenishableBudgetData {
                budget: AtomicUsize::new(initial_budget),
                waker: Mutex::default(),
            }),
        }
    }
}

impl Budgetable for ReplenishableBudget {}

impl BudgetableSealed for ReplenishableBudget {
    fn get(&self) -> usize {
        self.data.budget.load(Ordering::Acquire)
    }

    fn spend(&mut self, amount: usize) -> bool {
        self.data
            .budget
            .fetch_update(Ordering::Release, Ordering::Relaxed, |budget| {
                budget.checked_sub(amount)
            })
            .is_ok()
    }

    fn replenish(&mut self, amount: usize) {
        ReplenishableBudget::replenish(self, amount);
    }

    fn add_waker(&self, new_waker: &std::task::Waker) {
        let mut stored_waker = self.data.waker.lock().expect("panics should be impossible");

        if let Some((_, waker)) = stored_waker
            .iter_mut()
            .enumerate()
            .find(|(_, waker)| waker.will_wake(new_waker))
        {
            *waker = new_waker.clone();
        } else {
            stored_waker.push(new_waker.clone());
        }
    }

    fn remove_waker(&self, reference: &std::task::Waker) {
        let mut stored_waker = self.data.waker.lock().expect("panics should be impossible");
        if let Some((index, _)) = stored_waker
            .iter()
            .enumerate()
            .find(|(_, waker)| waker.will_wake(reference))
        {
            stored_waker.remove(index);
        }
    }
}

#[cfg(test)]
mod tests;
