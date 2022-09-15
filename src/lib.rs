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
    sync::{Arc, Mutex},
    task::Waker,
};

use crate::{sealed::BudgetableSealed, spend::SpendBudget};

/// A budget implementation compatible with any async executor.
pub mod asynchronous;
/// A standalone implementation does not require another async executor and
/// blocks the current thread while executing.
pub mod blocking;
mod replenishable;
/// Shared implementation of budget spending.
pub mod spend;

pub use replenishable::ReplenishableBudget;

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
    paused_future: Option<Waker>,
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
    #[must_use]
    fn budget(&self) -> usize {
        self.data.map_locked(|data| data.budget.get())
    }

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
        fn park_for_budget(&self);
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

    fn park_for_budget(&self) {
        std::thread::park();
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
