use bytemuck::{Pod, Zeroable};

use super::Panel;

impl Panel {
    pub fn new(point0: [f32; 3], point1: [f32; 3], normal: [f32; 3], color: [f32; 3]) -> Self {
        let mut points = [0.0f32; 8];
        for i in 0..3 {
            points[i] = point0[i];
        }
        points[3] = 1.0;
        for i in 0..3 {
            points[4 + i] = point1[i];
        }
        points[7] = 1.0;

        let mut p_normal = [0.0f32; 4];
        for i in 0..3 {
            p_normal[i] = normal[i];
        }
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
