use crate::input::WindowResize;
use angle::{Angle, Deg, Rad};
use bevy_ecs::event::EventReader;
use bevy_ecs::system::{ResMut, Resource};
use nalgebra::{
    vector, Matrix, Matrix4, Perspective3, Point3, UnitQuaternion, UnitVector3, Vector4,
};

// TODO: look up how to get the euler yaw and pitch angles from a quaternion
// IDEA: remove position and orientation and use transforms instead
#[derive(Resource)]
pub struct Camera {
    view: Matrix4<f32>,
    proj: Perspective3<f32>,

    pub position: Point3<f32>,
    pub orientation: UnitQuaternion<f32>,
}

impl Camera {
    pub fn new(fov: Deg<f32>, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let position = Point3::new(0.0, 0.0, 8.0);
        let orientation = UnitQuaternion::identity();

        let fov = Rad::from(fov);

        Camera {
            view: calculate_view(position, orientation),
            proj: calculate_projection(aspect_ratio, fov, near, far),
            position,
            orientation,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.proj.set_aspect(aspect_ratio);
    }

    pub fn view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn proj(&self) -> Matrix4<f32> {
        // https://johannesugb.github.io/gpu-programming/setting-up-a-proper-vulkan-projection-matrix/
        // z does not get flipped because Perspective3 already gives us a matrix where z is flipped
        // see: https://nalgebra.org/docs/user_guide/cg_recipes#screen-space-to-view-space
        const DIAGONAL: Vector4<f32> = Vector4::new(1.0, -1.0, 1.0, 1.0);
        const FLIP_Y: Matrix4<f32> = Matrix4::from_diagonal(&DIAGONAL);

        self.proj.as_matrix() * FLIP_Y
    }

    pub fn update(&mut self) {
        self.view = calculate_view(self.position, self.orientation);
    }

    /// in world-space
    pub const fn forward() -> UnitVector3<f32> {
        UnitVector3::new_unchecked(vector![0.0, 0.0, -1.0])
    }

    /// in world-space
    pub const fn right() -> UnitVector3<f32> {
        UnitVector3::new_unchecked(vector![1.0, 0.0, 0.0])
    }

    /// in world-space
    pub const fn up() -> UnitVector3<f32> {
        UnitVector3::new_unchecked(vector![0.0, 1.0, 0.0])
    }
}

pub fn update_camera(mut camera: ResMut<Camera>) {
    camera.update();
}

pub fn update_camera_aspect_ratio(
    mut camera: ResMut<Camera>,
    mut reader: EventReader<WindowResize>,
) {
    for event in reader.iter() {
        camera.update_aspect_ratio(event.width as f32 / event.height as f32);
    }
}

fn calculate_projection(
    aspect_ratio: f32,
    fov: Rad<f32>,
    near: f32,
    far: f32,
) -> Perspective3<f32> {
    Perspective3::new(aspect_ratio, fov.value(), near, far)
}

fn calculate_view(position: Point3<f32>, orientation: UnitQuaternion<f32>) -> Matrix4<f32> {
    let cam_direction = orientation * Camera::forward();
    let target = position + cam_direction.into_inner();

    Matrix::look_at_rh(&position, &target, &Camera::up())
}
