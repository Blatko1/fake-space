use crate::textures::{LIGHT_PLANK, MOSSY_STONE};

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

            let draw_ceiling_y_offset = (self.height - y - 1) * 4 * self.width;
            let draw_floor_y_offset = y * 4 * self.width;

            for x in 0..self.width {
                let cellx = floor_x as i32;
                let celly = floor_y as i32;

                let tx_ceiling =
                    (16.0 * (floor_x - cellx as f32)) as u32 & (16 - 1);
                let ty_ceiling =
                    (16.0 * (floor_y - celly as f32)) as u32 & (16 - 1);

                let tx_floor =
                    (32.0 * (floor_x - cellx as f32)) as u32 & (32 - 1);
                let ty_floor =
                    (32.0 * (floor_y - celly as f32)) as u32 & (32 - 1);

                floor_x += floor_step_x;
                floor_y += floor_step_y;

                let i_ceiling = (16 * 4 * ty_ceiling + tx_ceiling * 4) as usize;
                let i_floor = (32 * 4 * ty_floor + tx_floor * 4) as usize;

                // FLOOR
                color.copy_from_slice(&MOSSY_STONE[i_floor..i_floor + 4]);
                let index =
                    (draw_floor_y_offset + (self.width - x - 1) * 4) as usize;
                data[index..index + 4].copy_from_slice(&color);

                // CEILING
                color.copy_from_slice(&LIGHT_PLANK[i_ceiling..i_ceiling + 4]);
                let index =
                    (draw_ceiling_y_offset + (self.width - x - 1) * 4) as usize;
                data[index..index + 4].copy_from_slice(&color);
            }
        }
    }
}
