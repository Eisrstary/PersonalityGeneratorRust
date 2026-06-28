//! PAPS 性能基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use personality_generator::api::PersonalitySystem;

fn bench_system_creation(c: &mut Criterion) {
    c.bench_function("system_creation", |b| {
        b.iter(|| PersonalitySystem::new())
    });
}

fn bench_set_get_value(c: &mut Criterion) {
    let mut system = PersonalitySystem::new();
    c.bench_function("set_get_value", |b| {
        b.iter(|| {
            system.set_value("A001", black_box(0.7)).unwrap();
            system.get_value("A001").unwrap()
        })
    });
}

fn bench_coupling_analysis(c: &mut Criterion) {
    let mut system = PersonalitySystem::new();
    // Setup some values that will trigger couplings
    system.set_value("A009", 0.8).unwrap();
    system.set_value("B015", 0.8).unwrap();
    system.set_value("A008", 0.7).unwrap();
    system.set_value("B019", 0.7).unwrap();

    c.bench_function("coupling_analysis", |b| {
        b.iter(|| system.analyze_couplings())
    });
}

fn bench_advance_time(c: &mut Criterion) {
    let mut system = PersonalitySystem::new();
    system.set_drift_rate("B015", -0.01).unwrap();

    c.bench_function("advance_time_100days", |b| {
        b.iter(|| {
            let mut s = PersonalitySystem::new();
            s.set_drift_rate("B015", -0.01).unwrap();
            s.advance_time(100.0)
        })
    });
}

fn bench_relationship_collapse(c: &mut Criterion) {
    let mut system = PersonalitySystem::new();
    system.set_value("B015", 0.7).unwrap();
    system.add_relationship("test", "intimate");

    c.bench_function("relationship_collapse", |b| {
        b.iter(|| system.collapse_in_relationship("B015", "test"))
    });
}

criterion_group!(
    benches,
    bench_system_creation,
    bench_set_get_value,
    bench_coupling_analysis,
    bench_advance_time,
    bench_relationship_collapse,
);
criterion_main!(benches);
