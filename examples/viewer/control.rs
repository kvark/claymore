use cgmath;
use claymore_scene::Transform;


pub type MousePos = (i32, i32);

pub struct Control {
    rotate_speed: f32,
    move_speed: f32,
    zoom_speed: f32,
    rotate_base: Option<(MousePos, Transform<f32>)>,
    move_base: Option<(MousePos, cgmath::Vector3<f32>)>,
    last_pos: MousePos,
    space: Transform<f32>,
}

impl Control {
    pub fn new(rot_speed: f32, move_speed: f32, zoom_speed: f32,
               space: Transform<f32>) -> Control {
        Control {
            rotate_speed: rot_speed,
            move_speed: move_speed,
            zoom_speed: zoom_speed,
            rotate_base: None,
            move_base: None,
            last_pos: (0, 0),
            space: space,
        }
    }

    pub fn rot_capture(&mut self, transform: &Transform<f32>) {
        self.rotate_base = Some((self.last_pos, transform.clone()));
    }

    pub fn rot_release(&mut self) {
        self.rotate_base = None;
    }

    pub fn move_capture(&mut self, transform: &Transform<f32>) {
        self.move_base = Some((self.last_pos, transform.disp));
    }

    pub fn move_release(&mut self) {
        self.move_base = None;
    }

    pub fn position(&mut self, coords: MousePos,
                    transform: &mut Transform<f32>) {
        self.last_pos = coords;
        match self.rotate_base {
            Some((ref base_pos, ref base_transform)) => {
                use cgmath::Transform;
                // p' = Mp * Tc^ * (Tr * Rz * Tr^) * p
                // Tx = (Tr * Rz^ * Tr^) * Tc
                let path = (coords.0 - base_pos.0) as f32 * -self.rotate_speed;
                let rotation = cgmath::Decomposed {
                    scale: 1.0,
                    rot: cgmath::Rotation3::from_axis_angle(
                        &cgmath::vec3(0.0, 0.0, 1.0), cgmath::rad(path)),
                    disp: cgmath::zero(),
                };
                let space_inv = self.space.invert().unwrap();
                let relative = self.space.concat(&rotation.concat(&space_inv));
                *transform = relative.concat(base_transform);
            },
            None => (),
        }
        match self.move_base {
            Some((base_pos, ref base_disp)) => {
                use cgmath::{Vector, Rotation};
                let local_vector = cgmath::vec3(
                    -(coords.0 - base_pos.0) as f32,
                     (coords.1 - base_pos.1) as f32,
                    0.0).mul_s(self.move_speed);
                let cam_vector = transform.rot.rotate_vector(&local_vector);
                transform.disp = base_disp.add_v(&cam_vector);
            },
            None => (),
        }
    }

    pub fn wheel(&mut self, shift: i32, transform: &mut Transform<f32>) {
        use cgmath::{Vector, Transform};
        let vector = transform.transform_vector(&cgmath::vec3(0.0, 0.0, 1.0));
        transform.disp.add_self_v(&vector.mul_s(shift as f32 * -self.zoom_speed));
    }
}
