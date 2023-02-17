use cgmath::{Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, Vector2, Vector3};
use winit::event::{ElementState, VirtualKeyCode};

pub struct CameraSettings {
    fov: Rad<f32>,
    near: f32,
    far: f32,
}

pub struct Camera {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,

    position: Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,

    settings: CameraSettings,
}

pub struct CameraController {
    movement: Vector3<f32>,
    speed: f32,
    sensitivity: f32,
    rotation: Vector2<f32>, // yaw, pitch
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

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        CameraController {
            movement: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector2::new(0.0, 0.0),
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {}

    pub fn process_mouse(&mut self, (dX, dY): (f32, f32)) {
        self.rotation.x += dX;
        self.rotation.y += dY;
    }

    pub fn update_camera(&mut self, camera: &mut Camera, delta_time: f64) {
        camera.yaw += Deg(self.rotation.x).into();
        camera.pitch += Deg(self.rotation.y).into();

        // TODO: prevent full pitch rotation

        self.rotation = Vector2::new(0.0, 0.0);
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
