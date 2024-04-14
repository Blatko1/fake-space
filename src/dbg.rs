use crate::backend::gfx::Gfx;
use crate::backend::Canvas;
use crate::player::PlayerDebugData;
use crate::world::WorldDebugData;
use wgpu::RenderPass;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use wgpu_text::glyph_brush::{
    BuiltInLineBreaker, Extra, Layout, OwnedSection, Section, Text, VerticalAlign,
};
use wgpu_text::{BrushBuilder, BrushError, TextBrush};

pub struct DebugData {
    pub current_fps: i32,
    pub avg_fps_time: f64,

    pub player_data: PlayerDebugData,
    pub world_data: WorldDebugData,
}

pub struct Dbg {
    screen_position: (f32, f32),
    brush: TextBrush<FontVec>,
    content: OwnedSection<Extra>,
}

impl Dbg {
    pub fn new(gfx: &Gfx, font: FontVec) -> Self {
        let config = gfx.config();
        let brush = BrushBuilder::using_font(font).build(
            gfx.device(),
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
        }
    }

    pub fn update(&mut self, data: DebugData) {
        let player = data.player_data;
        let data_str = format!(
            "FPS: {}\n\
            Average frame time: {:.2} ms\n\
            Position: x: {:.2}, y: {:.2}, z: {:.2}\n\
            Direction: Vec3({:.2}, {:.2}, {:.2})\n\
            Angle: {:.2} degrees\n\
            Y-shearing: {}\n\
            Room: {} of {} total\n\
            Fly: {}, Ghost: {}\n\
            On ground: {}\n\
            Velocity: x: {:.2}, z: {:.2}\n\
            Air velocity: {:.2}",
            data.current_fps,
            data.avg_fps_time,
            player.eye_pos.x,
            player.eye_pos.y,
            player.eye_pos.z,
            player.forward_dir.x,
            player.forward_dir.y,
            player.forward_dir.z,
            player.yaw_angle,
            player.y_shearing,
            player.current_room_id,
            data.world_data.room_count - 1,
            player.physics_state.can_fly,
            player.physics_state.is_ghost,
            player.physics_state.is_grounded,
            player.physics_state.movement_velocity.x,
            player.physics_state.movement_velocity.y,
            player.physics_state.air_velocity
        );
        self.content = Section::default()
            .with_text(vec![Text::new(&data_str)
                .with_scale(35.0)
                .with_color([1.0, 1.0, 0.9, 1.0])])
            .with_screen_position(self.screen_position)
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker),
            )
            .to_owned();
    }

    pub fn resize(&mut self, canvas: &Canvas) {
        let region = canvas.region();
        let gfx = canvas.gfx();
        let config = gfx.config();
        self.screen_position = (region.x as f32 + 5.0, region.y as f32 + 5.0);
        self.brush
            .resize_view(config.width as f32, config.height as f32, gfx.queue());
    }

    pub fn queue_data(&mut self, gfx: &Gfx) -> Result<(), BrushError> {
        self.brush
            .queue(gfx.device(), gfx.queue(), vec![&self.content])
    }

    pub fn render<'pass>(&'pass self, rpass: &mut RenderPass<'pass>) {
        self.brush.draw(rpass)
    }
}
