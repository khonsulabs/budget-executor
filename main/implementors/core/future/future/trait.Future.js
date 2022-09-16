(function() {var implementors = {};
implementors["budget_executor"] = [{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/threadsafe/struct.BudgetedFuture.html\" title=\"struct budget_executor::asynchronous::threadsafe::BudgetedFuture\">BudgetedFuture</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::asynchronous::threadsafe::BudgetedFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/threadsafe/struct.WaitForBudgetFuture.html\" title=\"struct budget_executor::asynchronous::threadsafe::WaitForBudgetFuture\">WaitForBudgetFuture</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::asynchronous::threadsafe::WaitForBudgetFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/singlethreaded/struct.BudgetedFuture.html\" title=\"struct budget_executor::asynchronous::singlethreaded::BudgetedFuture\">BudgetedFuture</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::asynchronous::singlethreaded::BudgetedFuture"]},{"text":"impl&lt;Budget, F&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/asynchronous/singlethreaded/struct.WaitForBudgetFuture.html\" title=\"struct budget_executor::asynchronous::singlethreaded::WaitForBudgetFuture\">WaitForBudgetFuture</a>&lt;Budget, F&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::asynchronous::singlethreaded::WaitForBudgetFuture"]},{"text":"impl&lt;Output&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/blocking/threadsafe/struct.TaskHandle.html\" title=\"struct budget_executor::blocking::threadsafe::TaskHandle\">TaskHandle</a>&lt;Output&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Output: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + 'static,&nbsp;</span>","synthetic":false,"types":["budget_executor::blocking::threadsafe::TaskHandle"]},{"text":"impl&lt;Output&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/blocking/singlethreaded/struct.TaskHandle.html\" title=\"struct budget_executor::blocking::singlethreaded::TaskHandle\">TaskHandle</a>&lt;Output&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Output: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + 'static,&nbsp;</span>","synthetic":false,"types":["budget_executor::blocking::singlethreaded::TaskHandle"]},{"text":"impl&lt;'a, Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/spend/threadsafe/struct.SpendBudget.html\" title=\"struct budget_executor::spend::threadsafe::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::spend::threadsafe::SpendBudget"]},{"text":"impl&lt;'a, Budget&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/future/future/trait.Future.html\" title=\"trait core::future::future::Future\">Future</a> for <a class=\"struct\" href=\"budget_executor/spend/singlethreaded/struct.SpendBudget.html\" title=\"struct budget_executor::spend::singlethreaded::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::spend::singlethreaded::SpendBudget"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()