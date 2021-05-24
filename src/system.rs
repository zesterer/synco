use super::*;

use std::marker::PhantomData;

pub trait System<'a> {
    type Input: Input<'a>;
    type Output = ();

    fn run(self, inputs: Self::Input) -> Self::Output;
}

pub trait Input<'a> {
    fn fetch(ecs: &'a Ecs) -> Self;
}

impl<'a, R: Resource> Input<'a> for Read<'a, R> {
    fn fetch(ecs: &'a Ecs) -> Self { ecs.read_resource() }
}

impl<'a, R: Resource> Input<'a> for Write<'a, R> {
    fn fetch(ecs: &'a Ecs) -> Self { ecs.write_resource() }
}

impl<'a, P: Pattern> Input<'a> for Query<'a, P> {
    fn fetch(ecs: &'a Ecs) -> Self { P::fetch(ecs) }
}

macro_rules! impl_for_tuple {
    ($($x:ident),*) => {
        impl<'a, $($x: Input<'a>),*> Input<'a> for ($($x),*,) {
            fn fetch(ecs: &'a Ecs) -> Self {
                ($($x::fetch(ecs)),*,)
            }
        }
    };
}

impl_for_tuple!(A);
impl_for_tuple!(A, B);
impl_for_tuple!(A, B, C);
impl_for_tuple!(A, B, C, D);
impl_for_tuple!(A, B, C, D, E);
impl_for_tuple!(A, B, C, D, E, F);
impl_for_tuple!(A, B, C, D, E, F, G);
impl_for_tuple!(A, B, C, D, E, F, G, H);
impl_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W, X);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W, X, Y);
impl_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W, X, Y, Z);

pub struct FnSystem<F, P, O>(F, PhantomData<(P, O)>);

impl<'a, F: FnOnce(P) -> O, P: Input<'a>, O> System<'a> for FnSystem<F, P, O>
{
    type Input = P;
    type Output = O;
    fn run(self, inputs: Self::Input) -> Self::Output { self.0(inputs) }
}

pub trait IntoSystem<'a, P> {
    type System: System<'a>;

    fn into_system(self) -> Self::System;
}

impl<'a, S: System<'a>> IntoSystem<'a, ()> for S {
    type System = S;

    fn into_system(self) -> Self::System { self }
}

// impl<'a, F: FnOnce(A, B), A: Input<'a>, B: Input<'a>> IntoSystem<'a, (A, B)> for F {
//     type System = FnSystem<impl FnOnce((A, B)), (A, B)>;
//     fn into_system(self) -> Self::System { FnSystem(move |(a, b)| self(a, b), PhantomData) }
// }

impl<'a, Fn: FnOnce<Args, Output = O>, Args: Input<'a>, O> IntoSystem<'a, Args> for Fn {
    type System = FnSystem<impl FnOnce(Args) -> O, Args, O>;
    fn into_system(self) -> Self::System { FnSystem(move |args| self.call_once(args), PhantomData) }
}

// macro_rules! impl_for_fn {
//     ($($x:ident),*) => {
//         impl<'a, Fn: FnOnce($($x),*), $($x: Input<'a>),*> IntoSystem<'a, ($($x),*)> for Fn {
//             type System = FnSystem<impl FnOnce(($($x),*)), ($($x),*)>;
//             fn into_system(self) -> Self::System { FnSystem(move |($($x),*)| self($($x),*), PhantomData) }
//         }
//     };
// }

// impl_for_fn!(A);
// impl_for_fn!(A, B);
// impl_for_fn!(A, B, C);
// impl_for_fn!(A, B, C, D);
// impl_for_fn!(A, B, C, D, E);
// impl_for_fn!(A, B, C, D, E, F);
// impl_for_fn!(A, B, C, D, E, F, G);
// impl_for_fn!(A, B, C, D, E, F, G, H);
// impl_for_fn!(A, B, C, D, E, F, G, H, I);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W, X);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W, X, Y);
// impl_for_fn!(A, B, C, D, E, F, G, H, I, J, K, L, M, O, P, Q, R, S, T, U, V, W, X, Y, Z);
