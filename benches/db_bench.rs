use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use kvserver::ShardedDb;
use tokio::runtime::Runtime;

fn make_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("failed to build tokio runtime for benchmark")
}

fn setup_db(rt: &Runtime, shards: usize, key_count: usize) -> Arc<ShardedDb> {
    let db = Arc::new(ShardedDb::new(shards));
    rt.block_on(async {
        for i in 0..key_count {
            let key = format!("k{i}");
            db.put(key, b"seed-value".to_vec()).await;
        }
    });
    db
}

fn bench_get_uniform(c: &mut Criterion) {
    let rt = make_runtime();
    let key_count = 10_000;
    let keys: Vec<String> = (0..key_count).map(|i| format!("k{i}")).collect();
    let db = setup_db(&rt, 16, key_count);
    let cursor = AtomicUsize::new(0);

    c.bench_function("get_uniform_10k_keys", |b| {
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            let idx = cursor.fetch_add(1, Ordering::Relaxed) % key_count;
            let key = keys[idx].clone();
            async move {
                let result = db.get(&key).await;
                black_box(result);
            }
        })
    });
}

fn bench_get_hotspot(c: &mut Criterion) {
    let rt = make_runtime();
    let key_count = 10_000;
    let keys: Vec<String> = (0..key_count).map(|i| format!("k{i}")).collect();
    let hotspot_key = "k0".to_string();
    let db = setup_db(&rt, 16, key_count);
    let cursor = AtomicUsize::new(0);

    c.bench_function("get_hotspot_90_10", |b| {
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            let n = cursor.fetch_add(1, Ordering::Relaxed);
            let key = if n % 10 < 9 {
                hotspot_key.clone()
            } else {
                let idx = n % key_count;
                keys[idx].clone()
            };
            async move {
                let result = db.get(&key).await;
                black_box(result);
            }
        })
    });
}

fn bench_mixed_hotspot(c: &mut Criterion) {
    let rt = make_runtime();
    let key_count = 5_000;
    let keys: Vec<String> = (0..key_count).map(|i| format!("k{i}")).collect();
    let db = setup_db(&rt, 16, key_count);
    let cursor = AtomicUsize::new(0);

    c.bench_function("mixed_hotspot_read_write_delete", |b| {
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            let n = cursor.fetch_add(1, Ordering::Relaxed);
            let key = if n % 10 < 8 {
                "k0".to_string()
            } else {
                let idx = n % key_count;
                keys[idx].clone()
            };

            async move {
                match n % 20 {
                    0..=13 => {
                        let result = db.get(&key).await;
                        black_box(result);
                    }
                    14..=17 => {
                        db.put(key, b"updated-value".to_vec()).await;
                    }
                    _ => {
                        let _ = db.delete(&key).await;
                    }
                }
            }
        })
    });
}

fn bench_concurrent_gets(c: &mut Criterion) {
    let rt = make_runtime();
    let key_count = 10_000;
    let keys: Arc<Vec<String>> = Arc::new((0..key_count).map(|i| format!("k{i}")).collect());
    let db = setup_db(&rt, 16, key_count);

    let mut group = c.benchmark_group("concurrent_get_batch");
    group.throughput(Throughput::Elements(64));

    group.bench_function("uniform_64req", |b| {
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            let keys = keys.clone();
            async move {
                let mut handles = Vec::with_capacity(64);
                for i in 0..64 {
                    let db = db.clone();
                    let keys = keys.clone();
                    handles.push(tokio::spawn(async move {
                        let idx = i % keys.len();
                        let result = db.get(&keys[idx]).await;
                        black_box(result);
                    }));
                }

                for handle in handles {
                    let _ = handle.await;
                }
            }
        })
    });

    group.bench_function("hotspot_90_10_64req", |b| {
        b.to_async(&rt).iter(|| {
            let db = db.clone();
            let keys = keys.clone();
            async move {
                let mut handles = Vec::with_capacity(64);
                for i in 0..64 {
                    let db = db.clone();
                    let keys = keys.clone();
                    handles.push(tokio::spawn(async move {
                        let key = if i % 10 < 9 {
                            &keys[0]
                        } else {
                            let idx = i % keys.len();
                            &keys[idx]
                        };
                        let result = db.get(key).await;
                        black_box(result);
                    }));
                }

                for handle in handles {
                    let _ = handle.await;
                }
            }
        })
    });

    group.finish();
}

fn configure_criterion() -> Criterion {
    Criterion::default().sample_size(100)
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = bench_get_uniform, bench_get_hotspot, bench_mixed_hotspot, bench_concurrent_gets
);
criterion_main!(benches);
