use std::time::Instant;

use crate::backend::ctx::Ctx;
use crate::backend::Canvas;
use crate::player::PlayerDebugData;
use wgpu::RenderPass;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use wgpu_text::glyph_brush::{
    BuiltInLineBreaker, Extra, Layout, OwnedSection, Section, Text, VerticalAlign,
};
use wgpu_text::{BrushBuilder, BrushError, TextBrush};

use super::ScissorRegion;

pub struct DebugData {
    pub player_data: PlayerDebugData,
    //pub world_data: WorldDebugData,
}

pub struct DebugUI {
    screen_position: (f32, f32),
    brush: TextBrush<FontVec>,
    content: OwnedSection<Extra>,

    // FPS counting
    fps_print_delta: Instant,
    frame_count: u32,
    current_fps: u32,
}

impl DebugUI {
    pub fn new(ctx: &Ctx, font: FontVec) -> Self {
        let config = ctx.config();
        let brush = BrushBuilder::using_font(font).build(
            ctx.device(),
            config.width,
            config.height,
            config.format,
        );
        let screen_position = (config.width as f32 * 0.05, config.height as f32 * 0.05);
        let content = Section::default().add_text(Text::new("EMPTY")).to_owned();

        Self {
            screen_position,
            brush,
            content,

            fps_print_delta: Instant::now(),
            frame_count: 0,
            current_fps: 0,
        }
    }

    pub fn update(&mut self, data: DebugData) {
        let player = data.player_data;
        let time_per_frame = 1000.0 / self.current_fps as f64;
        let data_str = format!(
            "FPS: {}\n\
            Average frame time: {:.2} ms\n\
            Position: x: {:.2}, y: {:.2}, z: {:.2}\n\
            Direction: Vec3({:.2}, {:.2}, {:.2})\n\
            Angle: {:.2} degrees\n\
            Y-shearing: {}\n\
            Room: {} of {}\n\n\
            Fly: {}, Ghost: {}\n\
            On ground: {}\n\
            Velocity: x: {:.2}, z: {:.2}\n\
            Air velocity: {:.2}",
            self.current_fps,
            time_per_frame,
            player.eye_pos.x,
            player.eye_pos.y,
            player.eye_pos.z,
            player.forward_dir.x,
            player.forward_dir.y,
            player.forward_dir.z,
            player.yaw_angle,
            player.y_shearing,
            player.current_room_id,
            0, //data.world_data.room_count - 1,
            player.physics_state.can_fly,
            player.physics_state.is_ghost,
            player.physics_state.is_grounded,
            player.physics_state.movement_velocity.x,
            player.physics_state.movement_velocity.y,
            player.physics_state.air_velocity
        );
        self.content = Section::default()
            .with_text(vec![
                Text::new(&data_str)
                    .with_scale(35.0)
                    .with_color([1.0, 1.0, 0.9, 1.0]),
                Text::new(&format!("\nScore: {}", player.score))
                    .with_scale(60.0)
                    .with_color([0.81, 0.3, 0.2, 1.0]),
            ])
            .with_screen_position(self.screen_position)
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
            )
            .to_owned();
    }

    pub fn update_frame_timings(&mut self) {
        self.frame_count += 1;
        if self.fps_print_delta.elapsed().as_micros() >= 1000000 {
            self.fps_print_delta = Instant::now();
            self.current_fps = self.frame_count;
            self.frame_count = 0;
        }
    }

    pub fn resize(&mut self, region: ScissorRegion, ctx: &Ctx) {
        let queue = ctx.queue();
        let config = ctx.config();
        self.screen_position = (region.x as f32 + 5.0, region.y as f32 + 5.0);
        self.brush
            .resize_view(config.width as f32, config.height as f32, queue);
    }

    pub fn queue_data(&mut self, ctx: &Ctx) -> Result<(), BrushError> {
        self.brush
            .queue(ctx.device(), ctx.queue(), vec![&self.content])
    }

    pub fn render<'pass>(&'pass self, rpass: &mut RenderPass<'pass>) {
        self.brush.draw(rpass)
    }
}
