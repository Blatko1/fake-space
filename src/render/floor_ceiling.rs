use crate::textures::{BLUE_BRICK, LIGHT_PLANK};

use super::Raycaster;

impl Raycaster {
    pub fn draw_floor_and_ceiling(&self, data: &mut [u8]) {
        let ray_dir_x0 = self.dir.x - self.plane.x;
        let ray_dir_y0 = self.dir.y - self.plane.y;
        let ray_dir_x1 = self.dir.x + self.plane.x;
        let ray_dir_y1 = self.dir.y + self.plane.y;
        let pos_z = 0.5 * self.height as f32;

        let mut color = [0; 4];

        for y in self.height / 2..self.height {
            let p = y as f32 - self.height as f32 / 2.0;

            let row_dist = pos_z / p;

            let floor_step_x =
                row_dist * (ray_dir_x1 - ray_dir_x0) / self.width as f32;
            let floor_step_y =
                row_dist * (ray_dir_y1 - ray_dir_y0) / self.width as f32;

            let mut floor_x = self.pos.x + row_dist * ray_dir_x0;
            let mut floor_y = self.pos.y + row_dist * ray_dir_y0;

            for x in 0..self.width {
                let cellx = floor_x as i32;
                let celly = floor_y as i32;

                let tx = (16.0 * (floor_x - cellx as f32)) as u32 & (16 - 1);
                let ty = (16.0 * (floor_y - celly as f32)) as u32 & (16 - 1);

                floor_x += floor_step_x;
                floor_y += floor_step_y;

                let i = (16 * 4 * ty + tx * 4) as usize;

                color.copy_from_slice(&LIGHT_PLANK[i..i + 4]);
                let index =
                    (y * 4 * self.width + (self.width - x - 1) * 4) as usize;
                data[index..index + 4].copy_from_slice(&color);

                color.copy_from_slice(&BLUE_BRICK[i..i + 4]);
                let index = ((self.height - y - 1) * 4 * self.width
                    + (self.width - x - 1) * 4)
                    as usize;
                data[index..index + 4].copy_from_slice(&color);
            }
        }
    }
}
