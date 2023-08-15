use crate::textures::{BLUE_BRICK, LIGHT_PLANK};

use super::Raycaster;

const GRAY: [u8; 4] = [100, 100, 100, 255];

pub fn fill(rc: &Raycaster, data: &mut [u8]) {
    //let index = (rc.height / 2 * 4 * rc.width) as usize;
    //data[index..]
    //    .chunks_exact_mut(4)
    //    .for_each(|rgba| rgba.copy_from_slice(&GRAY));
    //rc.column_buffer[0..index].chunks_exact_mut(4).for_each(|rgba| rgba.copy_from_slice(&RED));
    //for x in 0..rc.width as usize {
    //    let begin_y = 0;
    //    let end_y = rc.height as usize / 2;
    //    canvas.draw_line(x, begin_y, end_y, &rc.column_buffer[0..index]);
    //}

    let ray_dir_x0 = rc.dir.x - rc.plane.x;
    let ray_dir_y0 = rc.dir.y - rc.plane.y;
    let ray_dir_x1 = rc.dir.x + rc.plane.x;
    let ray_dir_y1 = rc.dir.y + rc.plane.y;
    let pos_z = 0.5 * rc.height as f32;

    let mut color = [0; 4];

    for y in rc.height / 2..rc.height {
        let p = y as f32 - rc.height as f32 / 2.0;

        let row_dist = pos_z / p;

        let floor_step_x =
            row_dist * (ray_dir_x1 - ray_dir_x0) / rc.width as f32;
        let floor_step_y =
            row_dist * (ray_dir_y1 - ray_dir_y0) / rc.width as f32;

        let mut floor_x = rc.pos.x + row_dist * ray_dir_x0;
        let mut floor_y = rc.pos.y + row_dist * ray_dir_y0;

        for x in 0..rc.width {
            let cellx = floor_x as i32;
            let celly = floor_y as i32;

            let tx = (16.0 * (floor_x - cellx as f32)) as u32 & (16 - 1);
            let ty = (16.0 * (floor_y - celly as f32)) as u32 & (16 - 1);

            floor_x += floor_step_x;
            floor_y += floor_step_y;

            let i = (16 * 4 * ty + tx * 4) as usize;

            color.copy_from_slice(&LIGHT_PLANK[i..i + 4]);
            let index = (y * 4 * rc.width + (rc.width - x - 1) * 4) as usize;
            data[index..index + 4].copy_from_slice(&color);

            color.copy_from_slice(&BLUE_BRICK[i..i + 4]);
            let index = ((rc.height - y - 1) * 4 * rc.width
                + (rc.width - x - 1) * 4) as usize;
            data[index..index + 4].copy_from_slice(&color);
        }
    }
}
