use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;

fn benchmark_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");
    
    let product_data = json!({
        "id": "PROD-001",
        "name": "Test Product",
        "description": "A test product for benchmarking",
        "origin_location": "Nairobi",
        "category": "Food",
        "tags": ["organic", "fair-trade"],
        "certifications": ["ISO-9001"],
        "media_hashes": [],
        "custom": {}
    });

    group.bench_function("serialize_product", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&product_data).unwrap())
        });
    });

    group.finish();
}

fn benchmark_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashing");
    
    let data = b"test data for hashing benchmark";
    
    group.bench_function("sha256_hash", |b| {
        use sha2::{Sha256, Digest};
        b.iter(|| {
            let mut hasher = Sha256::new();
            black_box(hasher.update(data));
            black_box(hasher.finalize())
        });
    });

    group.finish();
}

fn benchmark_uuid_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("uuid_generation");
    
    group.bench_function("uuid_v4", |b| {
        b.iter(|| {
            black_box(uuid::Uuid::new_v4())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_json_serialization,
    benchmark_hashing,
    benchmark_uuid_generation
);
criterion_main!(benches);
