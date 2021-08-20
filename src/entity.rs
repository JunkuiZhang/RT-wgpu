mod panel_impl;
mod sphere_impl;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub position: [f32; 4],
    pub color: [f32; 3],
    pub radius: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Panel {
    pub points: [f32; 8],
    pub normal: [f32; 4],
    pub color: [f32; 3],
    pub _place_holder: f32,
}
