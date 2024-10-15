use criterion::{criterion_group, criterion_main, Criterion};


fn bench_play_game(c: &mut Criterion) {
    c.bench_function("bench_play_game", |b| {
        b.iter(|| {
            std::hint::black_box(for i in 1..=100 {
                play_game(i, false)
            });
        });
    });
}

criterion_group!(
    benches,
    bench_play_game,
);
criterion_main!(benches);

fn play_game(i: i32, is_debug: bool) {
    println!("play game: {}", i);
}