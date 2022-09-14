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
    marker::PhantomData,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    task::Waker,
};

use crate::{sealed::BudgetableSealed, spend::SpendBudget};

/// A budget implementation compatible with any async executor.
pub mod asynchronous;
/// A standalone implementation does not require another async executor and
/// blocks the current thread while executing.
pub mod blocking;
/// Shared implementation of budget spending.
pub mod spend;

#[derive(Debug)]
struct BudgetContext<Backing, Budget>
where
    Backing: Container<BudgetContextData<Budget>>,
{
    data: Backing,
    _budget: PhantomData<Budget>,
}

impl<Backing, Budget> Clone for BudgetContext<Backing, Budget>
where
    Backing: Container<BudgetContextData<Budget>>,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.cloned(),
            _budget: PhantomData,
        }
    }
}

#[derive(Debug)]
struct BudgetContextData<Budget> {
    budget: Budget,
    future_waker: Option<Waker>,
}

/// The result of a completed future.
#[derive(Debug)]
pub struct BudgetResult<T, Budget> {
    /// The output of the future.
    pub output: T,
    /// The budget after completing the future.
    pub balance: Budget,
}

impl<Backing, Budget> BudgetContext<Backing, Budget>
where
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
{
    /// Retrieves the current budget.
    ///
    /// This function should only be called by code that is guaranteed to be running
    /// by this executor. When called outside of code run by this executor, this function will.
    #[must_use]
    fn budget(&self) -> usize {
        self.data.map_locked(|data| data.budget.get())
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
    fn spend(&self, amount: usize) -> SpendBudget<'_, Backing, Budget> {
        SpendBudget {
            context: self,
            amount,
        }
    }
}

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

    /// Returns the remaining budget.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.budget.load(Ordering::Acquire)
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

trait Container<T>: Unpin + 'static {
    fn new(contained: T) -> Self;
    fn cloned(&self) -> Self;
    fn map_locked<R, F: FnOnce(&mut T) -> R>(&self, map: F) -> R;
}

#[derive(Debug)]
struct SyncContainer<T>(Arc<Mutex<T>>);

impl<T> Clone for SyncContainer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Container<T> for SyncContainer<T>
where
    T: 'static,
{
    fn map_locked<R, F: FnOnce(&mut T) -> R>(&self, map: F) -> R {
        let mut locked = self.0.lock().unwrap();
        map(&mut locked)
    }

    fn cloned(&self) -> Self {
        Self(self.0.clone())
    }

    fn new(contained: T) -> Self {
        Self(Arc::new(Mutex::new(contained)))
    }
}

#[derive(Debug)]
struct NotSyncContainer<T>(Rc<RefCell<T>>);

impl<T> Clone for NotSyncContainer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Container<T> for NotSyncContainer<T>
where
    T: 'static,
{
    fn map_locked<R, F: FnOnce(&mut T) -> R>(&self, map: F) -> R {
        let mut locked = self.0.borrow_mut();
        map(&mut *locked)
    }

    fn cloned(&self) -> Self {
        Self(self.0.clone())
    }

    fn new(contained: T) -> Self {
        Self(Rc::new(RefCell::new(contained)))
    }
}

#[cfg(test)]
mod tests;
