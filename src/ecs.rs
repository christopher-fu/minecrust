use crate::geometry::{
    boundingbox::BoundingBox, rectangle::Rectangle, square::Square, unitcube::UnitCube,
};
use crate::render::state::RenderState;
use crate::types::*;
use glutin::VirtualKeyCode;
use specs::prelude::*;
use specs_derive::Component;
use std::ops::DerefMut;

#[derive(Debug)]
pub struct TransformComponent {
    pub transform: Transform3f,
}

impl Component for TransformComponent {
    type Storage = FlaggedStorage<Self>;
}

impl TransformComponent {
    pub fn new(transform: Transform3f) -> TransformComponent {
        TransformComponent { transform }
    }
}

#[derive(Component)]
pub enum PrimitiveGeometryComponent {
    Rectangle(Rectangle),
    Square(Square),
    UnitCube(UnitCube),
}

impl PrimitiveGeometryComponent {
    pub fn new_rect(rect: Rectangle) -> PrimitiveGeometryComponent {
        PrimitiveGeometryComponent::Rectangle(rect)
    }

    pub fn new_square(square: Square) -> PrimitiveGeometryComponent {
        PrimitiveGeometryComponent::Square(square)
    }

    pub fn new_unit_cube(unit_cube: UnitCube) -> PrimitiveGeometryComponent {
        PrimitiveGeometryComponent::UnitCube(unit_cube)
    }

    pub fn vtx_data(&self, transform: &Transform3f) -> Vec<f32> {
        match self {
            PrimitiveGeometryComponent::Rectangle(rect) => rect.vtx_data(transform),
            PrimitiveGeometryComponent::Square(square) => square.vtx_data(transform),
            PrimitiveGeometryComponent::UnitCube(cube) => cube.vtx_data(transform),
        }
    }
}

pub struct BoundingBoxComponent {
    pub bbox: BoundingBox,
}

impl Component for BoundingBoxComponent {
    type Storage = FlaggedStorage<Self>;
}

pub struct BoundingBoxComponentSystem {
    inserted_id: ReaderId<InsertedFlag>,
    modified_id: ReaderId<ModifiedFlag>,
    inserted: BitSet,
    modified: BitSet,
}

impl BoundingBoxComponentSystem {
    pub fn new(
        inserted_id: ReaderId<InsertedFlag>,
        modified_id: ReaderId<ModifiedFlag>,
        inserted: BitSet,
        modified: BitSet,
    ) -> BoundingBoxComponentSystem {
        BoundingBoxComponentSystem {
            inserted_id,
            modified_id,
            inserted,
            modified,
        }
    }
}

impl<'a> System<'a> for BoundingBoxComponentSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, TransformComponent>,
        WriteStorage<'a, BoundingBoxComponent>,
    );

    fn run(&mut self, (entities, transforms, mut bounding_boxes): Self::SystemData) {
        self.inserted.clear();
        self.modified.clear();

        transforms.populate_inserted(&mut self.inserted_id, &mut self.inserted);
        transforms.populate_modified(&mut self.modified_id, &mut self.modified);

        for (entity, transform, _) in (&entities, &transforms, &self.inserted).join() {
            println!("{:?}", transform);
        }
    }
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct SelectedComponent;

pub struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, TransformComponent>,
        ReadStorage<'a, PrimitiveGeometryComponent>,
        WriteExpect<'a, RenderState>,
    );

    fn run(&mut self, (transform, geometry, mut render_state): Self::SystemData) {
        let render_state = render_state.deref_mut();
        let RenderState {
            ref mut vao,
            ref mut selection_vao,
            ref mut crosshair_vao,
            ref frame_time_delta,
            ref pressed_keys,
            ref mouse_delta,
            ref mut camera,
            ref mut shader_program,
            ref mut crosshair_shader_program,
            ref cobblestone_texture,
            ref selection_texture,
            ref crosshair_texture,
            ref projection,
        } = render_state;
        let d_yaw = mouse_delta.0 as f32 / 500.0;
        let d_pitch = mouse_delta.1 as f32 / 500.0;
        let delta_time = frame_time_delta.as_nanos() as f32 / 1_000_000_000.0f32;
        // Negate deltas because positive x is decreasing yaw, and positive y (origin for mouse
        // events is at upper left) is decreasing pitch.
        camera.rotate((-d_yaw, -d_pitch));
        let camera_speed = 3.0 * delta_time;
        for keycode in pressed_keys {
            match keycode {
                VirtualKeyCode::W => camera.pos += camera_speed * camera.direction().unwrap(),
                VirtualKeyCode::S => camera.pos -= camera_speed * camera.direction().unwrap(),
                VirtualKeyCode::A => {
                    let delta = camera_speed * (Vector3f::cross(&camera.direction(), &camera.up()));
                    camera.pos -= delta;
                }
                VirtualKeyCode::D => {
                    let delta = camera_speed * (Vector3f::cross(&camera.direction(), &camera.up()));
                    camera.pos += delta;
                }
                _ => (),
            }
        }

        let mut vtx_buf = vec![];
        for (transform, geometry) in (&transform, &geometry).join() {
            vtx_buf.extend(geometry.vtx_data(&transform.transform));
        }

        vao.buffer.set_buf(vtx_buf);
        vao.buffer.gl_bind(GlBufferUsage::DynamicDraw);

        unsafe {
            gl::DepthFunc(gl::LESS);
        }

        shader_program.set_used();
        shader_program.set_mat4f("view", &camera.to_matrix());
        shader_program.set_mat4f("projection", projection);

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, *cobblestone_texture);
        }
        vao.gl_draw();

        unsafe {
            gl::DepthFunc(gl::LEQUAL);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, *selection_texture);
        }
        selection_vao.gl_draw();

        crosshair_shader_program.set_used();
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, *crosshair_texture);
        }
        crosshair_vao.gl_draw();
    }
}
