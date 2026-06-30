use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use loro::LoroDoc;

fn make_snapshot(ops: u32) -> Vec<u8> {
    let doc = LoroDoc::new();
    let map = doc.get_map("root");
    for i in 0..ops {
        map.insert(&format!("key_{i}"), format!("val_{i}"))
            .expect("insert");
    }
    doc.export(loro::ExportMode::Snapshot)
        .expect("export snapshot")
}

fn make_delta(ops: u32, prefix: &str) -> Vec<u8> {
    let doc = LoroDoc::new();
    let map = doc.get_map("root");
    for i in 0..ops {
        map.insert(&format!("{prefix}_{i}"), format!("v_{i}"))
            .expect("insert");
    }
    doc.export(loro::ExportMode::all_updates())
        .expect("export update")
}

fn bench_apply_delta(c: &mut Criterion) {
    let mut group = c.benchmark_group("apply_delta");

    for op_count in [1u32, 100, 1000] {
        let base = make_snapshot(op_count);
        let delta = make_delta(op_count / 10 + 1, "new");

        group.bench_with_input(
            BenchmarkId::new("op_count", op_count),
            &(base.as_slice(), delta.as_slice()),
            |b, (base, delta)| {
                b.iter(|| {
                    frf_crdt::apply_delta(criterion::black_box(base), criterion::black_box(delta))
                        .expect("apply_delta");
                });
            },
        );
    }

    group.finish();
}

fn bench_apply_delta_empty_base(c: &mut Criterion) {
    let deltas: Vec<(u32, Vec<u8>)> = [1u32, 100, 500]
        .iter()
        .map(|&n| (n, make_delta(n, "k")))
        .collect();

    let mut group = c.benchmark_group("apply_delta_empty_base");
    for (n, delta) in &deltas {
        group.bench_with_input(BenchmarkId::new("op_count", n), delta, |b, delta| {
            b.iter(|| {
                frf_crdt::apply_delta(criterion::black_box(&[]), criterion::black_box(delta))
                    .expect("apply_delta");
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_apply_delta, bench_apply_delta_empty_base);
criterion_main!(benches);
