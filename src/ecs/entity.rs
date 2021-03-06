use crate::{
    block::BlockType,
    ecs::{AabbComponent, BlockComponent, PrimitiveGeometryComponent, TransformComponent},
    geometry::{Aabb, UnitCube},
    types::prelude::*,
};
use specs::{ReadStorage, WriteStorage};
use std::ops::{Deref, DerefMut};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Entity {
    pub entity: specs::Entity,
}

impl From<specs::Entity> for Entity {
    fn from(entity: specs::Entity) -> Entity {
        Entity { entity }
    }
}

impl Entity {
    pub fn new_empty(entities: &specs::Entities) -> Entity {
        Entity {
            entity: entities.create(),
        }
    }

    pub fn new_clone(
        entity: Entity,
        entities: &specs::Entities,
        transform_storage: &mut WriteStorage<TransformComponent>,
        aabb_storage: &mut WriteStorage<AabbComponent>,
    ) -> Entity {
        let entity = Entity::new_with_transform(
            entity.component(transform_storage).0,
            entities,
            transform_storage,
        );
        entity.set_aabb(aabb_storage, AabbComponent(*entity.aabb(aabb_storage)));
        entity
    }

    pub fn new_with_transform(
        transform: Transform3f,
        entities: &specs::Entities,
        storage: &mut WriteStorage<TransformComponent>,
    ) -> Entity {
        Entity::new_empty(entities).with_component(TransformComponent(transform), storage)
    }

    pub fn new_with_transform_w(transform: Transform3f, world: &specs::World) -> Entity {
        Entity::new_with_transform(transform, &world.entities(), &mut world.write_storage())
    }

    pub fn new_unitcube(
        transform: Transform3f,
        entities: &specs::Entities,
        transform_storage: &mut WriteStorage<TransformComponent>,
        aabb_storage: &mut WriteStorage<AabbComponent>,
    ) -> Entity {
        let unitcube = UnitCube::new(1.0);
        let aabb = Aabb::new(Point3f::origin(), Vector3f::new(0.5, 0.5, 0.5)).transform(&transform);
        let entity = entities
            .build_entity()
            .with(TransformComponent(transform), transform_storage)
            .with(AabbComponent(aabb), aabb_storage)
            .build();
        Entity { entity }
    }

    pub fn new_unitcube_w(transform: Transform3f, world: &specs::World) -> Entity {
        Entity::new_unitcube(
            transform,
            &world.entities(),
            &mut world.write_storage(),
            &mut world.write_storage(),
        )
    }

    pub fn new_block(
        transform: Transform3f,
        block_type: BlockType,
        entities: &specs::Entities,
        transform_storage: &mut WriteStorage<TransformComponent>,
        aabb_storage: &mut WriteStorage<AabbComponent>,
        block_storage: &mut WriteStorage<BlockComponent>,
    ) -> Entity {
        Entity::new_unitcube(
            transform,
            entities,
            transform_storage,
            aabb_storage,
        )
        .with_component(BlockComponent(block_type), block_storage)
    }

    pub fn new_block_w(
        transform: Transform3f,
        block_type: BlockType,
        world: &specs::World,
    ) -> Entity {
        Entity::new_block(
            transform,
            block_type,
            &world.entities(),
            &mut world.write_storage(),
            &mut world.write_storage(),
            &mut world.write_storage(),
        )
    }

    pub fn with_component<T, D>(self, component: T, storage: &mut specs::Storage<T, D>) -> Self
    where
        T: specs::Component,
        D: DerefMut<Target = specs::storage::MaskedStorage<T>>,
    {
        storage
            .insert(self.entity, component)
            .expect("Entity::with_component() failed");
        self
    }

    pub fn component<'a, T, D>(self, storage: &'a specs::Storage<T, D>) -> &'a T
    where
        T: specs::Component,
        D: Deref<Target = specs::storage::MaskedStorage<T>>,
    {
        storage.get(self.entity).unwrap()
    }

    pub fn transform<'a, D>(
        self,
        storage: &'a specs::Storage<TransformComponent, D>,
    ) -> &'a Transform3f
    where
        D: Deref<Target = specs::storage::MaskedStorage<TransformComponent>>,
    {
        &self.component(storage).0
    }

    pub fn set_transform(
        self,
        storage: &mut WriteStorage<TransformComponent>,
        transform: Transform3f,
    ) {
        if let Some(tc) = storage.get_mut(self.entity) {
            tc.0 = transform;
        } else {
            storage
                .insert(self.entity, TransformComponent(transform))
                .expect("insert() failed");
        }
    }

    /// Panics if the entity doesn't have a `PrimitiveGeometryComponent`.
    pub fn geometry<'a, D>(
        self,
        storage: &'a specs::Storage<PrimitiveGeometryComponent, D>,
    ) -> &'a PrimitiveGeometryComponent
    where
        D: Deref<Target = specs::storage::MaskedStorage<PrimitiveGeometryComponent>>,
    {
        self.component(storage)
    }

    pub fn set_geometry(
        self,
        storage: &mut WriteStorage<PrimitiveGeometryComponent>,
        geometry: PrimitiveGeometryComponent,
    ) {
        if let Some(pgc) = storage.get_mut(self.entity) {
            *pgc = geometry
        } else {
            storage
                .insert(self.entity, geometry)
                .expect("insert() failed");
        }
    }

    pub fn aabb<'a, D>(self, storage: &'a specs::Storage<AabbComponent, D>) -> &'a Aabb
    where
        D: Deref<Target = specs::storage::MaskedStorage<AabbComponent>>,
    {
        &self.component(storage).0
    }

    pub fn set_aabb(self, storage: &mut WriteStorage<AabbComponent>, aabb: AabbComponent) {
        if let Some(bb) = storage.get_mut(self.entity) {
            *bb = aabb
        } else {
            storage.insert(self.entity, aabb).expect("insert() failed");
        }
    }

    pub fn position(self, storage: &ReadStorage<TransformComponent>) -> Point3f {
        self.transform(storage).translation()
    }

    pub fn position_aabb(self, storage: &ReadStorage<AabbComponent>) -> Point3f {
        self.aabb(storage).center
    }
}
