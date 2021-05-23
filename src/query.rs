use super::{*, entity::EntityIter};

use std::{
    cell::{UnsafeCell, Ref},
    marker::PhantomData,
};

pub struct Query<'a, P: Pattern> {
    entities: Read<'a, Entities>,
    filter: (BitMask, BitMask),
    state: UnsafeCell<P::State<'a>>,
}

impl<'a, P: Pattern> Clone for Query<'a, P>
    where P::State<'a>: Clone
{
    fn clone(&self) -> Self {
        Self {
            entities: self.entities.clone(),
            filter: self.filter.clone(),
            state: UnsafeCell::new(unsafe { &*self.state.get() }.clone()),
        }
    }
}

impl<'a, P: Pattern> Query<'a, P> {
    pub fn iter(&mut self) -> QueryIter<'_, 'a, P> {
        QueryIter {
            state: &mut self.state,
            entities: self.entities.iter_filter(&self.filter),
        }
    }

    pub fn get(&mut self, entity: Entity) -> Option<P::Output<'_>> {
        if self.entities.comp_mask(entity)?.matches(&self.filter) {
            // Safety: filter has been checked, access must be valid
            Some(unsafe { P::get_unchecked(self.state.get_mut(), entity) })
        } else {
            None
        }
    }
}

pub struct QueryIter<'a, 'b, P: Pattern> {
    state: &'a mut UnsafeCell<P::State<'b>>,
    entities: EntityIter<'a>,
}

impl<'a, 'b: 'a, P: Pattern> Iterator for QueryIter<'a, 'b, P> {
    type Item = P::Output<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entity = self.entities.next()?;
        Some(unsafe { P::get_unchecked(&mut *self.state.get(), entity) })
    }
}

pub trait Pattern: Sized {
    type State<'a>: 'a;
    type Output<'a>: 'a;

    fn comp_filter(ecs: &Ecs) -> (BitMask, BitMask);

    fn fetch_inner<'a>(ecs: &'a Ecs) -> Self::State<'a>;

    fn fetch<'a>(ecs: &'a Ecs) -> Query<'a, Self> {
        Query {
            entities: ecs.entities.read(),
            filter: Self::comp_filter(ecs),
            state: UnsafeCell::new(Self::fetch_inner(ecs)),
        }
    }

    unsafe fn get_unchecked<'a, 'b: 'a>(state: &'a mut Self::State<'b>, entity: Entity) -> Self::Output<'a>;
}

impl<'c, C: Component> Pattern for &'c C
    where for<'a> C::Storage: Storage<C, Ref<'a> = &'a C>
{
    type State<'a> = Read<'a, C::Storage>;
    type Output<'a> = &'a C;

    fn comp_filter(ecs: &Ecs) -> (BitMask, BitMask) {
        let mask = BitMask::with(ecs.storage_id::<C>() as u64);
        (mask.clone(), mask)
    }

    fn fetch_inner<'a>(ecs: &'a Ecs) -> Self::State<'a> { ecs.read_resource() }

    unsafe fn get_unchecked<'a, 'b: 'a>(state: &'a mut Self::State<'b>, entity: Entity) -> Self::Output<'a> {
        state.get_unchecked(entity)
    }
}

impl<'c, C: Component> Pattern for &'c mut C
    where for<'a> C::Storage: Storage<C, RefMut<'a> = &'a mut C>
{
    type State<'a> = Write<'a, C::Storage>;
    type Output<'a> = &'a mut C;

    fn comp_filter(ecs: &Ecs) -> (BitMask, BitMask) {
        let mask = BitMask::with(ecs.storage_id::<C>() as u64);
        (mask.clone(), mask)
    }

    fn fetch_inner<'a>(ecs: &'a Ecs) -> Self::State<'a> { ecs.write_resource() }

    unsafe fn get_unchecked<'a, 'b: 'a>(state: &'a mut Self::State<'b>, entity: Entity) -> Self::Output<'a> {
        state.get_unchecked_mut(entity)
    }
}

impl Pattern for Entity {
    type State<'a> = ();
    type Output<'a> = Entity;

    fn comp_filter(ecs: &Ecs) -> (BitMask, BitMask) {
        let mask = BitMask::zero();
        (mask.clone(), mask)
    }

    fn fetch_inner<'a>(ecs: &'a Ecs) -> Self::State<'a> { () }

    unsafe fn get_unchecked<'a, 'b: 'a>(state: &'a mut Self::State<'b>, entity: Entity) -> Self::Output<'a> {
        entity
    }
}

pub struct Not<C: Component>(PhantomData<C>);

impl<C: Component> Pattern for Not<C> {
    type State<'a> = ();
    type Output<'a> = ();

    fn comp_filter(ecs: &Ecs) -> (BitMask, BitMask) {
        (BitMask::zero(), BitMask::with(ecs.storage_id::<C>() as u64))
    }

    fn fetch_inner<'a>(ecs: &'a Ecs) -> Self::State<'a> {}

    unsafe fn get_unchecked<'a, 'b: 'a>(state: &'a mut Self::State<'b>, entity: Entity) -> Self::Output<'a> {}
}

macro_rules! impl_for_tuple {
    ($($x:ident),*) => {
        impl<$($x: Pattern),*> Pattern for ($($x),*,) {
            type State<'a> = ($($x::State<'a>),*,);
            type Output<'a> = ($($x::Output<'a>),*,);

            fn comp_filter(ecs: &Ecs) -> (BitMask, BitMask) {
                let filter = (BitMask::zero(), BitMask::zero());
                $(let filter = {
                    let new_filter = $x::comp_filter(ecs);
                    BitMask::combine_filters(filter, new_filter)
                        .unwrap_or_else(|| panic!("Incompatible pattern: {}", type_name::<$x>()))
                };);*
                filter
            }

            fn fetch_inner<'a>(ecs: &'a Ecs) -> Self::State<'a> {
                ($($x::fetch_inner(ecs)),*,)
            }

            #[allow(non_snake_case)]
            unsafe fn get_unchecked<'a, 'b: 'a>(($($x),*,): &'a mut Self::State<'b>, entity: Entity) -> Self::Output<'a> {
                ($($x::get_unchecked($x, entity)),*,)
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
