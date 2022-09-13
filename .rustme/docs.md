An approach to "throttling" async tasks in Rust using manual instrumentation.

[![crate version](https://img.shields.io/crates/v/budget-executor.svg)](https://crates.io/crates/budget-executor)
[![Live Build Status](https://img.shields.io/github/workflow/status/khonsulabs/budget-executor/Tests/main)](https://github.com/khonsulabs/budget-executor/actions?query=workflow:Tests)
[![HTML Coverage Report for `main` branch](https://khonsulabs.github.io/budget-executor/coverage/badge.svg)](https://khonsulabs.github.io/budget-executor/coverage/)
[![Documentation](https://img.shields.io/badge/docs-main-informational)]($docs-base$)

This crate implements a manual task throttling approach using a simple `usize` to
track available budget. Both leverage Rust's approach to async tasks to enable
pausing the task when it requests to spend more budget than is remaining.

One use-case/inspiration for this crate is a scripting language built in Rust.
By defining parts of the interpeter using `async` functions, untrusted scripts
can be executed with this crate with limited budget to protect against infinite
loops or too-intensive scripts slowing down a server. The interpreter can assign
different costs to various operations.

This crate has two implementations of this approach: one that blocks the current
thread and one that works with any async runtime.

## Using from non-async code (Blocking)

This example is `examples/simple.rs` in the repository:

```rust,ignore
$../examples/simple.rs$
```

When run, it produces this output:

```sh
~/p/budget-executor (main)> cargo run --example simple
   Compiling budget-executor v0.1.0 (/home/ecton/projects/budget-executor)
    Finished dev [unoptimized + debuginfo] target(s) in 0.31s
     Running `target/debug/examples/simple`
> Asking to spend 1 from the budget
+5 budget
> Asking to spend 5 from the budget
+5 budget
> Asking to spend 1 from the budget
> Asking to spend 25 from the budget
+5 budget
+5 budget
+5 budget
+5 budget
+5 budget
Task completed with balance: Remaining(3), output: true
```

## Using from async code

This example is `examples/simple-async.rs` in the repository:

```rust,ignore
$../examples/simple-async.rs$
```

When run, it produces the same output as displayed in the blocking section.
