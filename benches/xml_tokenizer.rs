use std::time::Duration;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};


fn strategy_a(input: &[u8]) -> usize {
    sift::xml::tokens::XmlTokenizer::from(input).count()
}

fn strategy_b(input: &[u8]) -> usize {
    sift::xml::new_xml::XmlTokenizer::from(input).count()
}

fn compare_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_tokenizer");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    //let input = std::fs::read(std::path::Path::new("tests/xmlconf/xmlconf.xml")).unwrap();
    let input = std::fs::read(std::path::Path::new("test_files/discogs_20260101_artists.xml")).unwrap();

    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("Old", |b| { b.iter(|| strategy_a(black_box(&input))) });
    group.bench_function("New", |b| { b.iter(|| strategy_b(black_box(&input))) });

    group.finish();
}

criterion_group!(benches, compare_strategies);
criterion_main!(benches);
