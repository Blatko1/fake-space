use futures::executor::block_on;
use utils::GFXUtil;
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod utils;



fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "error");
    }
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    window.set_title("False 3D Game");

    let gfx = block_on(GFXUtil::init(window)).unwrap();

    
}
