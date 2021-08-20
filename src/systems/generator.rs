use crate::{
    entity::{Panel, Sphere},
    settings::{
        SAMPLES_PER_PIXEL, TEXTURE_HEIGHT, TEXTURE_WIDTH, WINDOW_HEIGHT, WINDOW_TOTAL_PIXEL,
        WINDOW_WIDHT,
    },
};

pub fn generate_sphere_scene() -> Vec<Sphere> {
    let mut res = vec![];
    let sphere = Sphere::new([300.0, 60.0, -160.0], [0.0, 0.0, 0.7], 60.0);
    res.push(sphere);

    res
}

pub fn generate_panel_scene() -> Vec<Panel> {
    let mut res = vec![];
    let panel_top = Panel::new(
        [0.0, 600.0, -600.0],
        [600.0, 600.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.75, 0.75, 0.75],
    );
    res.push(panel_top);
    let panel_left = Panel::new(
        [0.0, 0.0, -600.0],
        [0.0, 600.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.12, 0.45, 0.15],
    );
    res.push(panel_left);
    let panel_back = Panel::new(
        [0.0, 0.0, -600.0],
        [600.0, 600.0, -600.0],
        [0.0, 0.0, 1.0],
        [0.75, 0.75, 0.75],
    );
    res.push(panel_back);
    let panel_right = Panel::new(
        [600.0, 0.0, -600.0],
        [600.0, 600.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.65, 0.05, 0.05],
    );
    res.push(panel_right);
    let panel_bottom = Panel::new(
        [0.0, 0.0, -600.0],
        [600.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.75, 0.75, 0.75],
    );
    res.push(panel_bottom);

    res
}

pub fn generate_lights_scene() -> Vec<Panel> {
    let mut res = vec![];
    let panel_light = Panel::new(
        [225.0, 599.0, -350.0],
        [375.0, 599.0, -200.0],
        [0.0, -1.0, 0.0],
        [7.0, 7.0, 7.0],
    );
    res.push(panel_light);

    res
}

pub fn generate_input_data() -> Vec<f32> {
    let mut result = Vec::with_capacity((WINDOW_TOTAL_PIXEL * 2) as usize);
    for row_num in 0..TEXTURE_HEIGHT as usize {
        for col_num in 0..TEXTURE_WIDTH as usize {
            result.push(col_num as f32);
            result.push(row_num as f32);
            // for _ in 0..SAMPLES_PER_PIXEL as usize {
            //     let x = col_num as f32 + rng.gen_range(0.0..1.0);
            //     let y = WINDOW_HEIGHT as f32 - row_num as f32 - rng.gen_range(0.0..1.0);
            //     result.push(x - (WINDOW_WIDHT / 2) as f32);
            //     result.push(y - (WINDOW_HEIGHT / 2) as f32);
            // }
        }
    }

    result
}
