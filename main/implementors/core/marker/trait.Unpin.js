(function() {var implementors = {};
implementors["budget_executor"] = [{"text":"impl&lt;Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/threadsafe/struct.Context.html\" title=\"struct budget_executor::asynchronous::threadsafe::Context\">Context</a>&lt;Budget&gt;","synthetic":true,"types":["budget_executor::asynchronous::threadsafe::Context"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/threadsafe/struct.BudgetedFuture.html\" title=\"struct budget_executor::asynchronous::threadsafe::BudgetedFuture\">BudgetedFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::threadsafe::BudgetedFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"enum\" href=\"budget_executor/asynchronous/threadsafe/enum.Progress.html\" title=\"enum budget_executor::asynchronous::threadsafe::Progress\">Progress</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;F as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html#associatedtype.Output\" title=\"type core::future::future::Future::Output\">Output</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::asynchronous::threadsafe::Progress"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/threadsafe/struct.IncompleteFuture.html\" title=\"struct budget_executor::asynchronous::threadsafe::IncompleteFuture\">IncompleteFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::threadsafe::IncompleteFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/threadsafe/struct.WaitForBudgetFuture.html\" title=\"struct budget_executor::asynchronous::threadsafe::WaitForBudgetFuture\">WaitForBudgetFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::threadsafe::WaitForBudgetFuture"]},{"text":"impl&lt;Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/singlethreaded/struct.Context.html\" title=\"struct budget_executor::asynchronous::singlethreaded::Context\">Context</a>&lt;Budget&gt;","synthetic":true,"types":["budget_executor::asynchronous::singlethreaded::Context"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/singlethreaded/struct.BudgetedFuture.html\" title=\"struct budget_executor::asynchronous::singlethreaded::BudgetedFuture\">BudgetedFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::singlethreaded::BudgetedFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"enum\" href=\"budget_executor/asynchronous/singlethreaded/enum.Progress.html\" title=\"enum budget_executor::asynchronous::singlethreaded::Progress\">Progress</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;F as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html#associatedtype.Output\" title=\"type core::future::future::Future::Output\">Output</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::asynchronous::singlethreaded::Progress"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/singlethreaded/struct.IncompleteFuture.html\" title=\"struct budget_executor::asynchronous::singlethreaded::IncompleteFuture\">IncompleteFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::singlethreaded::IncompleteFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/singlethreaded/struct.WaitForBudgetFuture.html\" title=\"struct budget_executor::asynchronous::singlethreaded::WaitForBudgetFuture\">WaitForBudgetFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::singlethreaded::WaitForBudgetFuture"]},{"text":"impl&lt;Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/blocking/threadsafe/struct.Runtime.html\" title=\"struct budget_executor::blocking::threadsafe::Runtime\">Runtime</a>&lt;Budget&gt;","synthetic":true,"types":["budget_executor::blocking::threadsafe::Runtime"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"enum\" href=\"budget_executor/blocking/threadsafe/enum.Progress.html\" title=\"enum budget_executor::blocking::threadsafe::Progress\">Progress</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;F as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html#associatedtype.Output\" title=\"type core::future::future::Future::Output\">Output</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::blocking::threadsafe::Progress"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/blocking/threadsafe/struct.IncompleteFuture.html\" title=\"struct budget_executor::blocking::threadsafe::IncompleteFuture\">IncompleteFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::blocking::threadsafe::IncompleteFuture"]},{"text":"impl&lt;Output&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/blocking/threadsafe/struct.TaskHandle.html\" title=\"struct budget_executor::blocking::threadsafe::TaskHandle\">TaskHandle</a>&lt;Output&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Output: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::blocking::threadsafe::TaskHandle"]},{"text":"impl&lt;Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/blocking/singlethreaded/struct.Runtime.html\" title=\"struct budget_executor::blocking::singlethreaded::Runtime\">Runtime</a>&lt;Budget&gt;","synthetic":true,"types":["budget_executor::blocking::singlethreaded::Runtime"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"enum\" href=\"budget_executor/blocking/singlethreaded/enum.Progress.html\" title=\"enum budget_executor::blocking::singlethreaded::Progress\">Progress</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;&lt;F as <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html#associatedtype.Output\" title=\"type core::future::future::Future::Output\">Output</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::blocking::singlethreaded::Progress"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/blocking/singlethreaded/struct.IncompleteFuture.html\" title=\"struct budget_executor::blocking::singlethreaded::IncompleteFuture\">IncompleteFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::blocking::singlethreaded::IncompleteFuture"]},{"text":"impl&lt;Output&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/blocking/singlethreaded/struct.TaskHandle.html\" title=\"struct budget_executor::blocking::singlethreaded::TaskHandle\">TaskHandle</a>&lt;Output&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Output: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::blocking::singlethreaded::TaskHandle"]},{"text":"impl&lt;'a, Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/spend/threadsafe/struct.SpendBudget.html\" title=\"struct budget_executor::spend::threadsafe::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt;","synthetic":true,"types":["budget_executor::spend::threadsafe::SpendBudget"]},{"text":"impl&lt;'a, Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/spend/singlethreaded/struct.SpendBudget.html\" title=\"struct budget_executor::spend::singlethreaded::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt;","synthetic":true,"types":["budget_executor::spend::singlethreaded::SpendBudget"]},{"text":"impl&lt;T, Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/struct.BudgetResult.html\" title=\"struct budget_executor::BudgetResult\">BudgetResult</a>&lt;T, Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::BudgetResult"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> for <a class=\"struct\" href=\"budget_executor/struct.ReplenishableBudget.html\" title=\"struct budget_executor::ReplenishableBudget\">ReplenishableBudget</a>","synthetic":true,"types":["budget_executor::ReplenishableBudget"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()