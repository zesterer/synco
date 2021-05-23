use super::*;

use std::marker::PhantomData;

pub trait Component: Sized + 'static {
    type Storage: Storage<Self> = VecStorage<Self>;
}

pub(crate) struct ComponentId<C: Any> {
    pub id: u64,
    phantom: PhantomData<C>,
}

impl<C: Any> ComponentId<C> {
    pub fn new(id: u64) -> Self {
        Self { id, phantom: PhantomData }
    }
}
