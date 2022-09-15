(function() {var implementors = {};
implementors["budget_executor"] = [{"text":"impl&lt;Budget:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"budget_executor/blocking/threadsafe/struct.Runtime.html\" title=\"struct budget_executor::blocking::threadsafe::Runtime\">Runtime</a>&lt;Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::blocking::threadsafe::Runtime"]},{"text":"impl&lt;Budget:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"budget_executor/blocking/singlethreaded/struct.Runtime.html\" title=\"struct budget_executor::blocking::singlethreaded::Runtime\">Runtime</a>&lt;Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::blocking::singlethreaded::Runtime"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"budget_executor/struct.ReplenishableBudget.html\" title=\"struct budget_executor::ReplenishableBudget\">ReplenishableBudget</a>","synthetic":false,"types":["budget_executor::replenishable::ReplenishableBudget"]},{"text":"impl&lt;'a, Budget:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"budget_executor/spend/threadsafe/struct.SpendBudget.html\" title=\"struct budget_executor::spend::threadsafe::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::spend::threadsafe::SpendBudget"]},{"text":"impl&lt;'a, Budget:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"budget_executor/spend/singlethreaded/struct.SpendBudget.html\" title=\"struct budget_executor::spend::singlethreaded::SpendBudget\">SpendBudget</a>&lt;'a, Budget&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Budget: <a class=\"trait\" href=\"budget_executor/trait.Budgetable.html\" title=\"trait budget_executor::Budgetable\">Budgetable</a>,&nbsp;</span>","synthetic":false,"types":["budget_executor::spend::singlethreaded::SpendBudget"]},{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>, Budget:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.63.0/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"budget_executor/struct.BudgetResult.html\" title=\"struct budget_executor::BudgetResult\">BudgetResult</a>&lt;T, Budget&gt;","synthetic":false,"types":["budget_executor::BudgetResult"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()