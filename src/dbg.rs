use glam::Vec3;
use crate::backend::gfx::Gfx;
use wgpu::RenderPass;
use wgpu_text::glyph_brush::ab_glyph::FontVec;
use wgpu_text::glyph_brush::{BuiltInLineBreaker, Extra, Layout, OwnedSection, Section, Text, VerticalAlign};
use wgpu_text::{BrushBuilder, BrushError, TextBrush};
use crate::backend::Canvas;
use crate::state::State;

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

    pub fn update(&mut self, state: &State, avg_fps_time: f32, fps: i32) {
        let world = state.get_world();
        let player = state.get_player();
        let camera = player.get_camera();
        let position = camera.get_origin();
        let direction = camera.get_direction();
        let angle = camera.get_yaw_angle().to_degrees();

        self.content = Section::default()
            .with_text(vec![
                Text::new(&format!(
                    "FPS: {}\n\
                    Average frame time: {:.2} ms\n\
                    Position: x: {:.3}, y: {:.3}, z: {:.3}\n\
                    Direction: Vec3[{:.3}, {:.3}, {:.3}]\n\
                    Angle: {:.2} degrees\n\
                    RoomID: {}\n\
                    Room count: {}",
                    fps, avg_fps_time,
                    position.x, position.y, position.z,
                    direction.x, direction.y, direction.z,
                    angle, player.get_current_room_id().0, world.room_count()
                )).with_scale(30.0).with_color([0.9, 0.2, 0.3, 1.0]),
            ])
            .with_screen_position(self.screen_position)
            .with_layout(Layout::default()
                .v_align(VerticalAlign::Top)
                .line_breaker(BuiltInLineBreaker::AnyCharLineBreaker))
            .to_owned();
    }

    pub fn resize(&mut self, canvas: &Canvas) {
        let region = canvas.region();
        let gfx = canvas.gfx();
        let config = gfx.config();
        self.screen_position = (region.x as f32 + 5.0, region.y as f32 + 5.0);
        self.brush.resize_view(config.width as f32, config.height as f32, gfx.queue());
    }

    pub fn queue_data(&mut self, gfx: &Gfx) -> Result<(), BrushError> {
        self.brush
            .queue(gfx.device(), gfx.queue(), vec![&self.content])
    }

    pub fn render<'pass>(&'pass self, rpass: &mut RenderPass<'pass>) {
        self.brush.draw(rpass)
    }
}
