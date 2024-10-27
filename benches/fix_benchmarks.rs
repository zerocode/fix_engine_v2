use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fix_engine::{FixField, FixMessage};

fn encode_benchmark(c: &mut Criterion) {
    let mut msg = FixMessage::new();
    msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
    msg.add_field(FixField::new(9, b"100".to_vec()));
    msg.add_field(FixField::new(35, b"D".to_vec()));
    msg.add_field(FixField::new(49, b"SENDER".to_vec()));
    msg.add_field(FixField::new(56, b"TARGET".to_vec()));
    msg.add_field(FixField::new(34, b"1".to_vec()));
    msg.add_field(FixField::new(52, b"20240101-12:00:00.000".to_vec()));

    c.bench_function("encode_message", |b| {
        b.iter(|| black_box(msg.clone()).encode())
    });
}

fn decode_benchmark(c: &mut Criterion) {
    let mut msg = FixMessage::new();
    msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
    msg.add_field(FixField::new(9, b"100".to_vec()));
    msg.add_field(FixField::new(35, b"D".to_vec()));
    msg.add_field(FixField::new(49, b"SENDER".to_vec()));
    msg.add_field(FixField::new(56, b"TARGET".to_vec()));
    msg.add_field(FixField::new(34, b"1".to_vec()));
    msg.add_field(FixField::new(52, b"20240101-12:00:00.000".to_vec()));

    let encoded = msg.encode().unwrap();

    c.bench_function("decode_message", |b| {
        b.iter(|| FixMessage::decode(black_box(&encoded)))
    });
}

criterion_group!(benches, encode_benchmark, decode_benchmark);
criterion_main!(benches);