use bytemuck::{Pod, Zeroable};

use super::Sphere;

impl Sphere {
    pub fn new(pos: [f32; 3], color: [f32; 3], radius: f32) -> Self {
        let mut position = [0.0f32; 4];
        position[..3].clone_from_slice(&pos[..]);
        position[3] = 1.0;

        Sphere {
            position,
            color,
            radius,
        }
    }

    pub fn raw_data(&self) -> Vec<f32> {
        let mut res = vec![];
        for num in self.position {
            res.push(num);
        }
        res.push(1.0);
        for num in self.color {
            res.push(num);
        }
        res.push(self.radius);
        // res.push(-10.0);

        res
    }
}

unsafe impl Zeroable for Sphere {}
unsafe impl Pod for Sphere {}
