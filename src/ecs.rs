use crate::camera::Camera;
use crate::geometry::{
    boundingbox::BoundingBox, rectangle::Rectangle, square::Square, unitcube::UnitCube,
    PrimitiveGeometry,
};
use crate::gl::{shader::Program, VertexArrayObject};
use crate::types::*;
use crate::utils::{f32, pt3f, quat4f, vec3f, NSEC_PER_SEC};
use glutin;
use glutin::VirtualKeyCode;
use specs::prelude::*;
use specs::Entity;
use specs_derive::Component;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::ops::DerefMut;
use std::time::Duration;

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

    pub fn vtx_data(&mut self, transform: &Transform3f) -> Vec<f32> {
        match self {
            PrimitiveGeometryComponent::Rectangle(ref mut rect) => rect.vtx_data(transform),
            PrimitiveGeometryComponent::Square(ref mut square) => square.vtx_data(transform),
            PrimitiveGeometryComponent::UnitCube(ref mut cube) => cube.vtx_data(transform),
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

pub struct RenderState {
    pub vao: VertexArrayObject,
    pub selection_vao: VertexArrayObject,
    pub crosshair_vao: VertexArrayObject,
    pub elapsed_time: Duration,
    pub frame_time_delta: Duration,
    pub pressed_keys: HashSet<glutin::VirtualKeyCode>,
    pub mouse_delta: (f64, f64),
    pub camera: Camera,
    pub shader_program: Program,
    pub crosshair_shader_program: Program,
    pub cobblestone_texture: u32,
    pub selection_texture: u32,
    pub crosshair_texture: u32,
    pub projection: Matrix4f,
    pub selected_cube: Option<Entity>,
    pub camera_animation: Option<CameraAnimation>,
}

#[derive(Component, Default)]
#[storage(NullStorage)]
pub struct SelectedComponent;

pub struct RenderSystem;

#[derive(Debug, Clone, Copy)]
pub struct CameraAnimation {
    pub start_pos: Point3f,
    pub start_yaw_q: UnitQuaternionf,
    pub start_pitch_q: UnitQuaternionf,
    pub end_pos: Point3f,
    pub end_yaw_q: UnitQuaternionf,
    pub end_pitch_q: UnitQuaternionf,
    pub start_time: f32,
    pub duration: f32,
}

impl CameraAnimation {
    pub fn new(
        camera: &Camera,
        end_position: Point3f,
        end_direction: &Vector3f,
        start_time: f32,
        duration: f32,
    ) -> CameraAnimation {
        let (mut yaw_diff, pitch_diff) = vec3f::yaw_pitch_diff(&camera.direction(), &end_direction);
        if yaw_diff > PI {
            yaw_diff -= 2.0 * PI;
        } else if yaw_diff < -PI {
            yaw_diff += 2.0 * PI;
        }
        let rot_yaw_q = UnitQuaternionf::from_euler_angles(0.0, yaw_diff, 0.0);
        let rot_pitch_q = UnitQuaternionf::from_euler_angles(pitch_diff, 0.0, 0.0);
        CameraAnimation {
            start_pos: camera.pos,
            start_yaw_q: camera.yaw_q,
            start_pitch_q: camera.pitch_q,
            end_pos: end_position,
            end_yaw_q: rot_yaw_q * camera.yaw_q,
            end_pitch_q: rot_pitch_q * camera.pitch_q,
            start_time,
            duration,
        }
    }

    /// Returns the point, yaw quaternion, and pitch quaternion of the animation at time `t`.
    pub fn at(&self, time: f32) -> (Point3f, UnitQuaternionf, UnitQuaternionf) {
        let t = (time - self.start_time) / self.duration;
        (
            pt3f::clerp(&self.start_pos, &self.end_pos, t),
            quat4f::clerp(&self.start_yaw_q, &self.end_yaw_q, t),
            quat4f::clerp(&self.start_pitch_q, &self.end_pitch_q, t),
        )
    }

    pub fn end_time(&self) -> f32 {
        self.start_time + self.duration
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, TransformComponent>,
        WriteStorage<'a, PrimitiveGeometryComponent>,
        WriteExpect<'a, RenderState>,
    );

    fn run(&mut self, (transform_storage, mut geometry, mut render_state): Self::SystemData) {
        let render_state = render_state.deref_mut();
        let RenderState {
            ref mut vao,
            ref mut selection_vao,
            ref mut crosshair_vao,
            ref elapsed_time,
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
            ref selected_cube,
            ref mut camera_animation,
        } = render_state;
        let d_yaw = mouse_delta.0 as f32 / 500.0;
        let d_pitch = mouse_delta.1 as f32 / 500.0;
        let frame_time_delta_f = frame_time_delta.as_nanos() as f32 / 1_000_000_000.0f32;
        let elapsed_time_f = elapsed_time.as_nanos() as f32 / NSEC_PER_SEC as f32;
        let mut camera_animation_finished = false;
        if let Some(camera_animation) = camera_animation {
            // Check if animation has expired
            if elapsed_time_f >= camera_animation.end_time() {
                camera.pos = camera_animation.end_pos;
                camera.pitch_q = camera_animation.end_pitch_q;
                camera.yaw_q = camera_animation.end_yaw_q;
                camera_animation_finished = true;
            } else {
                let (pos, yaw_q, pitch_q) = camera_animation.at(elapsed_time_f);
                camera.pos = pos;
                camera.pitch_q = pitch_q;
                camera.yaw_q = yaw_q;
                camera_animation_finished = false;
            }
        } else {
            camera.rotate((-d_yaw, d_pitch));
        }
        if camera_animation_finished {
            *camera_animation = None;
        }
        let camera_speed = 3.0 * frame_time_delta_f;
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
        for (transform, geometry) in (&transform_storage, &mut geometry).join() {
            vtx_buf.extend(geometry.vtx_data(&transform.transform));
        }

        vao.buffer_mut().set_buf(vtx_buf);
        vao.buffer_mut().gl_bind(GlBufferUsage::DynamicDraw);

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

        match selected_cube {
            Some(cube) => {
                let transform = transform_storage.get(*cube).unwrap();
                if let Some(PrimitiveGeometryComponent::UnitCube(ref mut cube_geom)) =
                    geometry.get_mut(*cube)
                {
                    let vtx_data = cube_geom.vtx_data(&transform.transform);
                    selection_vao.buffer_mut().set_buf(vtx_data);
                    selection_vao
                        .buffer_mut()
                        .gl_bind(GlBufferUsage::DynamicDraw);
                    unsafe {
                        gl::DepthFunc(gl::LEQUAL);
                        gl::ActiveTexture(gl::TEXTURE0);
                        gl::BindTexture(gl::TEXTURE_2D, *selection_texture);
                    }
                    selection_vao.gl_draw();
                } else {
                    unreachable!("selected cube wasn't a unit cube");
                }
            }
            None => (),
        }

        crosshair_shader_program.set_used();
        unsafe {
            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, *crosshair_texture);
        }
        crosshair_vao.gl_draw();
    }
}
