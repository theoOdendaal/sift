use std::time::Duration;

use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};


fn xml_tokenization(input: &[u8]) -> usize {
    sift::xml::tokens::XmlTokenizer::from(input).count()
}

fn benchmark_xml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("xml_tokenizer");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));

    //let input = std::fs::read(std::path::Path::new("tests/xmlconf/xmlconf.xml")).unwrap();
    let input = std::fs::read(std::path::Path::new("test_files/discogs_20260101_artists.xml")).unwrap();

    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("Xml tokenization", |b| { b.iter(|| xml_tokenization(black_box(&input))) });

    group.finish();
}

criterion_group!(benches, benchmark_xml_parsing);
criterion_main!(benches);
