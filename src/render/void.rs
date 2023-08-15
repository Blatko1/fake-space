use super::{RayCast, Raycaster};

const PURPLE: [u8; 4] = [200, 0, 220, 255];

pub fn draw(rc: &Raycaster, ray: &RayCast, data: &mut [u8]) {
    let draw_x_offset = 4 * (rc.width - ray.screen_x - 1) as usize;
    for y in 0..rc.height - 1 {
        let index = (rc.height as usize - 1 - y as usize) * rc.four_width
            + draw_x_offset;
        data[index..index + 4].copy_from_slice(&PURPLE);
    }
    //let begin = (ray.screen_x * 4 * rc.height) as usize;
    //let end = (ray.screen_x * 4 * rc.height + rc.height * 4) as usize;
    //data[begin..end]
    //    .chunks_exact_mut(4)
    //    .for_each(|rgba| rgba.copy_from_slice(&PURPLE));
}
