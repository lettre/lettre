use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};

fn bench_simple_send(c: &mut Criterion) {
    let sender = SmtpTransport::builder_dangerous("127.0.0.1")
        .port(2525)
        .build();

    c.bench_function("send email", move |b| {
        b.iter(|| {
            let email = Message::builder()
                .from("NoBody <nobody@domain.tld>".parse().unwrap())
                .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
                .to("Hei <hei@domain.tld>".parse().unwrap())
                .subject("Happy new year")
                .header(ContentType::TEXT_PLAIN)
                .body(String::from("Be happy!"))
                .unwrap();
            let result = black_box(sender.send(&email));
            assert!(result.is_ok());
        })
    });
}

fn bench_reuse_send(c: &mut Criterion) {
    let sender = SmtpTransport::builder_dangerous("127.0.0.1")
        .port(2525)
        .build();
    c.bench_function("send email with connection reuse", move |b| {
        b.iter(|| {
            let email = Message::builder()
                .from("NoBody <nobody@domain.tld>".parse().unwrap())
                .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
                .to("Hei <hei@domain.tld>".parse().unwrap())
                .subject("Happy new year")
                .header(ContentType::TEXT_PLAIN)
                .body(String::from("Be happy!"))
                .unwrap();
            let result = black_box(sender.send(&email));
            assert!(result.is_ok());
        })
    });
}

criterion_group!(benches, bench_simple_send, bench_reuse_send);
criterion_main!(benches);
