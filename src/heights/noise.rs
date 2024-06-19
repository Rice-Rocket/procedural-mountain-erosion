use crate::textures::MountainTexture;

pub fn generate_heights(
    map: &mut MountainTexture,
) {
    for (x, y, px) in map.enumerate_pixels_mut() {
        *px = ((x as f32 + y as f32) / 10.0).sin() * 5.0;
    }
}
