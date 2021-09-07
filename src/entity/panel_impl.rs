use bytemuck::{Pod, Zeroable};

use super::Panel;

impl Panel {
    pub fn new(point0: [f32; 3], point1: [f32; 3], normal: [f32; 3], color: [f32; 3]) -> Self {
        let mut points = [0.0f32; 8];
        points[..3].clone_from_slice(&point0[..]);
        points[3] = 1.0;
        points[4..7].clone_from_slice(&point1[..]);
        points[7] = 1.0;

        let mut p_normal = [0.0f32; 4];
        p_normal[..3].clone_from_slice(&normal[..]);
        p_normal[3] = 0.0;

        Panel {
            points,
            normal: p_normal,
            color,
            _place_holder: -10.0,
        }
    }

    pub fn raw_data(&self) -> Vec<f32> {
        let mut res = vec![];
        for num in self.points {
            res.push(num);
        }
        for num in self.normal {
            res.push(num);
        }
        for num in self.color {
            res.push(num);
        }
        res.push(-2.0);

        res
    }
}

unsafe impl Zeroable for Panel {}
unsafe impl Pod for Panel {}
