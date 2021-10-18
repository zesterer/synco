use super::*;

pub use self::{
    null::NullStorage,
    vec::VecStorage,
};

use std::ops::{Deref, DerefMut};

pub trait Storage<T: Component>: Resource + Default {
    type Ref<'a>: Deref<Target = T> where T: 'a;
    type RefMut<'a>: DerefMut<Target = T> where T: 'a;

    /// Safety: The entity must have an already-inserted component in this storage.
    unsafe fn get_unchecked(&self, entity: EntityId) -> Self::Ref<'_>;
    /// Safety: The entity must have an already-inserted component in this storage.
    unsafe fn get_unchecked_mut(&mut self, entity: EntityId) -> Self::RefMut<'_>;
    /// Safety: The entity must not have an already-inserted component in this storage.
    unsafe fn insert_unchecked(&mut self, entity: EntityId, item: T);
    /// Safety: The entity must have an already-inserted component in this storage.
    unsafe fn remove_unchecked(&mut self, entity: EntityId) -> T;
}

pub mod null {
    use super::*;
    use std::{
        marker::PhantomData,
        mem::MaybeUninit,
    };

    pub struct NullStorage<T>(PhantomData<T>);

    impl<T> Default for NullStorage<T> {
        fn default() -> Self { Self(PhantomData) }
    }

    impl<T: Component> Storage<T> for NullStorage<T> {
        type Ref<'a> where T: 'a = &'a T;
        type RefMut<'a> where T: 'a = &'a mut T;

        unsafe fn get_unchecked(&self, entity: EntityId) -> Self::Ref<'_> {
            assert_eq!(core::mem::size_of::<T>(), 0);
            &*(&() as *const _ as *const _)
        }

        unsafe fn get_unchecked_mut(&mut self, entity: EntityId) -> Self::RefMut<'_> {
            assert_eq!(core::mem::size_of::<T>(), 0);
            &mut *(&mut () as *mut _ as *mut _)
        }

        unsafe fn insert_unchecked(&mut self, entity: EntityId, item: T) {}

        unsafe fn remove_unchecked(&mut self, entity: EntityId) -> T {
            MaybeUninit::uninit().assume_init_read()
        }
    }
}

pub mod vec {
    use super::*;
    use std::mem::MaybeUninit;

    pub struct VecStorage<T> {
        items: Vec<MaybeUninit<T>>,
    }

    impl<T> Default for VecStorage<T> {
        fn default() -> Self { Self { items: Vec::new() } }
    }

    impl<T: Component> Storage<T> for VecStorage<T> {
        type Ref<'a> where T: 'a = &'a T;
        type RefMut<'a> where T: 'a = &'a mut T;

        unsafe fn get_unchecked(&self, entity: EntityId) -> Self::Ref<'_> {
            self.items.get_unchecked(entity.idx()).assume_init_ref()
        }

        unsafe fn get_unchecked_mut(&mut self, entity: EntityId) -> Self::RefMut<'_> {
            self.items.get_unchecked_mut(entity.idx()).assume_init_mut()
        }

        unsafe fn insert_unchecked(&mut self, entity: EntityId, item: T) {
            let idx = entity.idx();
            self.items.resize_with(self.items.len().max(idx + 1), || MaybeUninit::uninit());
            self.items.get_unchecked_mut(idx).write(item);
        }

        unsafe fn remove_unchecked(&mut self, entity: EntityId) -> T {
            self.items.get_unchecked_mut(entity.idx()).assume_init_read()
        }
    }
}
