use super::{RayCast, Raycaster, Side};
use crate::{
    canvas::Canvas,
    map::{Map, Tile, TransparentTexture, WallTexture},
    textures::{
        BLUE_BRICK, BLUE_BRICK_HEIGHT, BLUE_BRICK_WIDTH, FENCE, FENCE_HEIGHT,
        FENCE_WIDTH, LIGHT_PLANK, LIGHT_PLANK_HEIGHT, LIGHT_PLANK_WIDTH,
    },
};

pub fn draw(rc: &Raycaster, ray: &RayCast, data: &mut [u8]) {
    let draw_x_offset = 4 * (rc.width - ray.screen_x - 1) as usize;
    let mut color = [0, 0, 0, 0];
    let mut color_2 = [0, 0, 0, 0];
    let half_h_i = rc.height as i32 / 2;
    let half_h_f = rc.height as f32 * 0.5;
    let hit = ray.through_hit.unwrap();
    let tile = hit[0].tile;
    let hit1 = hit[0];
    let hit2 = hit[1];
    let (texture, tex_width, tex_height) = match tile {
        Tile::Transparent(tex) => match tex {
            TransparentTexture::Fence => (FENCE, FENCE_WIDTH, FENCE_HEIGHT),
        },
        _ => unreachable!(),
    };
    //let wall_x = (hit.wall_x + 0.45/f32::atan2(ray.dir.y, ray.dir.x).tan()).clamp(0.0, 1.0 - f32::EPSILON);
    //let offset_dist = 0.45 / f32::atan2(ray.dir.y, ray.dir.x).sin();
    let line_pixel_height_hit1 = (rc.height as f32 / (hit1.wall_dist)) as i32;
    let half_l_hit1 = line_pixel_height_hit1 / 2;

    let line_pixel_height_hit2 = (rc.height as f32 / (hit2.wall_dist)) as i32;
    let half_l_hit2 = line_pixel_height_hit2 / 2;

    let begin_hit1 = (half_h_i - half_l_hit1).max(0) as u32;
    let end_hit1 = ((half_h_i + half_l_hit1) as u32).min(rc.height - 1);

    let begin_hit2 = (half_h_i - half_l_hit2).max(0) as u32;
    let end_hit2 = ((half_h_i + half_l_hit2) as u32).min(rc.height - 1);

    let tex_height_minus_one = tex_height as f32 - 1.0;
    let tex_x_hit1 = match hit1.side {
        Side::Vertical if ray.dir.x > 0.0 => {
            tex_width - (hit1.wall_x * tex_width as f32) as u32 - 1
        }

        Side::Horizontal if ray.dir.y < 0.0 => {
            tex_width - (hit1.wall_x * tex_width as f32) as u32 - 1
        }
        _ => (hit1.wall_x * tex_width as f32) as u32,
    };
    let tex_x_hit2 = match hit2.side {
        Side::Vertical if ray.dir.x > 0.0 => {
            tex_width - (hit2.wall_x * tex_width as f32) as u32 - 1
        }

        Side::Horizontal if ray.dir.y < 0.0 => {
            tex_width - (hit2.wall_x * tex_width as f32) as u32 - 1
        }
        _ => (hit2.wall_x * tex_width as f32) as u32,
    };
    let four_tex_x_hit1 = tex_x_hit1 * 4;
    let four_tex_x_hit2 = tex_x_hit2 * 4;
    //assert!(tex_x < 16);
    let tex_y_step_hit1 = tex_height as f32 / line_pixel_height_hit1 as f32;
    let mut tex_y_hit1 =
        (begin_hit1 as f32 + line_pixel_height_hit1 as f32 * 0.5 - half_h_f)
            * tex_y_step_hit1;
    let tex_y_step_hit2 = tex_height as f32 / line_pixel_height_hit2 as f32;
    let mut tex_y_hit2 =
        (begin_hit2 as f32 + line_pixel_height_hit2 as f32 * 0.5 - half_h_f)
            * tex_y_step_hit2;
    // TODO fix texture mapping.
    //assert!(tex_y >= 0.0);
    for y in begin_hit2..end_hit2 {
        //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
        let y_pos_hit2 = tex_y_hit2.min(tex_height_minus_one).round() as u32;

        let i = ((tex_height - y_pos_hit2 - 1) * tex_width * 4
            + four_tex_x_hit2) as usize;
        color.copy_from_slice(&texture[i..i + 4]);
        match hit1.side {
            Side::Vertical => (),
            Side::Horizontal => {
                color[0] = color[0].saturating_sub(15);
                color[1] = color[1].saturating_sub(15);
                color[2] = color[2].saturating_sub(15);
            }
        };
        let index = (rc.height as usize - 1 - y as usize) * rc.four_width
        + draw_x_offset;
        tex_y_hit2 += tex_y_step_hit2;
        if color[3] == 0 {
            continue;
        }
        data[index..index + 4].copy_from_slice(&color);
        //assert!(tex_y <= 16.0);
    }

    for y in begin_hit1..end_hit1 {
        //assert!(tex_y <= 15.0, "Not less!: y0: {}, y1: {}, y: {}", y0, y1, y);
        let y_pos_hit1 = tex_y_hit1.min(tex_height_minus_one).round() as u32;

        let i = ((tex_height - y_pos_hit1 - 1) * tex_width * 4
            + four_tex_x_hit1) as usize;
        color.copy_from_slice(&texture[i..i + 4]);
        match hit1.side {
            Side::Vertical => (),
            Side::Horizontal => {
                color[0] = color[0].saturating_sub(15);
                color[1] = color[1].saturating_sub(15);
                color[2] = color[2].saturating_sub(15);
            }
        };
        let index = (rc.height as usize - 1 - y as usize) * rc.four_width
            + draw_x_offset;
        tex_y_hit1 += tex_y_step_hit1;
        if color[3] == 0 {
            continue;
        }
        data[index..index + 4].copy_from_slice(&color);
        //assert!(tex_y <= 16.0);
    }
}
