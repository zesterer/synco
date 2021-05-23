use std::{
    cell::{RefCell, Ref, RefMut},
    ops::{Deref, DerefMut},
};

#[derive(Default)]
pub struct Row<T>(RefCell<T>);

/// Read Or Write
impl<T> Row<T> {
    pub fn new(x: T) -> Self { Self(RefCell::new(x)) }
    pub fn read(&self) -> Read<'_, T> { Read(self.0.borrow()) }
    pub fn write(&self) -> Write<'_, T> { Write(self.0.borrow_mut()) }
    pub fn get_mut(&mut self) -> &mut T { self.0.get_mut() }
    pub fn into_inner(self) -> T { self.0.into_inner() }
}

pub struct Read<'a, T>(Ref<'a, T>);

impl<'a, T> Clone for Read<'a, T> {
    fn clone(&self) -> Self { Self(Ref::clone(&self.0)) }
}

impl<'a, T> Deref for Read<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { self.0.deref() }
}

pub struct Write<'a, T>(RefMut<'a, T>);

impl<'a, T> Deref for Write<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { self.0.deref() }
}

impl<'a, T> DerefMut for Write<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target { self.0.deref_mut() }
}
