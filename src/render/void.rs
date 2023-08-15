use super::{RayCast, Raycaster};

const PURPLE: [u8; 4] = [200, 0, 220, 255];

impl Raycaster {
    pub fn draw_void(&self, ray: &RayCast, data: &mut [u8]) {
        for y in 0..self.height - 1 {
            let index = (self.height as usize - 1 - y as usize)
                * self.four_width
                + ray.draw_x_offset;
            data[index..index + 4].copy_from_slice(&PURPLE);
        }
        //let begin = (ray.screen_x * 4 * self.height) as usize;
        //let end = (ray.screen_x * 4 * self.height + self.height * 4) as usize;
        //data[begin..end]
        //    .chunks_exact_mut(4)
        //    .for_each(|rgba| rgba.copy_from_slice(&PURPLE));
    }
}
