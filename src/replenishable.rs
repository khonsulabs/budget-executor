use std::{
    sync::{Arc, Condvar, Mutex},
    task::Waker,
};

use crate::{sealed::BudgetableSealed, Budgetable};

/// An atomic budget storage that can be replenished by other threads or tasks
/// than the one driving the budgeted task.
#[derive(Clone, Debug, Default)]
pub struct ReplenishableBudget {
    data: Arc<Data>,
}

#[derive(Debug, Default)]
struct State {
    generation: usize,
    denied_budget_at_generation: Option<usize>,
    budget: usize,
    wakers: Vec<Waker>,
}

#[derive(Debug, Default)]
struct Data {
    sync: Condvar,
    state: Mutex<State>,
}

impl ReplenishableBudget {
    /// Adds `amount` to the budget. This will wake up the task if it is
    /// currently waiting for additional budget.
    pub fn replenish(&self, amount: usize) {
        let mut state = self.data.state.lock().expect("poisoned");
        state.generation = state.generation.wrapping_add(1);
        state.budget = state.budget.saturating_add(amount);

        for waker in state.wakers.drain(..) {
            waker.wake();
        }
        drop(state);
        self.data.sync.notify_all();
    }

    /// Returns the remaining budget.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.state.lock().expect("poisoned").budget
    }
}

impl ReplenishableBudget {
    /// Returns a new instance with the intiial budget provided.
    #[must_use]
    pub fn new(initial_budget: usize) -> Self {
        Self {
            data: Arc::new(Data {
                sync: Condvar::new(),
                state: Mutex::new(State {
                    generation: 0,
                    denied_budget_at_generation: None,
                    budget: initial_budget,
                    wakers: Vec::default(),
                }),
            }),
        }
    }
}

impl Budgetable for ReplenishableBudget {}

impl BudgetableSealed for ReplenishableBudget {
    fn get(&self) -> usize {
        self.remaining()
    }

    fn spend(&mut self, amount: usize) -> bool {
        let mut state = self.data.state.lock().expect("poisoned");
        if let Some(remaining) = state.budget.checked_sub(amount) {
            state.denied_budget_at_generation = None;
            state.budget = remaining;
            true
        } else {
            state.denied_budget_at_generation = Some(state.generation);
            false
        }
    }

    fn replenish(&mut self, amount: usize) {
        ReplenishableBudget::replenish(self, amount);
    }

    fn add_waker(&self, new_waker: &std::task::Waker) {
        let mut state = self.data.state.lock().expect("poisoned");

        if let Some((_, waker)) = state
            .wakers
            .iter_mut()
            .enumerate()
            .find(|(_, waker)| waker.will_wake(new_waker))
        {
            *waker = new_waker.clone();
        } else {
            state.wakers.push(new_waker.clone());
        }
    }

    fn remove_waker(&self, reference: &std::task::Waker) {
        let mut state = self.data.state.lock().expect("poisoned");
        if let Some((index, _)) = state
            .wakers
            .iter()
            .enumerate()
            .find(|(_, waker)| waker.will_wake(reference))
        {
            state.wakers.remove(index);
        }
    }

    fn park_for_budget(&self) {
        let mut state = self.data.state.lock().expect("poisoned");
        loop {
            if state.denied_budget_at_generation.is_some()
                && state.denied_budget_at_generation != Some(state.generation)
            {
                // The budget was previously denied, but new budget has been
                // allocated. By parking now, we could enter a deadlock if no
                // additional budget is allocated.
                break;
            }

            // Park the thread using a condvar.
            state = self.data.sync.wait(state).expect("poisoned");
        }
    }
}
