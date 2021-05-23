#![feature(
    generic_associated_types,
    maybe_uninit_ref,
    maybe_uninit_extra,
    associated_type_defaults,
    min_type_alias_impl_trait,
    unboxed_closures,
    fn_traits,
)]

pub mod component;
pub mod entity;
pub mod query;
pub mod resource;
pub mod row;
pub mod storage;
pub mod system;

pub use self::{
    component::Component,
    entity::{BitMask, Entity, Entities},
    query::{Query, Pattern, Not},
    resource::Resource,
    row::{Read, Write},
    storage::{Storage, VecStorage},
    system::{Input, IntoSystem, System},
};

use core::any::{Any, type_name};
use self::{
    component::ComponentId,
    row::Row,
};

use anymap::AnyMap;

pub struct Ecs {
    entities: Row<Entities>,
    component_id: u64,
    resources: AnyMap,
}

impl Default for Ecs {
    fn default() -> Self { Self::new() }
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            entities: Row::default(),
            component_id: 0,
            resources: AnyMap::new(),
        }
    }

    pub fn with_resource<R: Resource>(mut self, res: R) -> Self {
        self.insert_resource_inner(res);
        self
    }

    pub fn with_component<C: Component>(mut self) -> Self {
        if self.component_id >= u64::BITS as u64 {
            panic!("Too many components!");
        } else {
            self.insert_resource_inner(C::Storage::default());
            self.resources.insert(ComponentId::<C>::new(self.component_id));
            self.component_id += 1;
            self
        }
    }

    pub(crate) fn maybe_resource_inner<R: Resource>(&self) -> Option<&Row<R>> {
        self.resources.get()
    }

    pub(crate) fn maybe_resource_inner_mut<R: Resource>(&mut self) -> Option<&mut Row<R>> {
        self.resources.get_mut()
    }

    pub(crate) fn resource_inner<R: Resource>(&self) -> &Row<R> {
        self
            .maybe_resource_inner()
            .unwrap_or_else(|| panic!("Resource `{:?}` is not present in the ECS", type_name::<R>()))
    }

    pub(crate) fn resource_inner_mut<R: Resource>(&mut self) -> &mut Row<R> {
        self
            .maybe_resource_inner_mut()
            .unwrap_or_else(|| panic!("Resource `{:?}` is not present in the ECS", type_name::<R>()))
    }

    pub(crate) fn insert_resource_inner<R: Resource>(&mut self, res: R) -> Option<R> {
        self.resources.insert(Row::new(res)).map(Row::into_inner)
    }

    pub(crate) fn storage_id<C: Component>(&self) -> u64 {
        self.resources
            .get::<ComponentId<C>>()
            .map(|c_id| c_id.id)
            .unwrap_or_else(|| panic!("Storage for component `{:?}` is not present in the ECS", type_name::<C>()))
    }

    pub fn read_resource<R: Resource>(&self) -> Read<'_, R> { self.resource_inner().read() }
    pub fn write_resource<R: Resource>(&self) -> Write<'_, R> { self.resource_inner().write() }
    pub fn mut_resource<R: Resource>(&mut self) -> &mut R { self.resource_inner_mut().get_mut() }

    pub fn read_storage<C: Component>(&self) -> ReadStorage<'_, C> {
        ReadStorage {
            entities: self.entities.read(),
            storage: self.read_resource(),
        }
    }

    pub fn write_storage<C: Component>(&self) -> WriteStorage<'_, C> {
        WriteStorage {
            entities: self.entities.write(),
            storage: self.write_resource(),
        }
    }

    pub fn query<P: Pattern>(&self) -> Query<'_, P> {
        P::fetch(self)
    }

    pub fn run<'a, S: IntoSystem<'a, T>, T>(&'a self, sys: S) {
        let sys = sys.into_system();
        let inputs = <S::System as System<'a>>::Input::fetch(self);
        sys.run(inputs);
    }

    pub fn insert_comp<C: Component>(&mut self, entity: Entity, comp: C) -> Option<C> {
        let comp_id = self.storage_id::<C>();

        let entry = self.entities
            .get_mut()
            .entry_mut(entity)
            .expect("Created entity does not exist!");

        let old = if entry.comp_mask.bit_is_set(comp_id) {
            Some(unsafe { self
                .mut_resource::<C::Storage>()
                .remove_unchecked(entity) })
        } else {
            entry.comp_mask.set_bit(comp_id);
            None
        };

        unsafe { self
            .mut_resource::<C::Storage>()
            .insert_unchecked(entity, comp) };

        old
    }

    pub fn remove_comp<C: Component>(&mut self, entity: Entity, comp: C) -> Option<C> {
        let comp_id = self.storage_id::<C>();

        let entry = self.entities
            .get_mut()
            .entry_mut(entity)
            .expect("Created entity does not exist!");

        let old = if entry.comp_mask.bit_is_set(comp_id) {
            Some(unsafe { self
                .mut_resource::<C::Storage>()
                .remove_unchecked(entity) })
        } else {
            entry.comp_mask.unset_bit(comp_id);
            None
        };

        old
    }

    pub fn create(&mut self) -> EntityBuilder<'_> {
        let entity = self.entities.get_mut().create();
        EntityBuilder { entity, ecs: self }
    }
}

pub struct ReadStorage<'a, C: Component> {
    entities: Read<'a, Entities>,
    storage: Read<'a, C::Storage>,
}

pub struct WriteStorage<'a, C: Component> {
    entities: Write<'a, Entities>,
    storage: Write<'a, C::Storage>,
}

pub struct EntityBuilder<'a> {
    entity: Entity,
    ecs: &'a mut Ecs,
}

impl<'a> EntityBuilder<'a> {
    pub fn with<C: Component>(self, comp: C) -> Self {
        self.ecs.insert_comp(self.entity, comp);
        self
    }

    pub fn finish(self) -> Entity { self.entity }
}
