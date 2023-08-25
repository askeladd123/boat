use bevy::prelude::Vec3;

pub fn length_xz(value: &Vec3) -> f32 {
    (value.x * value.x + value.z * value.z).sqrt()
}
