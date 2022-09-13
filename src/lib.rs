#![doc = include_str!("../README.md")]
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
    cell::Cell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

/// A budget implementation compatible with any async executor.
pub mod asynchronous;
/// A standalone implementation does not require another async executor and
/// blocks the current thread while executing.
pub mod blocking;

thread_local! {
    static BUDGET: Cell<Option<Budget>> = Cell::new(None);

    static FUTURE_WAKER: Cell<Option<Waker>> = Cell::new(None);
}

fn compute_overage(original_budget_remaining: usize, balance: Budget) -> Budget {
    match balance {
        Budget::Remaining(remaining) => {
            Budget::Remaining(remaining.saturating_add(original_budget_remaining))
        }
        Budget::Overage(overage) => {
            if overage <= original_budget_remaining {
                Budget::Remaining(original_budget_remaining - overage)
            } else {
                Budget::Overage(overage - original_budget_remaining)
            }
        }
    }
}

/// The result of a completed future.
#[derive(Debug)]
pub struct BudgetResult<T> {
    /// The output of the future.
    pub output: T,
    /// The budget after completing the future.
    pub balance: Budget,
}

/// Spends `amount` from the curent budget.
///
/// This function returns a future which must be awaited.
pub fn spend(amount: usize) -> SpendBudget {
    SpendBudget { amount }
}

/// Spends `amount` from the curent budget.
///
/// This is a future that must be awaited. This future is created by `spend()`.
#[derive(Debug)]
#[must_use = "budget is not spent until this future is awaited"]
pub struct SpendBudget {
    amount: usize,
}

impl Future for SpendBudget {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        BUDGET.with(|budget| match budget.get() {
            Some(Budget::Remaining(remaining)) => {
                if let Some(remaining) = remaining.checked_sub(self.amount) {
                    budget.set(Some(Budget::Remaining(remaining)));
                    Poll::Ready(())
                } else {
                    // Not enough budget
                    FUTURE_WAKER.with(|waker| match waker.take() {
                        Some(existing_waker) if existing_waker.will_wake(cx.waker()) => {
                            waker.set(Some(existing_waker));
                        }
                        _ => {
                            waker.set(Some(cx.waker().clone()));
                        }
                    });
                    Poll::Pending
                }
            }
            Some(Budget::Overage(overage)) => {
                // We are running unthrottled, keep track of the total.
                let overage = overage.saturating_add(self.amount);
                budget.set(Some(Budget::Overage(overage)));
                Poll::Ready(())
            }
            None => panic!("burn_gas called outside of `run_with_gas`"),
        })
    }
}

/// Retrieves the current budget.
///
/// This function should only be called by code that is guaranteed to be running
/// by this executor. When called outside of code run by this executor, this function will.
pub fn budget() -> Option<Budget> {
    BUDGET.with(Cell::get)
}

/// The status of the budget.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Budget {
    /// The remaining budget.
    Remaining(usize),
    /// The amount of overspend.
    Overage(usize),
}

#[must_use]
struct BudgetGuard {
    needs_reset: bool,
}

fn set_budget(new_budget: Budget) -> BudgetGuard {
    BUDGET.with(|budget| {
        assert!(
            budget.get().is_none(),
            "blocking-executor is not able to be nested"
        );
        budget.set(Some(new_budget));
    });
    BudgetGuard { needs_reset: true }
}

impl BudgetGuard {
    pub fn take_budget(mut self) -> Budget {
        self.needs_reset = false;

        BUDGET.with(Cell::take).expect("should still be present")
    }
}

impl Drop for BudgetGuard {
    fn drop(&mut self) {
        if self.needs_reset {
            BUDGET.with(Cell::take);
        }
    }
}

#[cfg(test)]
mod tests;
