//! Criterion benchmarks for phenotype-policy-engine crate.
//!
//! Benchmarks policy evaluation performance with varying rule counts and complexity.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Benchmark policy engine evaluation with varying policy counts
fn bench_policy_evaluation(c: &mut Criterion) {
    use phenotype_policy_engine::{EvaluationContext, Policy, PolicyEngine, Rule, RuleType};

    let mut group = c.benchmark_group("policy_evaluation");

    // Benchmark with different policy counts
    for policy_count in [1, 10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*policy_count as u64));
        group.bench_with_input(
            BenchmarkId::new("evaluate_all_policies", policy_count),
            policy_count,
            |b, &count| {
                let engine = PolicyEngine::new();

                // Add policies
                for i in 0..count {
                    let policy = Policy::new(format!("policy-{}", i)).add_rule(Rule::new(
                        RuleType::Require,
                        "field",
                        ".*",
                    ));
                    engine.add_policy(policy);
                }

                // Context with required field
                let mut ctx = EvaluationContext::new();
                ctx.set_string("field", "value");

                b.iter(|| {
                    let result = engine.evaluate_all(&ctx).unwrap();
                    black_box(result.passed);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark policy evaluation with varying rule counts per policy
fn bench_policy_rules(c: &mut Criterion) {
    use phenotype_policy_engine::{EvaluationContext, Policy, PolicyEngine, Rule, RuleType};

    let mut group = c.benchmark_group("policy_rules");

    for rule_count in [1, 5, 10, 25].iter() {
        group.throughput(Throughput::Elements(*rule_count as u64));
        group.bench_with_input(
            BenchmarkId::new("rules_per_policy", rule_count),
            rule_count,
            |b, &count| {
                let engine = PolicyEngine::new();

                // Create policy with multiple rules
                let mut policy = Policy::new("multi-rule-policy");
                for i in 0..count {
                    policy =
                        policy.add_rule(Rule::new(RuleType::Require, format!("field-{}", i), ".*"));
                }
                engine.add_policy(policy);

                // Context with all required fields
                let mut ctx = EvaluationContext::new();
                for i in 0..count {
                    ctx.set_string(format!("field-{}", i), format!("value-{}", i));
                }

                b.iter(|| {
                    let result = engine.evaluate_all(&ctx).unwrap();
                    black_box(result.passed);
                })
            },
        );
    }

    group.finish();
}

/// Benchmark different rule types
fn bench_rule_types(c: &mut Criterion) {
    use phenotype_policy_engine::{EvaluationContext, Policy, PolicyEngine, Rule, RuleType};

    let mut group = c.benchmark_group("rule_types");

    // Require rule
    group.bench_function("require_rule_pass", |b| {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test").add_rule(Rule::new(RuleType::Require, "email", ".*"));
        engine.add_policy(policy);

        let mut ctx = EvaluationContext::new();
        ctx.set_string("email", "test@example.com");

        b.iter(|| {
            let result = engine.evaluate_all(&ctx).unwrap();
            black_box(result.passed);
        })
    });

    group.bench_function("require_rule_fail", |b| {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test").add_rule(Rule::new(RuleType::Require, "email", ".*"));
        engine.add_policy(policy);

        let ctx = EvaluationContext::new(); // Missing required field

        b.iter(|| {
            let result = engine.evaluate_all(&ctx).unwrap();
            black_box(result.passed);
        })
    });

    // Pattern matching rule
    group.bench_function("pattern_rule", |b| {
        let engine = PolicyEngine::new();
        let policy =
            Policy::new("test").add_rule(Rule::new(RuleType::Pattern, "email", r"^[^@]+@[^@]+$"));
        engine.add_policy(policy);

        let mut ctx = EvaluationContext::new();
        ctx.set_string("email", "test@example.com");

        b.iter(|| {
            let result = engine.evaluate_all(&ctx).unwrap();
            black_box(result.passed);
        })
    });

    // Range rule
    group.bench_function("range_rule", |b| {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test").add_rule(Rule::new(RuleType::Range, "age", "18-65"));
        engine.add_policy(policy);

        let mut ctx = EvaluationContext::new();
        ctx.set_number("age", 25.0);

        b.iter(|| {
            let result = engine.evaluate_all(&ctx).unwrap();
            black_box(result.passed);
        })
    });

    group.finish();
}

/// Benchmark policy engine CRUD operations
fn bench_policy_crud(c: &mut Criterion) {
    use phenotype_policy_engine::{Policy, PolicyEngine};

    let mut group = c.benchmark_group("policy_crud");

    group.bench_function("add_policy", |b| {
        let engine = PolicyEngine::new();
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            let policy = Policy::new(format!("policy-{}", counter));
            engine.add_policy(policy);
        })
    });

    group.bench_function("get_policy", |b| {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("test-policy"));

        b.iter(|| {
            let result = engine.get_policy("test-policy");
            black_box(result);
        })
    });

    group.bench_function("remove_policy", |b| {
        let engine = PolicyEngine::new();
        let mut counter = 0u64;

        b.iter(|| {
            counter += 1;
            let name = format!("policy-{}", counter);
            engine.add_policy(Policy::new(&name));
            let result = engine.remove_policy(&name);
            black_box(result);
        })
    });

    group.bench_function("enable_disable_policy", |b| {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("toggle-policy"));

        b.iter(|| {
            engine.enable_policy("toggle-policy").unwrap();
            engine.disable_policy("toggle-policy").unwrap();
        })
    });

    group.finish();
}

/// Benchmark single vs subset evaluation
fn bench_evaluation_modes(c: &mut Criterion) {
    use phenotype_policy_engine::{EvaluationContext, Policy, PolicyEngine, Rule, RuleType};

    let mut group = c.benchmark_group("evaluation_modes");

    // Setup: 50 policies
    let engine = PolicyEngine::new();
    for i in 0..50 {
        let policy = Policy::new(format!("policy-{}", i)).add_rule(Rule::new(
            RuleType::Require,
            "field",
            ".*",
        ));
        engine.add_policy(policy);
    }

    let mut ctx = EvaluationContext::new();
    ctx.set_string("field", "value");

    group.bench_function("evaluate_single", |b| {
        b.iter(|| {
            let result = engine.evaluate_single("policy-0", &ctx).unwrap();
            black_box(result.passed);
        })
    });

    group.bench_function("evaluate_subset_10", |b| {
        let names: Vec<&str> = (0..10).map(|i| "policy-0").collect();
        b.iter(|| {
            let result = engine.evaluate_subset(&["policy-0"], &ctx).unwrap();
            black_box(result.passed);
        })
    });

    group.bench_function("evaluate_all_50", |b| {
        b.iter(|| {
            let result = engine.evaluate_all(&ctx).unwrap();
            black_box(result.passed);
        })
    });

    group.finish();
}

criterion_group!(
    policy_engine_benches,
    bench_policy_evaluation,
    bench_policy_rules,
    bench_rule_types,
    bench_policy_crud,
    bench_evaluation_modes
);
criterion_main!(policy_engine_benches);
