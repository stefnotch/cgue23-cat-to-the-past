use cgmath::{Deg, EuclideanSpace, Matrix4, Point3, Rad, Vector3};

pub struct Camera {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,

    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new(fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Self {
        let position = Point3::origin();
        let yaw = Rad::from(Deg(-90.0));
        let pitch = Rad::from(Deg(0.0));

        Camera {
            view: calculate_view(position, yaw, pitch),
            proj: cgmath::perspective(Rad::from(Deg(fov)), aspect_ratio, near, far),
            position,
            yaw,
            pitch,
        }
    }

    pub fn update_aspect_ratio(&mut self, width: u32, height: u32) {}

    pub fn view(&self) -> &Matrix4<f32> {
        &self.view
    }

    pub fn proj(&self) -> &Matrix4<f32> {
        &self.proj
    }

    pub fn update(&mut self) {}
}

fn calculate_view(position: Point3<f32>, yaw: Rad<f32>, pitch: Rad<f32>) -> Matrix4<f32> {
    // See: https://sotrh.github.io/learn-wgpu/intermediate/tutorial12-camera/#the-camera
    let (sin_pitch, cos_pitch) = pitch.0.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.0.sin_cos();

    let cam_direction = Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw);

    Matrix4::look_to_rh(position, cam_direction, Vector3::unit_y())
}
