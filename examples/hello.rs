use synco::{Ecs, Component, Entity, System, Read, Query, Not};

pub struct TimeOfDay(f64);

pub struct DeltaTime(f64);

#[derive(Debug)]
pub struct Pos(f64);
impl Component for Pos {}

#[derive(Debug)]
pub struct Vel(f64);
impl Component for Vel {}

struct Phys;

impl<'a> System<'a> for Phys {
    type Input = (Read<'a, DeltaTime>, Query<'a, (&'a mut Pos, &'a Vel)>);

    fn run(self, (dt, mut q): Self::Input) {
        for (pos, vel) in q.iter() {
            pos.0 += vel.0 * dt.0;
        }
    }
}

fn physics(dt: Read<DeltaTime>, mut q: Query<(&mut Pos, &Vel)>) {
    for (pos, vel) in q.iter() {
        pos.0 += vel.0 * dt.0;
    }
}

fn main() {
    let mut ecs = Ecs::new()
        .with_resource(TimeOfDay(0.0))
        .with_resource(DeltaTime(1.0))
        .with_storage::<Pos>()
        .with_storage::<Vel>();

    ecs.create()
        .with(Pos(42.0))
        .with(Vel(43.0))
        .finish();

    ecs.create()
        .with(Pos(16.0))
        .with(Vel(17.0))
        .finish();

    ecs.create()
        .with(Pos(89.0))
        .with(Vel(90.0))
        .finish();

    ecs.run(Phys);

    for (entity, pos, vel) in ecs.query::<(Entity, &mut Pos, &Vel)>().iter() {
        println!("Entity {:?} has {:?}, {:?}", entity, pos, vel);
    }

    ecs.run(physics);

    let mut q = ecs.query::<&Pos>();

    for pos0 in q.clone().iter() {
        for pos1 in q.iter() {
            println!("{:?}, {:?}", pos0, pos1);
        }
    }

    println!("Hello, world!");
}
