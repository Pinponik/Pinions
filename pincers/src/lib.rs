pub use pincers_macros;
pub use winit;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub struct App {
    window: Option<Window>,
    title: String,
    poll: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            window: None,
            title: String::new(),
            poll: false,
        }
    }

    pub fn default() -> Self {
        Self::new()
    }
}
