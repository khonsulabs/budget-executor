use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{BudgetContext, BudgetContextData, Budgetable, Container};

/// Spends `amount` from the curent budget.
///
/// This is a future that must be awaited. This future is created by `spend()`.
#[derive(Debug)]
#[must_use = "budget is not spent until this future is awaited"]
pub(crate) struct SpendBudget<'a, Backing, Budget>
where
    Backing: Container<BudgetContextData<Budget>>,
{
    pub(crate) context: &'a BudgetContext<Backing, Budget>,
    pub(crate) amount: usize,
}

impl<'a, Backing, Budget> Future for SpendBudget<'a, Backing, Budget>
where
    Budget: Budgetable,
    Backing: Container<BudgetContextData<Budget>>,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.context.data.map_locked(|data| {
            if data.budget.spend(self.amount) {
                data.budget.remove_waker(cx.waker());
                Poll::Ready(())
            } else {
                // Not enough budget
                match &mut data.future_waker {
                    Some(existing_waker) if existing_waker.will_wake(cx.waker()) => {
                        *existing_waker = cx.waker().clone();
                    }
                    waker => {
                        *waker = Some(cx.waker().clone());
                    }
                }

                data.budget.add_waker(cx.waker());

                Poll::Pending
            }
        })
    }
}

macro_rules! define_public_interface {
    ($modulename:ident, $backing:ident, $moduledocs:literal) => {
        #[doc = $moduledocs]
        pub mod $modulename {
            use std::{
                future::Future,
                pin::Pin,
                task::{Context, Poll},
            };

            use crate::{BudgetContextData, Budgetable};

            type Backing<Budget> = crate::$backing<crate::BudgetContextData<Budget>>;

            /// Spends `amount` from the curent budget.
            ///
            /// This is a future that must be awaited. This future is created by `spend()`.
            #[derive(Debug)]
            #[must_use = "budget is not spent until this future is awaited"]
            pub struct SpendBudget<'a, Budget>(
                crate::SpendBudget<'a, crate::$backing<BudgetContextData<Budget>>, Budget>,
            )
            where
                Budget: Budgetable;

            impl<'a, Budget> Future for SpendBudget<'a, Budget>
            where
                Budget: Budgetable,
            {
                type Output = ();

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let inner = Pin::new(&mut self.0);
                    inner.poll(cx)
                }
            }

            impl<'a, Budget> From<super::SpendBudget<'a, Backing<Budget>, Budget>>
                for SpendBudget<'a, Budget>
            where
                Budget: Budgetable,
            {
                fn from(future: super::SpendBudget<'a, Backing<Budget>, Budget>) -> Self {
                    Self(future)
                }
            }
        }
    };
}

define_public_interface!(
    threadsafe,
    SyncContainer,
    "Threadsafe (`Send + Sync`) budget spending"
);
define_public_interface!(
    singlethreaded,
    NotSyncContainer,
    "Single-threaded (`!Send + !Sync`) budget spending"
);
