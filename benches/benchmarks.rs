use criterion::{black_box, criterion_group, criterion_main, Criterion};

use santorini_ai::santorini::{self, Game, Point, Move};
use santorini_ai::player::mcts_ai;

fn default_game() -> Game<Move> {
    let g = santorini::new_game();
    let p1 = Point::new(1.into(), 1.into());
    let p2 = Point::new(2.into(), 1.into());
    let p3 = Point::new(1.into(), 2.into());
    let p4 = Point::new(2.into(), 2.into());

    let action = g.can_place(p1, p2).expect("Invalid placement");
    let g = g.apply(action);
    let action = g.can_place(p3, p4).expect("Invalid placement");
    g.apply(action)
}

fn criterion_benchmark(c: &mut Criterion) {
    let g = default_game();

    {
        let mut group = c.benchmark_group("small");
        group.sample_size(1000);
        group.bench_function("simulate", |b| b.iter(|| mcts_ai::simulate(g)));
    }

    let n = mcts_ai::Node::new(g);
    c.bench_function("one step", |b| b.iter(|| {
        let mut n2 = n.clone();
        n2.step();
        n2
    }));

    let mut group = c.benchmark_group("large");
    group.sample_size(10);
    group.bench_function("ten step", |b| b.iter(|| {
        let mut n2 = n.clone();
        for _ in 0..10 { n2.step(); }
        n2
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
