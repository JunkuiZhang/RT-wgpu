struct Sphere {
    position: vec4<f32>;
    color: vec3<f32>;
    radius: f32;
};

struct Panel {
    point0: vec4<f32>;
    point1: vec4<f32>;
    normal: vec4<f32>;
    color: vec3<f32>;
    _placholder: f32;
};

struct PixelData {
    col_row: vec2<f32>;
};

[[block]]
struct InputBuffer {
    data: [[stride(8)]] array<PixelData>;
};

struct PixelColor {
    r: f32;
    g: f32;
    b: f32;
    // data: u32;
};

[[block]]
struct ResultBuffer {
    data: [[stride(12)]] array<PixelColor>;
    // data: [[stride(4)]] array<PixelColor>;
};

[[group(0), binding(0)]]
var<storage, read> input_list: InputBuffer;
[[group(0), binding(1)]]
var<storage, read_write> output_list: ResultBuffer;

[[block]]
struct SphereList {
    data: [[stride(32)]] array<Sphere, 1>;
};

[[group(1), binding(0)]]
var<uniform> sphere_list: SphereList;

[[block]]
struct PanelList {
    data: [[stride(64)]] array<Panel, 5>;
};

[[group(1), binding(1)]]
var<uniform> panel_list: PanelList;

[[block]]
struct LightList {
    data: [[stride(64)]] array<Panel, 1>;
};

[[group(1), binding(2)]]
var<uniform> light_list: LightList;

[[block]]
struct ConfigData {
    seed: u32;
    window_width: u32;
    window_height: u32;
    spp: u32;
    sphere_count: u32;
    panel_count: u32;
    light_count: u32;
    _p: u32;
};

[[group(1), binding(3)]]
var<storage, read> config_data: ConfigData;

struct HitInfo {
    hit_point: vec4<f32>;
    hit_normal: vec4<f32>;
    albedo: vec3<f32>;
    t: f32;
    hit_material: i32;
};

struct RandGen1Res {
    rng_state: u32;
    number: f32;
};
// source: https://www.reedbeta.com/blog/hash-functions-for-gpu-rendering/
fn rand_float_generate(rng_state: u32) -> RandGen1Res {
    var res: RandGen1Res;
    let state: u32 = rng_state;
    res.rng_state = state * 747796405u + 2891336453u;
    var word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    res.number = f32((word >> 22u) ^ word) / 4294967296.0;
    return res;
}

struct RandGen2Res {
    rng_state: u32;
    number0: f32;
    number1: f32;
};

fn rand_float_2_generate(rng_state: u32) -> RandGen2Res {
    var res: RandGen2Res;
    var state: u32 = rng_state;
    var word: u32 = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    state = state * 747796405u + 2891336453u;
    res.number0 = f32((word >> 22u) ^ word) / 4294967296.0;
    word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    res.rng_state = state * 747796405u + 2891336453u;
    res.number1 = f32((word >> 22u) ^ word) / 4294967296.0;
    return res;
}

fn point_in_points(point: vec4<f32>, point0: vec4<f32>, point1: vec4<f32>, normal: vec4<f32>) -> bool {
    let x_axis = vec4<f32>(1.0, 0.0, 0.0, 0.0);
    let y_axis = vec4<f32>(0.0, 1.0, 0.0, 0.0);
    let z_axis = vec4<f32>(0.0, 0.0, 1.0, 0.0);
    if (abs(dot(x_axis, normal)) > 0.001) {
        if (point0.y < point.y && point.y < point1.y && point0.z < point.z && point.z < point1.z) {
            return true;
        }
    }
    if (abs(dot(y_axis, normal)) > 0.001) {
        if (point0.x < point.x && point.x < point1.x && point0.z < point.z && point.z < point1.z) {
            return true;
        }
    }
    if (abs(dot(z_axis, normal)) > 0.001) {
        if (point0.x < point.x && point.x < point1.x && point0.y < point.y && point.y < point1.y) {
            return true;
        }
    }

    return false;
}

fn sphere_intersection(ray_origin: vec4<f32>, ray_direction: vec4<f32>, hit_info: HitInfo) -> HitInfo {
    let sphere_number: i32 = i32(config_data.sphere_count);
    var sphere_index: i32 = 0;
    var hit_rec: HitInfo;
    hit_rec = hit_info;
    loop {
        if (sphere_index >= sphere_number) {
            break;
        }

        let sphere = sphere_list.data[sphere_index];
        let a = dot(ray_direction, ray_direction);
        let oc = ray_origin - sphere.position;
        let b = 2.0 * dot(oc, ray_direction);
        let c = dot(oc, oc) - sphere.radius * sphere.radius;
        let indicator = b * b - 4.0 * a * c;
        if (indicator < 0.0) {
            continue;
        }
        let sphere_t = (-b - sqrt(indicator)) / (2.0 * a);
        if (sphere_t < 0.0) {
            continue;
        }
        if (sphere_t < hit_rec.t) {
            hit_rec.hit_point = ray_origin + sphere_t * ray_direction;
            hit_rec.hit_normal = normalize(hit_rec.hit_point - sphere.position);
            hit_rec.albedo = sphere.color;
            hit_rec.t = sphere_t;
            hit_rec.hit_material = 5;
        }

        continuing {
            sphere_index = sphere_index + 1;
        }
    }

    return hit_rec;
}

fn panel_intersection(ray_origin: vec4<f32>, ray_direction: vec4<f32>, hit_info: HitInfo) -> HitInfo {
    let panel_number: i32 = i32(config_data.panel_count);
    var panel_index: i32 = 0;
    var hit_rec: HitInfo;
    hit_rec = hit_info;
    loop {
        if (panel_index >= panel_number) {
            break;
        }

        let panel = panel_list.data[panel_index];
        var cos_dir_nor: f32 = dot(panel.normal, ray_direction);
        if (cos_dir_nor > 0.0) {
            continue;
        }
        var panel_t: f32;
        var panel_hit_point: vec4<f32>;
        panel_t = -dot(ray_origin - panel.point0, panel.normal) / dot(ray_direction, panel.normal);
        panel_hit_point = ray_origin + panel_t * ray_direction;
        if (panel_t < 0.0) {
            continue;
        }
        // hitpoint in points
        let in_points = point_in_points(panel_hit_point, panel.point0, panel.point1, panel.normal);
        if (panel_t < hit_rec.t && in_points) {
            hit_rec.hit_point = ray_origin + panel_t * ray_direction;
            hit_rec.hit_normal = panel.normal;
            hit_rec.albedo = panel.color;
            hit_rec.t = panel_t;
            hit_rec.hit_material = 5;
        }

        continuing {
            panel_index = panel_index + 1;
        }
    }

    return hit_rec;
}

fn light_intersection(ray_origin: vec4<f32>, ray_direction: vec4<f32>, hit_info: HitInfo) -> HitInfo {
    let panel_number: i32 = i32(config_data.panel_count);
    var panel_index: i32 = 0;
    var hit_rec: HitInfo;
    hit_rec = hit_info;
    loop {
        if (panel_index >= panel_number) {
            break;
        }

        let panel = light_list.data[panel_index];
        var cos_dir_nor: f32 = dot(panel.normal, ray_direction);
        if (cos_dir_nor > 0.0) {
            continue;
        }
        var panel_t: f32;
        var panel_hit_point: vec4<f32>;
        panel_t = -dot(ray_origin - panel.point0, panel.normal) / dot(ray_direction, panel.normal);
        panel_hit_point = ray_origin + panel_t * ray_direction;
        if (panel_t < 0.0) {
            continue;
        }
        // hitpoint in points
        let in_points = point_in_points(panel_hit_point, panel.point0, panel.point1, panel.normal);
        if (panel_t < hit_rec.t && in_points) {
            hit_rec.hit_point = ray_origin + panel_t * ray_direction;
            hit_rec.hit_normal = panel.normal;
            hit_rec.albedo = panel.color;
            hit_rec.t = panel_t;
            hit_rec.hit_material = 105;
        }

        continuing {
            panel_index = panel_index + 1;
        }
    }

    return hit_rec;
}

fn ray_intersect(ray_origin: vec4<f32>, ray_direction: vec4<f32>) -> HitInfo {
    var hit_rec: HitInfo;
    hit_rec.t = 1000000000.0;
    hit_rec.albedo = vec3<f32>(0.0, 0.0, 0.0);
    hit_rec.hit_material = -1;
    // sphere intersect
    hit_rec = sphere_intersection(ray_origin, ray_direction, hit_rec);

    // panel intersection
    hit_rec = panel_intersection(ray_origin, ray_direction, hit_rec);

    // light intersection
    hit_rec = light_intersection(ray_origin, ray_direction, hit_rec);

    return hit_rec;
}

fn ray_intersect_without_light(ray_origin: vec4<f32>, ray_direction: vec4<f32>) -> HitInfo {
    var hit_rec: HitInfo;
    hit_rec.t = 1000000000.0;
    hit_rec.albedo = vec3<f32>(0.0, 0.0, 0.0);
    hit_rec.hit_material = -1;
    // sphere intersect
    hit_rec = sphere_intersection(ray_origin, ray_direction, hit_rec);

    // panel intersection
    hit_rec = panel_intersection(ray_origin, ray_direction, hit_rec);

    return hit_rec;
}

struct ThetaPhiData {
    sin_theta: f32;
    cos_theta: f32;
    sin_phi: f32;
    cos_phi: f32;
};

fn get_theta_phi_data(given_normal: vec4<f32>) -> ThetaPhiData {
    var sin_theta: f32 = 0.0;
    var cos_theta: f32 = 0.0;
    var sin_phi: f32 = 0.0;
    var cos_phi: f32 = 0.0;

    let x_unit = vec4<f32>(1.0, 0.0, 0.0, 0.0);
    let y_unit = vec4<f32>(0.0, 1.0, 0.0, 0.0);
    cos_theta = dot(given_normal, y_unit);
    sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    let temp_vec = vec4<f32>(given_normal.x, 0.0, given_normal.z, 0.0);
    if (dot(temp_vec, temp_vec) > 0.001) {
        let xz = normalize(temp_vec);
        cos_phi = dot(xz, x_unit);
        sin_phi = sqrt(1.0 - cos_phi * cos_phi);
        if (given_normal.z < 0.0) {
            sin_phi = -sin_phi;
        }
    } else {
        cos_phi = 1.0;
    }

    var res: ThetaPhiData;
    res.sin_theta = sin_theta;
    res.cos_theta = cos_theta;
    res.sin_phi = sin_phi;
    res.cos_phi = cos_phi;

    return res;
}

fn rotate_vec_given_normal(temp_vec: vec4<f32>, normal: vec4<f32>) -> vec4<f32> {
    let theta_phi_data = get_theta_phi_data(normal);
    var trans_rotate_z: mat4x4<f32>;
    trans_rotate_z[0] = vec4<f32>(theta_phi_data.cos_theta, -theta_phi_data.sin_theta, 0.0, 0.0);
    trans_rotate_z[1] = vec4<f32>(theta_phi_data.sin_theta, theta_phi_data.cos_theta, 0.0, 0.0);
    trans_rotate_z[2] = vec4<f32>(0.0, 0.0, 1.0, 0.0);
    trans_rotate_z[3] = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    var trans_rotate_y: mat4x4<f32>;
    trans_rotate_y[0] = vec4<f32>(theta_phi_data.cos_phi, 0.0, -theta_phi_data.sin_phi, 0.0);
    trans_rotate_y[1] = vec4<f32>(0.0, 1.0, 0.0, 0.0);
    trans_rotate_y[2] = vec4<f32>(-theta_phi_data.sin_phi, 0.0, theta_phi_data.cos_phi, 0.0);
    trans_rotate_y[3] = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    return trans_rotate_y * trans_rotate_z * temp_vec;
}

struct ScatterResult {
    dir: vec4<f32>;
    pdf: f32;
    rng_state: u32;
};

fn generate_scatter_ray_dir(hit_info: HitInfo, rng_state: u32) -> ScatterResult {
    var state: u32 = rng_state;
    if (hit_info.hit_material < 10) {
        var result: ScatterResult;

        let rand2_res = rand_float_2_generate(state);
        state = rand2_res.rng_state;
        let a: f32 = rand2_res.number0;
        let b: f32 = rand2_res.number1;

        let sin_theta = sqrt(a);
        let cos_theta = sqrt(1.0 - a);
        let sin_phi = sin(2.0 * 3.141592653 * b);
        let cos_phi = cos(2.0 * 3.141592653 * b);
        let pdf = cos_theta / 3.141592653;
        let temp_dir = vec4<f32>(sin_theta * cos_phi, cos_theta, sin_theta * sin_phi, 0.0);

        let dir = rotate_vec_given_normal(temp_dir, hit_info.hit_normal);
        // let dir = hit_info.hit_normal;

        result.dir = dir;
        result.pdf = cos_theta / 3.141592653;
        result.rng_state = state;

        return result;
    }
    var result: ScatterResult;
    result.dir = vec4<f32>(0.0, 1.0, 0.0, 0.0);
    result.pdf = 1.0;
    result.rng_state = state;

    return result;
}

struct LightShadingData {
    sample_point: vec4<f32>;
    pdf_mul: f32;
    rng_state: u32;
};

fn light_get_direct_shading_data(panel_point0: vec4<f32>, panel_point1: vec4<f32>, panel_normal: vec4<f32>, rng_state: u32) -> LightShadingData {
    var state: u32 = rng_state;
    var pdf_mul: f32 = -1.0;
    var sample_point: vec4<f32>;
    var res: LightShadingData;
    if (abs(panel_normal.x) > 0.0) {
        let y_width = panel_point1.y - panel_point0.y;
        let z_width = panel_point1.z - panel_point0.z;

        let rand2_res = rand_float_2_generate(state);
        state = rand2_res.rng_state;
        let y_sample = rand2_res.number0 * y_width + panel_point0.y;
        let z_sample = rand2_res.number1 * z_width + panel_point0.z;

        pdf_mul = y_width * z_width;
        sample_point = vec4<f32>(panel_point0.x, y_sample, z_sample, 1.0);
    }
    if (abs(panel_normal.y) > 0.0) {
        let x_width = panel_point1.x - panel_point0.x;
        let z_width = panel_point1.z - panel_point0.z;

        let rand2_res = rand_float_2_generate(state);
        state = rand2_res.rng_state;
        let x_sample = rand2_res.number0 * x_width + panel_point0.x;
        let z_sample = rand2_res.number1 * z_width + panel_point0.z;

        pdf_mul = x_width * z_width;
        sample_point = vec4<f32>(x_sample, panel_point0.y, z_sample, 1.0);
    }
    if (abs(panel_normal.z) > 0.0) {
        let y_width = panel_point1.y - panel_point0.y;
        let x_width = panel_point1.x - panel_point0.x;

        let rand2_res = rand_float_2_generate(state);
        state = rand2_res.rng_state;
        let y_sample = rand2_res.number0 * y_width + panel_point0.y;
        let x_sample = rand2_res.number1 * x_width + panel_point0.z;

        pdf_mul = y_width * x_width;
        sample_point = vec4<f32>(x_sample, y_sample, panel_point0.z, 1.0);
    }
    res.pdf_mul = pdf_mul;
    res.sample_point = sample_point;
    res.rng_state = state;
    return res;
}

struct PointIntersectData {
    ray_in_direction: vec4<f32>;
    point: vec4<f32>;
    point_normal: vec4<f32>;
    point_albedo: vec3<f32>;
    pdf_mul: f32;
    light_sample_point: vec3<f32>;
    point_material: i32;
};

struct HitInfoArrayData {
    data: [[stride(80)]] array<PointIntersectData, 200>;
    rng_state: u32;
};

fn generate_hit_info_array(ray_in_origin: vec4<f32>, ray_in_direction: vec4<f32>, rng_state: u32) -> HitInfoArrayData {
    var state: u32 = rng_state;
    var res: HitInfoArrayData;

    var array_index: u32 = 0u;
    var ray_origin: vec4<f32> = ray_in_origin;
    var ray_direction: vec4<f32> = ray_in_direction;
    var missed: bool = false;
    var light_indicator: bool = false;
    loop {
        if (array_index >= 200u) {
            break;
        }
        if (missed) {
            var intersect_data: PointIntersectData;
            intersect_data.point_material = -15;
            res.data[array_index] = intersect_data;
        }
        if (light_indicator) {
            var intersect_data: PointIntersectData;
            intersect_data.point_material = 205;
        }

        let hit_info = ray_intersect(ray_origin, ray_direction);
        let light_panel = light_list.data[0];
        let sample_data = light_get_direct_shading_data(light_panel.point0, light_panel.point1, light_panel.normal, state);
        state = sample_data.rng_state;

        var intersect_data: PointIntersectData;
        intersect_data.ray_in_direction = ray_direction;
        intersect_data.point = hit_info.hit_point;
        intersect_data.point_normal = hit_info.hit_normal;
        intersect_data.point_albedo = hit_info.albedo;
        intersect_data.point_material = hit_info.hit_material;
        intersect_data.pdf_mul = sample_data.pdf_mul;
        intersect_data.light_sample_point = vec3<f32>(sample_data.sample_point.x, sample_data.sample_point.y, sample_data.sample_point.z);

        if (hit_info.hit_material < 0) {
            missed = true;
        }
        if (hit_info.hit_material > 100) {
            light_indicator = true;
        }

        ray_origin = hit_info.hit_point;
        let scatter_data = generate_scatter_ray_dir(hit_info, state);
        ray_direction = scatter_data.dir;
        intersect_data.point_normal[3] = scatter_data.pdf;
        state = scatter_data.rng_state;

        res.data[array_index] = intersect_data;
        continuing {
            array_index = array_index + 1u;
        }
    }
    res.rng_state = state;
    return res;
}

fn shade_point_array(point_array: HitInfoArrayData) -> vec3<f32> {
    var info_array: array<PointIntersectData, 200> = point_array.data;
    var shade_color: vec3<f32>;
    var index: i32 = 199;
    loop {
        if (index < 0) {
            break;
        }

        let point_info = info_array[index];
        if (point_info.point_material < -10 || point_info.point_material > 200) {
            continue;
        }
        if (point_info.point_material < 0) {
            shade_color = vec3<f32>(0.5, 0.5, 0.5);
            continue;
        }
        if (point_info.point_material > 100) {
            shade_color = point_info.point_albedo;
            continue;
        }
        let ray_in_dir = point_info.ray_in_direction;
        let point = point_info.point;
        let point_normal = vec4<f32>(point_info.point_normal.x, point_info.point_normal.y, point_info.point_normal.z, 0.0);
        let point_albedo = point_info.point_albedo;
        let point_pdf_mul = point_info.pdf_mul;
        let point_pdf = point_info.point_normal[3];
        let point_sample_point = vec4<f32>(point_info.light_sample_point, 1.0);

        // fresnel factor
        let f0: f32 = 0.45;
        let fresnel_factor = f0 + (1.0 - f0) * pow(abs(dot(ray_in_dir, point_normal)), 5.0);

        // if (index == 0) {
        //     shade_color = point_albedo * fresnel_factor;
        // } else {
        //     continue;
        // }

        // direct shading
        var direct_shade_res: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
        var light_index: i32 = 0;
        let light_count: i32 = i32(config_data.light_count);
        loop {
            if (light_index >= light_count) {
                break;
            }

            let light_panel = light_list.data[light_index];
            if (dot(point_normal, light_panel.normal) > 0.0) {
                continue;
            }
            
            let ptsp = point_sample_point - point;
            let length_square = dot(ptsp, ptsp);
            let temp_dir = normalize(ptsp);
            let temp_hit_info = ray_intersect_without_light(point, temp_dir);
            let temp_pp = temp_hit_info.hit_point - point;
            if (dot(temp_pp, temp_pp) < length_square) {
                continue;
            }

            let cos_theta = abs(dot(temp_dir, point_normal));
            let cos_theta_prime = abs(dot(temp_dir, light_panel.normal));
            direct_shade_res = direct_shade_res + point_albedo * light_panel.color * cos_theta * cos_theta_prime * point_pdf_mul * fresnel_factor / length_square;
            // direct_shade_res = direct_shade_res + light_panel.color * cos_theta * cos_theta_prime * point_pdf_mul * fresnel_factor / length_square;
            // direct_shade_res = direct_shade_res + point_albedo * light_panel.color * 0.00001 * cos_theta * cos_theta_prime;

            continuing {
                light_index = light_index + 1;
            }
        }

        // indirect shading
        var indirect_shade_res: vec3<f32>;
        if (index == 199) {
            indirect_shade_res = vec3<f32>(0.0, 0.0, 0.0);
        } else {
            indirect_shade_res = shade_color * point_albedo * dot(point_normal, info_array[index + 1].ray_in_direction) * fresnel_factor * point_pdf;
        }

        shade_color = direct_shade_res + indirect_shade_res;

        // if (index <= 0u) {
        //     break;
        // }
        continuing {
            index = index - 1;
        }
    }

    return shade_color;
}

// fn generate_spp_array(screen_pos: vec2<f32>) -> array<vec2<f32>, 20> {
//     var res: array<vec2<f32>, 20>;
//     var index: i32 = 0;
//     let total_sample_num: i32 = i32(config_data.spp);
//     loop {
//         if (index >= total_sample_num) {
//             break;
//         } 

//         let rand_col: f32 = rand_float_generate();
//         let rand_row: f32 = rand_float_generate();
//         let x: f32 = screen_pos.x + rand_col - f32(config_data.window_width) / 2.0;
//         let y: f32 = f32(config_data.window_height) / 2.0 - screen_pos.y + rand_row;

//         res[index] = vec2<f32>(x, y);

//         continuing {
//             index = index + 1;
//         }
//     }

//     return res;
// }

[[stage(compute), workgroup_size(64)]]
fn main([[builtin(global_invocation_id)]] gi_id: vec3<u32>) {
    var out_color: vec3<f32> = vec3<f32>(0.0, 0.0, 0.0);
    let entity_index = i32(gi_id.x);
    var rng_state: u32 = gi_id.x;

    let camera_pos = vec4<f32>(300.0, 300.0, 800.0, 1.0);
    let window_width = f32(config_data.window_width);
    let window_height = f32(config_data.window_height);

    var n: i32 = 0;
    let spp: i32 = i32(config_data.spp);
    let screen_pos = input_list.data[entity_index].col_row;
    loop {
        if (n >= spp) {
            out_color = out_color / f32(n);
            break;
        }

        var res_albedo: vec3<f32>;
        var ray_origin: vec4<f32> = camera_pos;
        let rand2_gen_res = rand_float_2_generate(rng_state);
        rng_state = rand2_gen_res.rng_state;
        let rand_col: f32 = rand2_gen_res.number0;
        let rand_row: f32 = rand2_gen_res.number1;
        let x: f32 = screen_pos.x + rand_col - window_width / 2.0;
        let y: f32 = window_height / 2.0 - screen_pos.y + rand_row;
        var ray_direction: vec4<f32> = normalize(vec4<f32>(x, y, 0.0, 1.0) - vec4<f32>(0.0, 0.0, 800.0, 1.0));
        let hit_array_data = generate_hit_info_array(ray_origin, ray_direction, rng_state);
        rng_state = hit_array_data.rng_state;
        res_albedo = shade_point_array(hit_array_data);
        
        continuing {
            n = n + 1;
            out_color = out_color + res_albedo;
        }
    }
    
    var res_color: PixelColor;
    // let r: f32 = clamp(sqrt(out_color.x), 0.0, 1.0) * 255.0;
    // let g: f32 = clamp(sqrt(out_color.y), 0.0, 1.0) * 255.0;
    // let b: f32 = clamp(sqrt(out_color.z), 0.0, 1.0) * 255.0;
    res_color.r = out_color.x;
    res_color.g = out_color.y;
    res_color.b = out_color.z;
    // let res_data: u32 = u32(exp2(24.0) * r) + u32(exp2(16.0) * g) + u32(exp2(8.0) * b) + 255u;
    output_list.data[entity_index] = res_color;
    // output_list.data[entity_index].data = res_data;
}