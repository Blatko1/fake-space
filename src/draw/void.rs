use super::{RayCast, Raycaster};

const PURPLE: [u8; 4] = [200, 0, 220, 255];

impl Raycaster {
    pub fn draw_void(&self, ray: &RayCast, data: &mut [u8]) {
        for y in 0..self.height - 1 {
            let index = (self.height as usize - 1 - y as usize)
                * self.four_width
                + ray.screen_x as usize * 4;
            data[index..index + 4].copy_from_slice(&PURPLE);
        }
    }
}
