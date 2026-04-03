//! Criterion benchmarks for phenotype-state-machine crate.
//!
//! Benchmarks state machine transitions, guarded transitions, and callbacks.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark basic state transitions
fn bench_state_transitions(c: &mut Criterion) {
    use phenotype_state_machine::{StateMachine, StateMachineBuilder};

    let mut group = c.benchmark_group("state_transitions");

    // Simple 3-state machine (traffic light)
    group.bench_function("simple_transition", |b| {
        let sm = StateMachineBuilder::new("red")
            .transition("red", "next", "green")
            .transition("green", "next", "yellow")
            .transition("yellow", "next", "red")
            .build()
            .unwrap();

        b.iter(|| {
            sm.send("next").unwrap();
        })
    });

    // Get current state
    group.bench_function("current_state", |b| {
        let sm = StateMachineBuilder::new("idle")
            .transition("idle", "start", "running")
            .build()
            .unwrap();

        b.iter(|| {
            let state = sm.current();
            black_box(state);
        })
    });

    // Check if transition is possible
    group.bench_function("can_send", |b| {
        let sm = StateMachineBuilder::new("idle")
            .transition("idle", "start", "running")
            .transition("running", "stop", "idle")
            .build()
            .unwrap();

        b.iter(|| {
            let can = sm.can_send("start");
            black_box(can);
        })
    });

    group.finish();
}

/// Benchmark guarded transitions
fn bench_guarded_transitions(c: &mut Criterion) {
    use phenotype_state_machine::StateMachineBuilder;

    let mut group = c.benchmark_group("guarded_transitions");

    // Guard that always allows
    group.bench_function("guard_allow", |b| {
        let sm = StateMachineBuilder::new("locked")
            .guarded_transition("locked", "unlock", "unlocked", |_, _| true)
            .build()
            .unwrap();

        b.iter(|| {
            sm.send("unlock").unwrap();
            // Reset state by creating a new transition
            let _ = sm.send("lock");
        })
    });

    // Guard that checks condition
    group.bench_function("guard_with_condition", |b| {
        use std::sync::atomic::{AtomicBool, Ordering};

        let allowed = AtomicBool::new(true);
        let sm = StateMachineBuilder::new("pending")
            .guarded_transition("pending", "approve", "approved", move |_, _| {
                allowed.load(Ordering::SeqCst)
            })
            .build()
            .unwrap();

        b.iter(|| {
            let result = sm.send("approve");
            black_box(result.is_ok());
        })
    });

    group.finish();
}

/// Benchmark callbacks (on_enter, on_exit)
fn bench_callbacks(c: &mut Criterion) {
    use phenotype_state_machine::StateMachineBuilder;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let mut group = c.benchmark_group("callbacks");

    group.bench_function("on_enter_callback", |b| {
        let counter = AtomicUsize::new(0);
        let sm = StateMachineBuilder::new("a")
            .transition("a", "go", "b")
            .on_enter("b", |_state| {
                // Simulating some work
                black_box(1);
            })
            .build()
            .unwrap();

        b.iter(|| {
            sm.send("go").unwrap();
            // Reset
            let _ = sm.send("reset");
        })
    });

    group.bench_function("on_exit_callback", |b| {
        let sm = StateMachineBuilder::new("a")
            .transition("a", "go", "b")
            .on_exit("a", |_state| {
                // Simulating some work
                black_box(1);
            })
            .build()
            .unwrap();

        b.iter(|| {
            sm.send("go").unwrap();
            // Reset
            let _ = sm.send("reset");
        })
    });

    group.bench_function("multiple_callbacks", |b| {
        let sm = StateMachineBuilder::new("a")
            .transition("a", "go", "b")
            .on_exit("a", |_state| black_box(1))
            .on_enter("b", |_state| black_box(2))
            .on_exit("b", |_state| black_box(3))
            .build()
            .unwrap();

        b.iter(|| {
            sm.send("go").unwrap();
            // Reset
            let _ = sm.send("reset");
        })
    });

    group.finish();
}

/// Benchmark state machines with varying complexity
fn bench_complexity_scaling(c: &mut Criterion) {
    use phenotype_state_machine::StateMachineBuilder;

    let mut group = c.benchmark_group("complexity_scaling");

    for state_count in [5, 10, 25, 50].iter() {
        group.throughput(Throughput::Elements(*state_count as u64));
        group.bench_with_input(
            BenchmarkId::new("many_states", state_count),
            state_count,
            |b, &count| {
                let mut builder = StateMachineBuilder::new("state-0");

                // Create a cycle of states
                for i in 0..count {
                    let from = format!("state-{}", i);
                    let to = format!("state-{}", (i + 1) % count);
                    builder = builder.transition(&from, "next", &to);
                }

                let sm = builder.build().unwrap();

                b.iter(|| {
                    sm.send("next").unwrap();
                })
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent access (StateMachine is Send + Sync)
fn bench_concurrent_access(c: &mut Criterion) {
    use phenotype_state_machine::StateMachineBuilder;
    use std::sync::Arc;
    use std::thread;

    c.bench_function("concurrent_transitions", |b| {
        let sm = Arc::new(
            StateMachineBuilder::new("idle")
                .transition("idle", "start", "running")
                .transition("running", "stop", "idle")
                .build()
                .unwrap(),
        );

        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let sm = sm.clone();
                    thread::spawn(move || {
                        for _ in 0..25 {
                            let _ = sm.send("start");
                            let _ = sm.send("stop");
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        })
    });
}

/// Benchmark building state machines
fn bench_builder(c: &mut Criterion) {
    use phenotype_state_machine::StateMachineBuilder;

    let mut group = c.benchmark_group("builder");

    group.bench_function("build_simple", |b| {
        b.iter(|| {
            let sm = StateMachineBuilder::new("idle")
                .transition("idle", "start", "running")
                .transition("running", "stop", "idle")
                .build()
                .unwrap();
            black_box(sm);
        })
    });

    group.bench_function("build_complex", |b| {
        b.iter(|| {
            let mut builder = StateMachineBuilder::new("s0");
            for i in 0..50 {
                let from = format!("s{}", i);
                let to = format!("s{}", i + 1);
                builder = builder.transition(&from, "next", &to);
            }
            let sm = builder.build().unwrap();
            black_box(sm);
        })
    });

    group.finish();
}

criterion_group!(
    state_machine_benches,
    bench_state_transitions,
    bench_guarded_transitions,
    bench_callbacks,
    bench_complexity_scaling,
    bench_concurrent_access,
    bench_builder
);
criterion_main!(state_machine_benches);
