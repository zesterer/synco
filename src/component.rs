use super::*;

use std::marker::PhantomData;

pub trait Component: Sized + Any {
    type Storage: Storage<Self> = VecStorage<Self>;
}

pub(crate) struct ComponentId<C: Component> {
    pub id: u64,
    phantom: PhantomData<C>,
}

impl<C: Component> ComponentId<C> {
    pub fn new(id: u64) -> Self {
        Self { id, phantom: PhantomData }
    }
}
