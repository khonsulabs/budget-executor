use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    future::Future,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crate::{BudgetContext, BudgetContextData, BudgetResult, Budgetable, Container};

struct Runtime<Budget, Tasks, Backing>
where
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    context: BudgetContext<Backing, Budget>,
    tasks: Tasks,
}

impl<Budget, Tasks, Backing> Clone for Runtime<Budget, Tasks, Backing>
where
    Budget: Clone,
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            tasks: self.tasks.cloned(),
        }
    }
}

impl<Budget, Tasks, Backing> Deref for Runtime<Budget, Tasks, Backing>
where
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    type Target = BudgetContext<Backing, Budget>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<Budget, Tasks, Backing> Debug for Runtime<Budget, Tasks, Backing>
where
    Budget: Budgetable,
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("context", &self.context)
            .field("tasks", &())
            .finish()
    }
}

impl<Budget, Tasks, Backing> Runtime<Budget, Tasks, Backing>
where
    Budget: Budgetable,
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    pub fn spawn<Status, F: Future + 'static>(&self, future: F) -> TaskHandle<Status, F::Output>
    where
        Status: Container<SpawnedTaskStatus<F::Output>>,
    {
        self.tasks.map_locked(|tasks| {
            let status = Status::new(SpawnedTaskStatus::default());
            let task_id = tasks.next_task_id;
            tasks.next_task_id = tasks.next_task_id.checked_add(1).expect("u64 wrapped");

            // TODO two allocations isn't ideal. Can we do pin projection through
            // dynamic dispatch? I think it's "safe" because it seems to fit the
            // definition of structural pinning?
            tasks.woke.push_back(Box::new(SpawnedTask {
                id: task_id,
                future: Some(Box::pin(future)),
                status: status.cloned(),
                waker: budget_waker::for_current_thread(self.tasks.cloned(), Some(task_id)),
            }));

            TaskHandle {
                status,
                _output: PhantomData,
            }
        })
    }
}

#[derive(Default)]
pub(crate) struct RuntimeTasks {
    next_task_id: u64,
    woke: VecDeque<Box<dyn AnySpawnedTask>>,
    pending: HashMap<u64, Box<dyn AnySpawnedTask>>,
}

/// Executes `future` with the provided budget. The future will run until it
/// completes or until it has invoked [`spend()`](crate::spend) enough to
/// exhaust the budget provided. If the future never called
/// [`spend()`](crate::spend), this function will block the current thread until
/// the future completes.
///
/// This method should not be used within async methods. For running a future
/// inside of an async context, use
/// [`asynchronous::run_with_budget()`](crate::asynchronous::run_with_budget).
///
/// # Panics
///
/// Panics when called within from within `future` or any code invoked by
/// `future`.
fn run_with_budget<Budget, Tasks, Backing, F>(
    future: impl FnOnce(Runtime<Budget, Tasks, Backing>) -> F,
    initial_budget: Budget,
) -> Progress<Budget, Tasks, Backing, F>
where
    Budget: Budgetable,
    Tasks: Container<RuntimeTasks>,
    F: Future,
    Backing: Container<BudgetContextData<Budget>>,
{
    let runtime = Runtime {
        context: BudgetContext {
            data: Backing::new(crate::BudgetContextData {
                budget: initial_budget,
                future_waker: None,
            }),
            _budget: PhantomData,
        },
        tasks: Tasks::new(RuntimeTasks::default()),
    };

    let waker = budget_waker::for_current_thread(runtime.tasks.cloned(), None);
    execute_future(Box::pin(future(runtime.clone())), waker, runtime)
}

fn execute_future<Budget, Tasks, Backing, F>(
    mut future: Pin<Box<F>>,
    waker: Waker,
    runtime: Runtime<Budget, Tasks, Backing>,
) -> Progress<Budget, Tasks, Backing, F>
where
    Budget: Budgetable,
    Tasks: Container<RuntimeTasks>,
    F: Future,
    Backing: Container<BudgetContextData<Budget>>,
{
    let mut pinned_future = Pin::new(&mut future);
    let mut cx = Context::from_waker(&waker);
    loop {
        let poll_result = pinned_future.poll(&mut cx);
        let (ran_out_of_budget, budget_remaining_after_completion) =
            runtime.context.data.map_locked(|data| {
                data.budget.remove_waker(cx.waker());
                let ran_out_of_budget = data.future_waker.take().is_some();
                let budget_remaining_after_completion = if poll_result.is_ready() {
                    Some(data.budget.clone())
                } else {
                    if !ran_out_of_budget {
                        data.budget.add_waker(cx.waker());
                    }
                    None
                };
                (ran_out_of_budget, budget_remaining_after_completion)
            });

        if let Poll::Ready(output) = poll_result {
            return Progress::Complete(BudgetResult {
                output,
                balance: budget_remaining_after_completion
                    .expect("should always be present when Ready returns"),
            });
        }

        if ran_out_of_budget {
            return Progress::NoBudget(IncompleteFuture {
                future,
                waker,
                runtime,
            });
        }

        pinned_future = Pin::new(&mut future);

        // If we have our own tasks to run, execute them. Otherwise, park
        // the thread.
        let mut task_to_wake = runtime.tasks.map_locked(|tasks| tasks.woke.pop_front());

        if task_to_wake.is_none() {
            std::thread::park();
        } else {
            // Call poll on all of the futures that are awake.
            while let Some(mut task) = task_to_wake {
                let result = task.poll();
                task_to_wake = runtime.tasks.map_locked(|tasks| {
                    match result {
                        Poll::Ready(_) => {
                            // The future is complete.
                        }
                        Poll::Pending => {
                            // Move the future to the pending queue.
                            tasks.pending.insert(task.id(), task);
                        }
                    }
                    tasks.woke.pop_front()
                });
            }
        }
    }
}

/// A future that was budgeted with [`run_with_budget()`] that has not yet
/// completed.
struct IncompleteFuture<Budget, Tasks, Backing, F>
where
    F: Future,
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    future: Pin<Box<F>>,
    waker: Waker,
    runtime: Runtime<Budget, Tasks, Backing>,
}

impl<Budget, Tasks, Backing, F> IncompleteFuture<Budget, Tasks, Backing, F>
where
    F: Future,
    Budget: Budgetable,
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    pub fn continue_with_additional_budget(
        self,
        additional_budget: usize,
    ) -> Progress<Budget, Tasks, Backing, F> {
        let Self {
            future,
            waker,
            runtime,
            ..
        } = self;
        runtime
            .context
            .data
            .map_locked(|data| data.budget.replenish(additional_budget));

        execute_future(future, waker, runtime)
    }
    /// Waits for additional budget to be allocated through
    /// [`ReplenishableBudget::replenish()`].
    pub fn wait_for_budget(self) -> Progress<Budget, Tasks, Backing, F> {
        let Self {
            future,
            waker,
            runtime,
            ..
        } = self;

        runtime
            .context
            .data
            .map_locked(|data| data.budget.add_waker(&waker));
        std::thread::park();
        runtime
            .context
            .data
            .map_locked(|data| data.budget.remove_waker(&waker));

        execute_future(future, waker, runtime)
    }
}

/// The progress of a future's execution.
#[must_use]
enum Progress<Budget, Tasks, Backing, F: Future>
where
    Tasks: Container<RuntimeTasks>,
    Backing: Container<BudgetContextData<Budget>>,
{
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteFuture<Budget, Tasks, Backing, F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output, Budget>),
}

mod budget_waker {
    use std::{
        any::TypeId,
        sync::Arc,
        task::{RawWaker, RawWakerVTable, Waker},
        thread::Thread,
    };

    use crate::{blocking::RuntimeTasks, Container, NotSyncContainer, SyncContainer};

    struct WakerData<Tasks> {
        thread: Thread,
        task_id: Option<u64>,
        tasks: Tasks,
    }

    impl<Tasks> WakerData<Tasks>
    where
        Tasks: Container<RuntimeTasks>,
    {
        pub fn wake(&self) {
            if let Some(task_id) = &self.task_id {
                let woke_task = self.tasks.map_locked(|tasks| {
                    if let Some(task) = tasks.pending.remove(task_id) {
                        tasks.woke.push_back(task);
                        true
                    } else {
                        false
                    }
                });
                if woke_task {
                    self.thread.unpark();
                }
            } else {
                // The main task is unblocked, we always unpark.
                self.thread.unpark();
            }
        }
    }

    pub(crate) fn for_current_thread<Tasks>(tasks: Tasks, task_id: Option<u64>) -> Waker
    where
        Tasks: Container<RuntimeTasks>,
    {
        let arc_thread = Arc::new(WakerData {
            tasks,
            thread: std::thread::current(),
            task_id,
        });
        let arc_thread = Arc::into_raw(arc_thread);

        unsafe { Waker::from_raw(RawWaker::new(arc_thread.cast::<()>(), vtable::<Tasks>())) }
    }

    unsafe fn clone<Tasks>(arc_thread: *const ()) -> RawWaker
    where
        Tasks: Container<RuntimeTasks>,
    {
        let arc_thread: Arc<WakerData<Tasks>> =
            Arc::from_raw(arc_thread.cast::<WakerData<Tasks>>());
        let cloned = arc_thread.clone();
        let cloned = Arc::into_raw(cloned);

        let _ = Arc::into_raw(arc_thread);

        RawWaker::new(cloned.cast::<()>(), vtable::<Tasks>())
    }

    fn vtable<Tasks>() -> &'static RawWakerVTable
    where
        Tasks: Container<RuntimeTasks>,
    {
        let task_type = TypeId::of::<Tasks>();
        let sync_type = TypeId::of::<SyncContainer<RuntimeTasks>>();
        let not_sync_type = TypeId::of::<NotSyncContainer<RuntimeTasks>>();

        if task_type == sync_type {
            &SYNC_VTABLE
        } else if task_type == not_sync_type {
            &NOT_SYNC_VTABLE
        } else {
            unreachable!("unknown type Tasks")
        }
    }

    unsafe fn wake_consuming<Tasks>(arc_thread: *const ())
    where
        Tasks: Container<RuntimeTasks>,
    {
        let arc_thread: Arc<WakerData<Tasks>> = Arc::from_raw(arc_thread as *mut WakerData<Tasks>);
        arc_thread.wake();
    }

    unsafe fn wake_by_ref<Tasks>(arc_thread: *const ())
    where
        Tasks: Container<RuntimeTasks>,
    {
        let arc_thread: Arc<WakerData<Tasks>> = Arc::from_raw(arc_thread as *mut WakerData<Tasks>);
        arc_thread.wake();

        let _ = Arc::into_raw(arc_thread);
    }

    unsafe fn drop_waker<Tasks>(arc_thread: *const ()) {
        let arc_thread: Arc<WakerData<Tasks>> = Arc::from_raw(arc_thread as *mut WakerData<Tasks>);
        drop(arc_thread);
    }

    const SYNC_VTABLE: RawWakerVTable = RawWakerVTable::new(
        clone::<SyncContainer<RuntimeTasks>>,
        wake_consuming::<SyncContainer<RuntimeTasks>>,
        wake_by_ref::<SyncContainer<RuntimeTasks>>,
        drop_waker::<SyncContainer<RuntimeTasks>>,
    );

    const NOT_SYNC_VTABLE: RawWakerVTable = RawWakerVTable::new(
        clone::<NotSyncContainer<RuntimeTasks>>,
        wake_consuming::<NotSyncContainer<RuntimeTasks>>,
        wake_by_ref::<NotSyncContainer<RuntimeTasks>>,
        drop_waker::<NotSyncContainer<RuntimeTasks>>,
    );
}

struct SpawnedTask<Status, F>
where
    Status: Container<SpawnedTaskStatus<F::Output>>,
    F: Future,
{
    id: u64,
    future: Option<Pin<Box<F>>>,
    status: Status,
    waker: Waker,
}

struct SpawnedTaskStatus<Output> {
    output: TaskOutput<Output>,
    waker: Option<Waker>,
}

impl<Output> Default for SpawnedTaskStatus<Output> {
    fn default() -> Self {
        Self {
            output: TaskOutput::NotCompleted,
            waker: None,
        }
    }
}

trait AnySpawnedTask {
    fn id(&self) -> u64;
    fn poll(&mut self) -> Poll<()>;
}

impl<Status, F> AnySpawnedTask for SpawnedTask<Status, F>
where
    Status: Container<SpawnedTaskStatus<F::Output>>,
    F: Future,
{
    fn id(&self) -> u64 {
        self.id
    }

    fn poll(&mut self) -> Poll<()> {
        match self.future.as_mut() {
            Some(future) => {
                let pinned_future = Pin::new(future);
                match pinned_future.poll(&mut Context::from_waker(&self.waker)) {
                    Poll::Ready(output) => {
                        self.status.map_locked(|status| {
                            status.output = TaskOutput::Completed(Some(output));
                            if let Some(waker) = status.waker.take() {
                                waker.wake();
                            }
                        });
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            None => Poll::Ready(()),
        }
    }
}

struct TaskHandle<Status, Output> {
    status: Status,
    _output: PhantomData<Output>,
}

impl<Status, Output> Future for TaskHandle<Status, Output>
where
    Status: Container<SpawnedTaskStatus<Output>>,
{
    type Output = Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.status.map_locked(|status| {
            match &mut status.output {
                TaskOutput::Completed(output) => {
                    Poll::Ready(output.take().expect("already polled completion"))
                }
                TaskOutput::NotCompleted => {
                    // todo check if needed to overwrite
                    status.waker = Some(cx.waker().clone());
                    Poll::Pending
                }
            }
        })
    }
}

enum TaskOutput<Output> {
    NotCompleted,
    Completed(Option<Output>),
}

macro_rules! define_public_interface {
    ($modname:ident, $backing:ident, $moduledocs:literal) => {
        #[doc = $moduledocs]
        pub mod $modname {
            use std::{
                future::Future,
                pin::Pin,
                task::{Context, Poll},
            };

            use crate::{
                blocking::RuntimeTasks, spend::$modname::SpendBudget, BudgetContextData,
                BudgetResult, Budgetable,
            };

            /// A lightweight asynchronous runtime that runs a future while
            /// keeping track of a budget.
            ///
            /// Regardless of whether the threadsafe or single-threaded versions
            /// of the Runtime are used, this Runtime will always be a
            /// single-threaded runtime. The benefit of using the threadsafe
            /// runtime is to be able to move paused runtimes between different
            /// threads or allow the running futures to be woken by external
            /// threads.
            #[derive(Debug, Clone)]
            pub struct Runtime<Budget>(
                super::Runtime<
                    Budget,
                    crate::$backing<RuntimeTasks>,
                    crate::$backing<BudgetContextData<Budget>>,
                >,
            )
            where
                Budget: Budgetable;

            impl<Budget> Runtime<Budget>
            where
                Budget: Budgetable,
            {
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
                pub fn run_with_budget<F>(
                    future: impl FnOnce(Self) -> F,
                    initial_budget: Budget,
                ) -> Progress<Budget, F>
                where
                    Budget: Budgetable,
                    F: Future,
                {
                    Progress::from(super::run_with_budget(
                        |rt| future(Self(rt)),
                        initial_budget,
                    ))
                }

                /// Schedules `future` to run.
                ///
                /// This runtime will only execute one future at any given
                /// moment in time. Context switches between futures will only
                /// happen when the running future yields control at an `await`.
                ///
                /// Returns a handle that can be `.await`ed to wait for the task
                /// to complete.
                pub fn spawn<F: Future + 'static>(&self, future: F) -> TaskHandle<F::Output> {
                    TaskHandle(self.0.spawn(future))
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
                    // How do we re-export SpendBudget since it's sahrd with async too. crate-level module?
                    SpendBudget::from(self.0.spend(amount))
                }
            }

            /// The progress of a future's execution.
            #[must_use]
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

            impl<Budget, F> Progress<Budget, F>
            where
                Budget: Budgetable,
                F: Future,
            {
                /// Continues executing the contained future until it is
                /// completed.
                ///
                /// This function will never return if the future enters an
                /// infinite loop or deadlocks, regardless of whether the budget
                /// is exhausted or not.
                pub fn wait_until_complete(self) -> BudgetResult<F::Output, Budget> {
                    let mut progress = self;
                    loop {
                        match progress {
                            Progress::NoBudget(incomplete) => {
                                progress = incomplete.wait_for_budget();
                            }
                            Progress::Complete(result) => break result,
                        }
                    }
                }
            }

            impl<Budget, F>
                From<
                    super::Progress<
                        Budget,
                        crate::$backing<RuntimeTasks>,
                        crate::$backing<BudgetContextData<Budget>>,
                        F,
                    >,
                > for Progress<Budget, F>
            where
                Budget: Budgetable,
                F: Future,
            {
                fn from(
                    progress: super::Progress<
                        Budget,
                        crate::$backing<RuntimeTasks>,
                        crate::$backing<BudgetContextData<Budget>>,
                        F,
                    >,
                ) -> Self {
                    match progress {
                        super::Progress::NoBudget(incomplete) => {
                            Self::NoBudget(IncompleteFuture(incomplete))
                        }
                        super::Progress::Complete(result) => Self::Complete(result),
                    }
                }
            }

            /// A future that was budgeted with [`Runtime::run_with_budget()`] that has not yet
            /// completed.
            pub struct IncompleteFuture<Budget, F>(
                pub(super)  super::IncompleteFuture<
                    Budget,
                    crate::$backing<RuntimeTasks>,
                    crate::$backing<BudgetContextData<Budget>>,
                    F,
                >,
            )
            where
                Budget: Budgetable,
                F: Future;

            impl<Budget, F> IncompleteFuture<Budget, F>
            where
                Budget: Budgetable,
                F: Future,
            {
                /// Adds `additional_budget` to the remaining balance and continues
                /// executing the future.
                pub fn continue_with_additional_budget(
                    self,
                    additional_budget: usize,
                ) -> Progress<Budget, F> {
                    Progress::from(self.0.continue_with_additional_budget(additional_budget))
                }

                /// Waits for additional budget to be allocated through
                /// [`ReplenishableBudget::replenish()`](crate::ReplenishableBudget::replenish).
                pub fn wait_for_budget(self) -> Progress<Budget, F> {
                    Progress::from(self.0.wait_for_budget())
                }
            }

            /// A handle to a task scheduled with a [`Runtime`]. Invoking
            /// `.await` on this type will return the task's output when the
            /// original task completes.
            ///
            /// The task will continue to execute even if the handle is dropped.
            pub struct TaskHandle<Output>(
                super::TaskHandle<crate::$backing<super::SpawnedTaskStatus<Output>>, Output>,
            );

            impl<Output> Future for TaskHandle<Output>
            where
                Output: Unpin + 'static,
            {
                type Output = Output;

                fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                    let inner = Pin::new(&mut self.0);
                    inner.poll(cx)
                }
            }
        }
    };
}

define_public_interface!(
    threadsafe,
    SyncContainer,
    "A threadsafe (`Send + Sync`), blocking budgeting implementation that is runtime agnostic.\n\nThe only difference between this module and the [`singlethreaded`] module is that this one uses [`std::sync::Arc`] and [`std::sync::Mutex`] instead of [`std::rc::Rc`] and [`std::cell::RefCell`]."
);

define_public_interface!(
    singlethreaded,
    NotSyncContainer,
    "A single-threaded (`!Send + !Sync`), blocking budgeting implementation that is runtime agnostic.\n\nThe only difference between this module and the [`threadsafe`] module is that this one uses [`std::rc::Rc`] and [`std::cell::RefCell`] instead of [`std::sync::Arc`] and [`std::sync::Mutex`]."
);
