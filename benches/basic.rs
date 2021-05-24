use criterion::{criterion_group, criterion_main, Criterion, black_box};

const ENTITIES: usize = 10000;
const ITER: usize = 1000;

#[derive(PartialEq, Debug)]
struct Pos([i64; 2]);
#[derive(PartialEq, Debug)]
struct Vel([i64; 2]);
struct Sticky;

fn pos_vel_iter_synco(c: &mut Criterion) {
    use synco::*;

    impl Component for Pos {}

    impl Component for Vel {}

    impl Component for Sticky {}

    let mut ecs = Ecs::new()
        .with_storage::<Pos>()
        .with_storage::<Vel>()
        .with_storage::<Sticky>();

    for _ in 0..ENTITIES {
        ecs.create()
            .with(Pos([1, 2]))
            .with(Vel([3, 4]))
            .finish();
    }

    c.bench_function("pos_vel_iter_synco", |b| {
        for pos in ecs.query::<&mut Pos>().iter() {
            pos.0 = [0, 0];
        }

        b.iter(|| {
            for _ in 0..ITER {
                for (pos, vel) in ecs.query::<(&mut Pos, &Vel)>().iter() {
                    pos.0[0] += vel.0[0];
                    pos.0[1] += vel.0[1];

                    black_box((pos, vel));
                }
            }
        });
    });
}

fn pos_vel_iter_specs(c: &mut Criterion) {
    use specs::prelude::*;

    impl Component for Pos {
        type Storage = VecStorage<Self>;
    }

    impl Component for Vel {
        type Storage = VecStorage<Self>;
    }

    impl Component for Sticky {
        type Storage = VecStorage<Self>;
    }

    let mut ecs = World::new();

    ecs.register::<Pos>();
    ecs.register::<Vel>();

    for _ in 0..ENTITIES {
        ecs.create_entity()
            .with(Pos([1, 2]))
            .with(Vel([3, 4]))
            .build();
    }

    struct Clear;

    impl<'a> System<'a> for Clear {
        type SystemData = (WriteStorage<'a, Pos>,);

        fn run(&mut self, (mut pos,): Self::SystemData) {
            for (pos,) in (&mut pos,).join() {
                pos.0 = [0, 0];
            }
        }
    }

    struct Sys;

    impl<'a> System<'a> for Sys {
        type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

        fn run(&mut self, (mut pos, vel): Self::SystemData) {
            for (pos, vel) in (&mut pos, &vel).join() {
                pos.0[0] += vel.0[0];
                pos.0[1] += vel.0[1];

                black_box((pos, vel));
            }
        }
    }

    c.bench_function("pos_vel_iter_specs", |b| {
        Clear.run_now(&ecs);

        b.iter(|| {
            for _ in 0..ITER {
                Sys.run_now(&ecs);
            }
        });
    });
}

criterion_group!(compare, pos_vel_iter_synco, pos_vel_iter_specs);
criterion_main!(compare);
