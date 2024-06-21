use bevy::prelude::*;
use rand::Rng as _;
use crate::{rng::Rng, textures::{MountainTexture, MountainTextures}};

#[derive(Resource)]
pub struct ErosionSettings {
    pub max_lifetime: usize,
    pub erosion_radius: i32,
    pub inertia: f32,
    pub sediment_capacity_factor: f32,
    pub min_sediment_capacity: f32,
    pub erode_speed: f32,
    pub deposit_speed: f32,
    pub evaporation_speed: f32,
    pub gravity: f32,
    pub start_speed: f32,
    pub start_water: f32,
    pub brush_indices: Vec<i32>,
    pub brush_weights: Vec<f32>,
}

impl Default for ErosionSettings {
    fn default() -> Self {
        Self {
            max_lifetime: 30,
            erosion_radius: 3,
            inertia: 0.3,
            sediment_capacity_factor: 3.0,
            min_sediment_capacity: 0.01,
            erode_speed: 0.3,
            deposit_speed: 0.3,
            evaporation_speed: 0.01,
            gravity: 4.0,
            start_speed: 1.0,
            start_water: 1.0,
            brush_indices: Vec::new(),
            brush_weights: Vec::new(),
        }
    }
}

pub fn erode_init(
    mut settings: ResMut<ErosionSettings>,
    textures: Res<MountainTextures>,
) {
    let width = textures.heightmap.width();
    settings.brush_indices.clear();
    settings.brush_weights.clear();

    let mut weight_sum = 0.0;
    for brush_y in -settings.erosion_radius..=settings.erosion_radius {
        for brush_x in -settings.erosion_radius..=settings.erosion_radius {
            let sqr_dst = brush_x * brush_x + brush_y * brush_y;
            if sqr_dst < settings.erosion_radius * settings.erosion_radius {
                settings.brush_indices.push(brush_y * width as i32 + brush_x);
                let weight = 1.0 - (sqr_dst as f32).sqrt() / settings.erosion_radius as f32;
                weight_sum += weight;
                settings.brush_weights.push(weight);
            }
        }
    }

    for w in settings.brush_weights.iter_mut() {
        *w /= weight_sum;
    }
}

pub fn erode_main(
    mut textures: ResMut<MountainTextures>,
    // raw: Res<MountainTexturesRaw>,
    // mut images: ResMut<Assets<Image>>,
    mut rng: ResMut<Rng>,
    settings: Res<ErosionSettings>,
) {
    // let Some(im) = images.get_mut(&raw.map) else { return };

    let map_width = textures.heightmap.width();
    let map_height = textures.heightmap.height();
    let map_size = textures.heightmap.size();
    let index = rng.rng.gen_range(0..textures.heightmap.size());
    let mut pos_x = (index % map_width) as f32 / map_width as f32;
    let mut pos_y = (index / map_width) as f32 / map_height as f32;

    let mut dx = 0.0;
    let mut dy = 0.0;
    let mut sediment = 0.0;
    let mut speed = settings.start_speed;
    let mut water = settings.start_water;

    for _ in 0..settings.max_lifetime {
        let node_x = ((pos_x * map_width as f32).floor() as usize).min(map_width - 1);
        let node_y = ((pos_y * map_height as f32).floor() as usize).min(map_height - 1);
        let droplet_idx = node_y * map_width + node_x;
    
        let cell_offset_x = (pos_x * map_width as f32) - node_x as f32;
        let cell_offset_y = (pos_y * map_height as f32) - node_y as f32;

        let (height, g) = find_height_and_grad(pos_x, pos_y, map_width, map_height, &textures.heightmap);

        dx = dx * settings.inertia - g.x * (1.0 - settings.inertia);
        dy = dy * settings.inertia - g.y * (1.0 - settings.inertia);

        let len = (dx * dx + dy * dy).sqrt().max(0.0001);
        dx /= len;
        dy /= len;

        pos_x += dx;
        pos_y += dy;

        if (dx == 0.0 && dy == 0.0) || !(0.0..1.0).contains(&pos_x) || !(0.0..1.0).contains(&pos_y) { break };

        let (new_height, _) = find_height_and_grad(pos_x, pos_y, map_width, map_height, &textures.heightmap);
        let delta_height = new_height - height;

        let sediment_capacity = settings.min_sediment_capacity
            .max(-delta_height * speed * water * settings.sediment_capacity_factor);

        if sediment > sediment_capacity || delta_height > 0.0 {
            let amount_to_deposit = if delta_height > 0.0 { delta_height.min(sediment) } else { (sediment - sediment_capacity) * settings.deposit_speed };
            sediment -= amount_to_deposit;

            textures.heightmap[droplet_idx] += amount_to_deposit * (1.0 - cell_offset_x) * (1.0 - cell_offset_y);
            if droplet_idx + 1 < map_size { textures.heightmap[droplet_idx + 1] += amount_to_deposit * cell_offset_x * (1.0 - cell_offset_y) };
            if droplet_idx + map_width < map_size { textures.heightmap[droplet_idx + map_width] += amount_to_deposit * (1.0 - cell_offset_x) * cell_offset_y };
            if droplet_idx + map_width + 1 < map_size { textures.heightmap[droplet_idx + map_width + 1] += amount_to_deposit * cell_offset_x * cell_offset_y };
        } else {
            let amount_to_erode = ((sediment_capacity - sediment) * settings.erode_speed).min(-delta_height);

            for i in 0..settings.brush_indices.len() {
                let erode_idx = droplet_idx + settings.brush_indices[i] as usize;
                
                let weighted_erode_amount = amount_to_erode * settings.brush_weights[i];
                let erode_height = textures.heightmap.get(erode_idx).unwrap_or(new_height);
                let delta_sediment = if erode_height < weighted_erode_amount { erode_height } else { weighted_erode_amount };
                if erode_idx < textures.heightmap.size() {
                    textures.heightmap[erode_idx] -= delta_sediment;
                }
                sediment += delta_sediment;
            }
        }
        
        speed = (speed * speed + delta_height * settings.gravity).max(0.0).sqrt();
        water *= 1.0 - settings.evaporation_speed;
    }

    // *im = textures.as_raw();
}

fn get_height(im: &Image, x: usize, y: usize, width: usize) -> f32 {
    let i = (y * width + x) * std::mem::size_of::<f32>() * 4;
    let mut bits: u32 = unsafe { std::mem::transmute_copy(&im.data[i]) };
    bits |= unsafe { std::mem::transmute_copy::<u8, u32>(&im.data[i + 1]) } << 8;
    bits |= unsafe { std::mem::transmute_copy::<u8, u32>(&im.data[i + 2]) } << 16;
    bits |= unsafe { std::mem::transmute_copy::<u8, u32>(&im.data[i + 3]) } << 24;
    f32::from_bits(bits)
}

fn set_height(im: &mut Image, x: usize, y: usize, width: usize, h: f32) {
    let bits = h.to_bits();
    let i = (y * width + x) * std::mem::size_of::<f32>() * 4;
    im.data[i] = (bits & 0x000000FF) as u8;
    im.data[i + 1] = ((bits & 0x0000FF00) >> 8) as u8;
    im.data[i + 2] = ((bits & 0x00FF0000) >> 16) as u8;
    im.data[i + 3] = ((bits & 0xFF000000) >> 24) as u8;
}

fn find_height_and_grad(x: f32, y: f32, width: usize, height: usize, map: &MountainTexture) -> (f32, Vec2) {
    let node_x = (x * width as f32).floor() as usize;
    let node_y = (y * height as f32).floor() as usize;
    let droplet_idx = node_y * width + node_x;
    
    let cell_offset_x = (x * width as f32) - node_x as f32;
    let cell_offset_y = (y * height as f32) - node_y as f32;

    let height_nw = map[droplet_idx];
    let height_ne = map.get(droplet_idx + 1).unwrap_or(height_nw);
    let height_sw = map.get(droplet_idx + width).unwrap_or(height_nw);
    let height_se = map.get(droplet_idx + width + 1).unwrap_or(height_nw);

    let gx = (height_ne - height_nw) * (1.0 - cell_offset_y) + (height_se - height_sw) * cell_offset_y;
    let gy = (height_sw - height_nw) * (1.0 - cell_offset_x) + (height_se - height_ne) * cell_offset_x;

    let height = height_nw * (1.0 - cell_offset_x) * (1.0 - cell_offset_y) 
        + height_ne * cell_offset_x * (1.0 - cell_offset_y)
        + height_sw * (1.0 - cell_offset_x) * cell_offset_y
        + height_se * cell_offset_x * cell_offset_y;

    (height, Vec2::new(gx, gy))
}
