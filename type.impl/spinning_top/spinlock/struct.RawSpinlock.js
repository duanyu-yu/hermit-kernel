(function() {
    var type_impls = Object.fromEntries([["hermit_sync",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-RawSpinlock%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#28\">Source</a><a href=\"#impl-Debug-for-RawSpinlock%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"spinning_top/spinlock/struct.RawSpinlock.html\" title=\"struct spinning_top::spinlock::RawSpinlock\">RawSpinlock</a>&lt;R&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + <a class=\"trait\" href=\"spinning_top/relax/trait.Relax.html\" title=\"trait spinning_top::relax::Relax\">Relax</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#28\">Source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","hermit_sync::mutex::spin::RawSpinMutex"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-RawMutex-for-RawSpinlock%3CR%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#47\">Source</a><a href=\"#impl-RawMutex-for-RawSpinlock%3CR%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;R&gt; <a class=\"trait\" href=\"lock_api/mutex/trait.RawMutex.html\" title=\"trait lock_api::mutex::RawMutex\">RawMutex</a> for <a class=\"struct\" href=\"spinning_top/spinlock/struct.RawSpinlock.html\" title=\"struct spinning_top::spinlock::RawSpinlock\">RawSpinlock</a>&lt;R&gt;<div class=\"where\">where\n    R: <a class=\"trait\" href=\"spinning_top/relax/trait.Relax.html\" title=\"trait spinning_top::relax::Relax\">Relax</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedconstant.INIT\" class=\"associatedconstant trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#48\">Source</a><a href=\"#associatedconstant.INIT\" class=\"anchor\">§</a><h4 class=\"code-header\">const <a href=\"lock_api/mutex/trait.RawMutex.html#associatedconstant.INIT\" class=\"constant\">INIT</a>: <a class=\"struct\" href=\"spinning_top/spinlock/struct.RawSpinlock.html\" title=\"struct spinning_top::spinlock::RawSpinlock\">RawSpinlock</a>&lt;R&gt; = _</h4></section></summary><div class='docblock'>Initial value for an unlocked mutex.</div></details><details class=\"toggle\" open><summary><section id=\"associatedtype.GuardMarker\" class=\"associatedtype trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#54\">Source</a><a href=\"#associatedtype.GuardMarker\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"lock_api/mutex/trait.RawMutex.html#associatedtype.GuardMarker\" class=\"associatedtype\">GuardMarker</a> = <a class=\"struct\" href=\"lock_api/struct.GuardSend.html\" title=\"struct lock_api::GuardSend\">GuardSend</a></h4></section></summary><div class='docblock'>Marker type which determines whether a lock guard should be <code>Send</code>. Use\none of the <code>GuardSend</code> or <code>GuardNoSend</code> helper types here.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.lock\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#57\">Source</a><a href=\"#method.lock\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lock_api/mutex/trait.RawMutex.html#tymethod.lock\" class=\"fn\">lock</a>(&amp;self)</h4></section></summary><div class='docblock'>Acquires this mutex, blocking the current thread until it is able to do so.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.try_lock\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#71\">Source</a><a href=\"#method.try_lock\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lock_api/mutex/trait.RawMutex.html#tymethod.try_lock\" class=\"fn\">try_lock</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Attempts to acquire this mutex without blocking. Returns <code>true</code>\nif the lock was successfully acquired and <code>false</code> otherwise.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.unlock\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#87\">Source</a><a href=\"#method.unlock\" class=\"anchor\">§</a><h4 class=\"code-header\">unsafe fn <a href=\"lock_api/mutex/trait.RawMutex.html#tymethod.unlock\" class=\"fn\">unlock</a>(&amp;self)</h4></section></summary><div class='docblock'>Unlocks this mutex. <a href=\"lock_api/mutex/trait.RawMutex.html#tymethod.unlock\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_locked\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/spinning_top/spinlock.rs.html#92\">Source</a><a href=\"#method.is_locked\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"lock_api/mutex/trait.RawMutex.html#method.is_locked\" class=\"fn\">is_locked</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/core/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Checks whether the mutex is currently locked.</div></details></div></details>","RawMutex","hermit_sync::mutex::spin::RawSpinMutex"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[6784]}