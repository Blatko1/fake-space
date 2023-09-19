use super::{RayHit, Raycaster};

const PURPLE: [u8; 4] = [200, 0, 220, 255];

impl Raycaster {
    pub fn draw_void(&self, hit: RayHit, data: &mut [u8]) {
        data.chunks_exact_mut(4)
            .skip(hit.screen_x as usize)
            .step_by(self.width as usize)
            .for_each(|rgba| unsafe {
                std::ptr::copy_nonoverlapping(
                    PURPLE.as_ptr(),
                    rgba.as_mut_ptr(),
                    rgba.len(),
                );
            });
    }
}
