use pincers::pincers_macros;
use pincers_macros::Witaj;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct App {
    window: Option<Window>,
    poll: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = Window::default_attributes()
                .with_title("My first Winit window")
                .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0));

            let window = event_loop.create_window(window_attributes).unwrap();
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if self.poll {
                    event_loop.set_control_flow(ControlFlow::Wait);
                    self.poll = false;
                } else {
                    event_loop.set_control_flow(ControlFlow::Poll);
                    self.poll = true;
                }
            }
            _ => (),
        }
    }
}

fn main() {
    #[derive(Witaj)]
    struct MyStruct {
        name: String,
    }

    let my_struct = MyStruct {
        name: "John".to_string(),
    };
    my_struct.say_hello();

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap();
}
