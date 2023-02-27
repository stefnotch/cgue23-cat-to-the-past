use bevy_ecs::system::Resource;
use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, Vector2, Vector3};

pub struct CameraSettings {
    fov: Rad<f32>,
    near: f32,
    far: f32,
}

#[derive(Resource)]
pub struct Camera {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,

    position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,

    settings: CameraSettings,
}

impl Camera {
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let position = Point3::new(0.0, 0.0, -6.0);
        let yaw = Rad::from(Deg(0.0));
        let pitch = Rad::from(Deg(0.0));

        let settings = CameraSettings {
            fov: Rad::from(Deg(fov)),
            near,
            far,
        };

        Camera {
            view: calculate_view(position, yaw, pitch),
            proj: cgmath::perspective(Deg(fov), aspect_ratio, near, far),
            position,
            yaw,
            pitch,
            settings,
        }
    }

    pub fn update_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.proj = cgmath::perspective(
            self.settings.fov,
            aspect_ratio,
            self.settings.near,
            self.settings.far,
        );
    }

    pub fn view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn proj(&self) -> &Matrix4<f32> {
        &self.proj
    }

    pub fn update(&mut self) {
        self.view = calculate_view(self.position, self.yaw, self.pitch);
    }
}

fn calculate_view(position: Point3<f32>, yaw: Rad<f32>, pitch: Rad<f32>) -> Matrix4<f32> {
    // See: https://sotrh.github.io/learn-wgpu/intermediate/tutorial12-camera/#the-camera
    let (sin_pitch, cos_pitch) = pitch.0.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.0.sin_cos();

    let cam_direction =
        Vector3::new(cos_pitch * sin_yaw, -sin_pitch, cos_pitch * cos_yaw).normalize();

    Matrix4::look_to_rh(position, cam_direction, -Vector3::unit_y())
}
