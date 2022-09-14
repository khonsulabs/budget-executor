use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    fmt::Debug,
    future::Future,
    ops::Deref,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use crate::{
    sealed::BudgetableSealed, BudgetContext, BudgetResult, Budgetable, ReplenishableBudget,
};

#[derive(Clone)]
pub struct Runtime<Budget> {
    context: BudgetContext<Budget>,
    tasks: Rc<RefCell<RuntimeTasks>>,
}

impl<Budget> Deref for Runtime<Budget> {
    type Target = BudgetContext<Budget>;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl<Budget> Debug for Runtime<Budget>
where
    Budget: Budgetable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runtime")
            .field("context", &self.context)
            .field("tasks", &())
            .finish()
    }
}

impl<Budget> Runtime<Budget>
where
    Budget: Budgetable,
{
    pub fn spawn<F: Future + 'static>(&self, future: F) -> TaskHandle<F::Output> {
        let mut tasks = self.tasks.borrow_mut();
        let status = Rc::new(RefCell::new(SpawnedTaskStatus::default()));
        let task_id = tasks.next_task_id;
        tasks.next_task_id = tasks.next_task_id.checked_add(1).expect("u64 wrapped");

        // TODO two allocations isn't ideal. Can we do pin projection through
        // dynamic dispatch? I think it's "safe" because it seems to fit the
        // definition of structural pinning?
        tasks.woke.push_back(Box::new(SpawnedTask {
            id: task_id,
            future: Some(Box::pin(future)),
            status: status.clone(),
            waker: budget_waker::for_current_thread(self.tasks.clone(), Some(task_id)),
        }));

        TaskHandle { status }
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
pub fn run_with_budget<Budget: Budgetable, F: Future>(
    future: impl FnOnce(Runtime<Budget>) -> F,
    initial_budget: Budget,
) -> Progress<Budget, F> {
    let runtime = Runtime {
        context: BudgetContext {
            data: Rc::new(crate::BudgetContextData {
                budget: RefCell::new(initial_budget),
                future_waker: RefCell::new(None),
            }),
        },
        tasks: Rc::new(RefCell::new(RuntimeTasks::default())),
    };

    let waker = budget_waker::for_current_thread(runtime.tasks.clone(), None);
    execute_future(Box::pin(future(runtime.clone())), waker, runtime)
}

fn execute_future<Budget: Budgetable, F: Future>(
    mut future: Pin<Box<F>>,
    waker: Waker,
    runtime: Runtime<Budget>,
) -> Progress<Budget, F> {
    let mut pinned_future = Pin::new(&mut future);
    let mut cx = Context::from_waker(&waker);
    loop {
        let poll_result = pinned_future.poll(&mut cx);
        runtime
            .context
            .data
            .budget
            .borrow()
            .remove_waker(cx.waker());
        let ran_out_of_budget = runtime.context.data.future_waker.take().is_some();

        if let Poll::Ready(output) = poll_result {
            return Progress::Complete(BudgetResult {
                output,
                balance: runtime.context.data.budget.borrow().clone(),
            });
        }

        if ran_out_of_budget {
            return Progress::NoBudget(IncompleteFuture {
                future,
                waker,
                runtime,
            });
        }

        runtime.context.data.budget.borrow().add_waker(cx.waker());
        pinned_future = Pin::new(&mut future);

        // If we have our own tasks to run, execute them. Otherwise, park
        // the thread.
        let mut tasks = runtime.tasks.borrow_mut();
        if tasks.woke.is_empty() {
            drop(tasks);
            std::thread::park();
        } else {
            while let Some(mut task) = tasks.woke.pop_front() {
                drop(tasks);
                let result = task.poll();
                tasks = runtime.tasks.borrow_mut();
                match result {
                    Poll::Ready(_) => {}
                    Poll::Pending => {
                        tasks.pending.insert(task.id(), task);
                    }
                }
            }
        }
    }
}

/// A future that was budgeted with [`run_with_budget()`] that has not yet
/// completed.
pub struct IncompleteFuture<Budget, F>
where
    F: Future,
{
    future: Pin<Box<F>>,
    waker: Waker,
    runtime: Runtime<Budget>,
}

impl<Budget, F> IncompleteFuture<Budget, F>
where
    F: Future,
    Budget: Budgetable,
{
    /// Adds `additional_budget` to the remaining balance and continues
    /// executing the future.
    pub fn continue_with_additional_budget(self, additional_budget: usize) -> Progress<Budget, F> {
        let Self {
            future,
            waker,
            runtime,
            ..
        } = self;
        runtime
            .context
            .data
            .budget
            .borrow_mut()
            .replenish(additional_budget);

        execute_future(future, waker, runtime)
    }
}

impl<F> IncompleteFuture<ReplenishableBudget, F>
where
    F: Future,
{
    /// Waits for additional budget to be allocated through
    /// [`ReplenishableBudget::replenish()`].
    pub fn wait_for_budget(self) -> Progress<ReplenishableBudget, F> {
        let Self {
            future,
            waker,
            runtime,
            ..
        } = self;

        runtime.context.data.budget.borrow().add_waker(&waker);
        std::thread::park();
        runtime.context.data.budget.borrow().remove_waker(&waker);

        execute_future(future, waker, runtime)
    }
}

/// The progress of a future's execution.
#[must_use]
pub enum Progress<Budget, F: Future> {
    /// The future was interrupted because it requested to spend more budget
    /// than was available.
    NoBudget(IncompleteFuture<Budget, F>),
    /// The future has completed.
    Complete(BudgetResult<F::Output, Budget>),
}

impl<F> Progress<ReplenishableBudget, F>
where
    F: Future,
{
    pub fn wait_until_complete(self) -> BudgetResult<F::Output, ReplenishableBudget> {
        let mut progress = self;
        loop {
            match progress {
                Progress::NoBudget(incomplete) => progress = incomplete.wait_for_budget(),
                Progress::Complete(result) => break result,
            }
        }
    }
}

mod budget_waker {
    use std::{
        cell::RefCell,
        rc::Rc,
        sync::Arc,
        task::{RawWaker, RawWakerVTable, Waker},
        thread::Thread,
    };

    use crate::blocking::RuntimeTasks;

    struct WakerData {
        thread: Thread,
        task_id: Option<u64>,
        tasks: Rc<RefCell<RuntimeTasks>>,
    }

    impl WakerData {
        pub fn wake(&self) {
            if let Some(task_id) = &self.task_id {
                let mut tasks = self.tasks.borrow_mut();
                if let Some(task) = tasks.pending.remove(task_id) {
                    tasks.woke.push_back(task);
                    drop(tasks);
                    self.thread.unpark();
                } else {
                    // Task already finished or already woken, do nothing.
                }
            } else {
                // The main task is unblocked, we always unpark.
                self.thread.unpark();
            }
        }
    }

    pub(crate) fn for_current_thread(
        tasks: Rc<RefCell<RuntimeTasks>>,
        task_id: Option<u64>,
    ) -> Waker {
        let arc_thread = Arc::new(WakerData {
            tasks,
            thread: std::thread::current(),
            task_id,
        });
        let arc_thread = Arc::into_raw(arc_thread);

        unsafe { Waker::from_raw(RawWaker::new(arc_thread.cast::<()>(), &BUDGET_WAKER_VTABLE)) }
    }

    unsafe fn clone(arc_thread: *const ()) -> RawWaker {
        let arc_thread: Arc<WakerData> = Arc::from_raw(arc_thread.cast::<WakerData>());
        let cloned = arc_thread.clone();
        let cloned = Arc::into_raw(cloned);

        let _ = Arc::into_raw(arc_thread);

        RawWaker::new(cloned.cast::<()>(), &BUDGET_WAKER_VTABLE)
    }

    unsafe fn wake_consuming(arc_thread: *const ()) {
        let arc_thread: Arc<WakerData> = Arc::from_raw(arc_thread as *mut WakerData);
        arc_thread.wake();
    }

    unsafe fn wake_by_ref(arc_thread: *const ()) {
        let arc_thread: Arc<WakerData> = Arc::from_raw(arc_thread as *mut WakerData);
        arc_thread.wake();

        let _ = Arc::into_raw(arc_thread);
    }

    unsafe fn drop_waker(arc_thread: *const ()) {
        let arc_thread: Arc<WakerData> = Arc::from_raw(arc_thread as *mut WakerData);
        drop(arc_thread);
    }

    const BUDGET_WAKER_VTABLE: RawWakerVTable =
        RawWakerVTable::new(clone, wake_consuming, wake_by_ref, drop_waker);
}

struct SpawnedTask<F>
where
    F: Future,
{
    id: u64,
    future: Option<Pin<Box<F>>>,
    status: Rc<RefCell<SpawnedTaskStatus<F::Output>>>,
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

impl<F> AnySpawnedTask for SpawnedTask<F>
where
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
                        let mut status = self.status.borrow_mut();
                        status.output = TaskOutput::Completed(Some(output));
                        if let Some(waker) = status.waker.take() {
                            waker.wake();
                        }
                        Poll::Ready(())
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
            None => Poll::Ready(()),
        }
    }
}

pub struct TaskHandle<Output> {
    status: Rc<RefCell<SpawnedTaskStatus<Output>>>,
}

impl<Output> Future for TaskHandle<Output> {
    type Output = Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut status = self.status.borrow_mut();
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
    }
}

enum TaskOutput<Output> {
    NotCompleted,
    Completed(Option<Output>),
}
