use criterion::{criterion_group, criterion_main, Criterion};

use rand::rngs::SmallRng;
use rand::SeedableRng;

use santorini_ai::mcts::santorini::{SantoriniNode, SantoriniSimulation};
use santorini_ai::mcts::{Node, Simulation};
use santorini_ai::player::mcts_ai::MctsSantoriniParams;
use santorini_ai::santorini::{self, Point};

fn default_node() -> SantoriniNode {
    let g = santorini::new_game();
    let p1 = Point::new(1.into(), 1.into());
    let p2 = Point::new(2.into(), 1.into());
    let p3 = Point::new(1.into(), 2.into());
    let p4 = Point::new(2.into(), 2.into());

    let action = g.can_place(p1, p2).expect("Invalid placement");
    let g = g.apply(action);
    let action = g.can_place(p3, p4).expect("Invalid placement");
    g.apply(action).into()
}

fn criterion_benchmark(c: &mut Criterion) {
    let s_node = default_node();
    let mut rng = SmallRng::from_entropy();

    {
        let mut group = c.benchmark_group("small");
        group.sample_size(500);
        group.bench_function("simulate", |b| {
            b.iter(|| SantoriniSimulation {}.simulate(&s_node, &mut rng))
        });
    }

    let mut params = MctsSantoriniParams::default();
    let node = Node::new(&mut params, s_node);
    c.bench_function("one step", |b| {
        b.iter(|| {
            let mut n2 = node.clone();
            n2.step(&mut params);
            n2
        })
    });

    let mut group = c.benchmark_group("large");
    group.sample_size(20);
    group.bench_function("ten step", |b| {
        b.iter(|| {
            let mut n2 = node.clone();
            for _ in 0..10 {
                n2.step(&mut params);
            }
            n2
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
