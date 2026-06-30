#[cfg_attr(feature = "no_std", no_std)]
#[cfg(feature = "no_std")]
pub use heapless;
use num::Num;
pub use pinions_macros;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Instant;
pub use winit;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[cfg(feature = "no_std")]
type Str<const N: usize> = heapless::String<N>;

#[cfg(not(feature = "no_std"))]
type Str<const N: usize> = String;

#[cfg(feature = "no_std")]
type Vect<T, const N: usize> = heapless::Vec<T, N>;

#[cfg(not(feature = "no_std"))]
type Vect<T, const N: usize = 0> = Vec<T>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Point {
    x: f32,
    y: f32,
}

struct Mouse {
    position: Option<Point>,
    pressed: bool,
}

struct Event {
    timestamp: Instant,
    event: isize,
}

impl Clone for Event {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Event {}

impl Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Event")
            .field("timestamp", &self.timestamp)
            .field("event", &self.event)
            .finish()
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            timestamp: Instant::now(),
            event: 0,
        }
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.eq(&other.timestamp)
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.timestamp.cmp(&other.timestamp))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

pub struct Wid<const L: usize, I: Num, const S: usize, const E: usize> {
    pub label: Str<L>,
    pub icon: Vect<I, S>,
    pub mouse: Mouse,
    pub events: Arc<Mutex<Vect<Event, E>>>,
}

impl<const L: usize, I: Num, const S: usize, const E: usize> Wid<L, I, S, E> {
    pub fn new() -> Self {
        let mut lbl = Str::<L>::new();
        let icon = Vect::<I, S>::new();
        Self {
            label: lbl,
            icon,
            mouse: Mouse {
                position: None,
                pressed: false,
            },
            events: Arc::new(Mutex::new(Vect::<Event, E>::new())),
        }
    }

    pub fn sort_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.as_mut_slice().sort();
    }
}

pub struct Win<
    const T: usize, // title length
    const E: usize, // event length
    const V: usize, // widget count
    const L: usize, // label length -\
    I: Num,         // icon type      | <- for struct Wid
    const S: usize, // icon size    -/
> {
    window: Option<Window>,
    title: Str<T>,
    poll: bool,
    events: Arc<Mutex<Vect<Event, E>>>,
    widgets: Vect<Wid<L, I, S, E>, V>,
}

impl<const T: usize, const E: usize, const V: usize, const L: usize, I: Num, const S: usize>
    Win<T, E, V, L, I, S>
{
    pub fn new() -> Self {
        Self {
            window: None,
            title: Str::<T>::new(),
            poll: false,
            events: Arc::new(Mutex::new(Vect::<Event, E>::new())),
            widgets: Vect::<Wid<L, I, S, E>, V>::new(),
        }
    }

    pub fn default() -> Self {
        Self::new()
    }
    fn title(&mut self, title: Str<T>) {
        self.title = title;
    }
}

impl<const T: usize, const E: usize, const V: usize, const L: usize, I: Num, const S: usize>
    winit::application::ApplicationHandler for Win<T, E, V, L, I, S>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes =
                winit::window::Window::default_attributes().with_title(self.title.as_str());
            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    self.window = Some(window);
                }
                Err(err) => {
                    eprintln!("Failed to create window: {err}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let point = Point {
                    x: position.x as f32,
                    y: position.y as f32,
                };
                for widget in self.widgets.iter_mut() {
                    let point_for_widget = point.clone();
                    widget.mouse.position = Some(point_for_widget);
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                if button == winit::event::MouseButton::Left {
                    let pressed = matches!(state, winit::event::ElementState::Pressed);
                    for widget in self.widgets.iter_mut() {
                        widget.mouse.pressed = pressed;
                    }
                }
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // Poll for events if needed
        if self.poll {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

impl<const T: usize, const E: usize, const V: usize, const L: usize, I: Num, const S: usize>
    Win<T, E, V, L, I, S>
{
    /// Start the event loop and run the application.
    /// This method will block until the event loop exits.
    pub fn run(mut self) -> Result<(), winit::error::EventLoopError> {
        let event_loop = winit::event_loop::EventLoop::new()?;
        event_loop.set_control_flow(if self.poll {
            winit::event_loop::ControlFlow::Poll
        } else {
            if !self.events.lock().unwrap().is_empty() {
                winit::event_loop::ControlFlow::Wait
            } else {
                let instant = self.events.lock().unwrap()[0].timestamp.clone();
                winit::event_loop::ControlFlow::WaitUntil(instant)
            }
        });
        event_loop.run_app(&mut self)
    }

    pub fn sort_events(&self) {
        let mut events = self.events.lock().unwrap();
        events.as_mut_slice().sort();
    }
}

/// Sets the poll flag to true, which will cause the window to request redraws
/// in the event loop's `about_to_wait` callback.
pub fn set_poll<
    const T: usize,
    const E: usize,
    const V: usize,
    const L: usize,
    I: Num,
    const S: usize,
>(
    win: &mut Win<T, E, V, L, I, S>,
) {
    win.poll = true;
}

/// Unsets the poll flag, which will stop requesting redraws in the event loop's
/// `about_to_wait` callback.
pub fn unset_poll<
    const T: usize,
    const E: usize,
    const V: usize,
    const L: usize,
    I: Num,
    const S: usize,
>(
    win: &mut Win<T, E, V, L, I, S>,
) {
    win.poll = false;
}
