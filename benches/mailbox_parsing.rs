use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lettre::message::{Mailbox, Mailboxes};

fn bench_parse_single(mailbox: &str) {
    assert!(mailbox.parse::<Mailbox>().is_ok());
}

fn bench_parse_multiple(mailboxes: &str) {
    assert!(mailboxes.parse::<Mailboxes>().is_ok());
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse single mailbox", |b| {
        b.iter(|| bench_parse_single(black_box("\"Benchmark test\" <test@mail.local>")))
    });

    c.bench_function("parse multiple mailboxes", |b| {
        b.iter(|| {
            bench_parse_multiple(black_box(
                "\"Benchmark test\" <test@mail.local>, Test <test@mail.local>, <test@mail.local>, test@mail.local",
            ))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
