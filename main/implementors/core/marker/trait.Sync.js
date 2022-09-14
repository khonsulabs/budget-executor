(function() {var implementors = {};
implementors["budget_executor"] = [{"text":"impl&lt;Budget, F&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/struct.BudgetedFuture.html\" title=\"struct budget_executor::asynchronous::BudgetedFuture\">BudgetedFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::BudgetedFuture"]},{"text":"impl&lt;Budget, F&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"budget_executor/asynchronous/enum.Progress.html\" title=\"enum budget_executor::asynchronous::Progress\">Progress</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::Progress"]},{"text":"impl&lt;Budget, F&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/struct.IncompleteAsyncFuture.html\" title=\"struct budget_executor::asynchronous::IncompleteAsyncFuture\">IncompleteAsyncFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::IncompleteAsyncFuture"]},{"text":"impl&lt;Budget, F&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/struct.WaitForBudgetFuture.html\" title=\"struct budget_executor::asynchronous::WaitForBudgetFuture\">WaitForBudgetFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::asynchronous::WaitForBudgetFuture"]},{"text":"impl&lt;Budget&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/blocking/struct.Runtime.html\" title=\"struct budget_executor::blocking::Runtime\">Runtime</a>&lt;Budget&gt;","synthetic":true,"types":["budget_executor::blocking::Runtime"]},{"text":"impl&lt;Budget, F&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/blocking/struct.IncompleteFuture.html\" title=\"struct budget_executor::blocking::IncompleteFuture\">IncompleteFuture</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::blocking::IncompleteFuture"]},{"text":"impl&lt;Budget, F&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"enum\" href=\"budget_executor/blocking/enum.Progress.html\" title=\"enum budget_executor::blocking::Progress\">Progress</a>&lt;Budget, F&gt;","synthetic":true,"types":["budget_executor::blocking::Progress"]},{"text":"impl&lt;Output&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/blocking/struct.TaskHandle.html\" title=\"struct budget_executor::blocking::TaskHandle\">TaskHandle</a>&lt;Output&gt;","synthetic":true,"types":["budget_executor::blocking::TaskHandle"]},{"text":"impl&lt;Budget&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/struct.BudgetContext.html\" title=\"struct budget_executor::BudgetContext\">BudgetContext</a>&lt;Budget&gt;","synthetic":true,"types":["budget_executor::BudgetContext"]},{"text":"impl&lt;T, Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/struct.BudgetResult.html\" title=\"struct budget_executor::BudgetResult\">BudgetResult</a>&lt;T, Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a>,&nbsp;</span>","synthetic":true,"types":["budget_executor::BudgetResult"]},{"text":"impl&lt;'a, Budget&gt; !<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/struct.SpendBudget.html\" title=\"struct budget_executor::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt;","synthetic":true,"types":["budget_executor::SpendBudget"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Sync.html\" title=\"trait core::marker::Sync\">Sync</a> for <a class=\"struct\" href=\"budget_executor/struct.ReplenishableBudget.html\" title=\"struct budget_executor::ReplenishableBudget\">ReplenishableBudget</a>","synthetic":true,"types":["budget_executor::ReplenishableBudget"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()