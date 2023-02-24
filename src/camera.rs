use crate::input::InputMap;
use cgmath::{Deg, EuclideanSpace, InnerSpace, Matrix4, Point3, Rad, Vector2, Vector3};
use std::f32::consts::FRAC_PI_2;
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

pub struct PlayerController {
    speed: f32,
    sensitivity: f32,
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

impl PlayerController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        PlayerController { speed, sensitivity }
    }

    pub fn update_camera(&mut self, camera: &mut Camera, input: &InputMap, delta_time: f64) {
        let (dx, dy) = input.mouse_delta();

        camera.yaw += Deg(dx as f32 * self.sensitivity).into();
        camera.pitch += Deg(dy as f32 * self.sensitivity).into();

        const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

        if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
            camera.pitch = Rad(SAFE_FRAC_PI_2);
        }

        let mut direction: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        if input.is_pressed(VirtualKeyCode::W) {
            direction.z += 1.0;
        }
        if input.is_pressed(VirtualKeyCode::S) {
            direction.z += -1.0;
        }

        if input.is_pressed(VirtualKeyCode::A) {
            direction.x += -1.0;
        }
        if input.is_pressed(VirtualKeyCode::D) {
            direction.x += 1.0;
        }

        if input.is_pressed(VirtualKeyCode::Space) {
            direction.y += 1.0;
        }
        if input.is_pressed(VirtualKeyCode::LShift) {
            direction.y += -1.0;
        }

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_sin, 0.0, yaw_cos).normalize();
        let right = Vector3::new(yaw_cos, 0.0, -yaw_sin).normalize();
        let up = Vector3::unit_y();

        camera.position += forward * direction.z * self.speed * delta_time as f32;
        camera.position += right * direction.x * self.speed * delta_time as f32;
        camera.position += up * direction.y * self.speed * delta_time as f32;
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
