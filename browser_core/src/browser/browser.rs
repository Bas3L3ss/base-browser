use std::println;
use std::{ eprintln, sync::Arc };

use anyhow::Result;
use url::Url;
use window::State;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::{ Window, WindowId };
use winit::{ event::{ WindowEvent }, event_loop::{ ControlFlow, EventLoop } };

use crate::{ browser::BrowserShared, tab::tab::Tab };

pub struct Browser<'a> {
    state: Option<State<'a>>,
    pub shared: BrowserShared,
    pub tabs: Vec<Tab<'a>>,
}

impl<'a> Browser<'a> {
    pub fn new() -> Self {
        Self {
            shared: BrowserShared::new().unwrap_or_else(|e| {
                eprintln!("Failed to initialize BrowserShared: {}", e);
                std::process::exit(1);
            }),
            tabs: Vec::new(),
            state: None,
        }
    }

    pub fn open_tab(&mut self, url: &'a Url) -> Result<()> {
        let mut tab = Tab::new();
        tab.navigate(&url, &mut self.shared)?;
        self.tabs.push(tab);
        Ok(())
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap_or_else(|e: winit::error::EventLoopError| {
            eprintln!("Failed to initialize winit event loop: {}", e);
            std::process::exit(1);
        });
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(self).unwrap();
    }
}

impl<'a> ApplicationHandler for Browser<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Broase");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let state = pollster::block_on(State::new(window));

        self.state = Some(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved { device_id: _, position } => {
                if let Some(state) = self.state.as_mut() {
                    state.cursor_position = position;
                }
            }

            // WindowEvent::KeyboardInput { event, .. } => {
            //     if let Some(state) = self.state.as_mut() {
            //         if let Some(ref mut world) = state.world {
            //             if let PhysicalKey::Code(keycode) = event.physical_key {
            //                 match keycode {
            //                     KeyCode::KeyW => {
            //                         match event.state {
            //                             winit::event::ElementState::Pressed => {
            //                                 world.keys.insert(glfw::Key::W, true);
            //                             }
            //                             winit::event::ElementState::Released => {
            //                                 world.keys.insert(glfw::Key::W, false);
            //                             }
            //                         }
            //                     }
            //                     KeyCode::KeyA => {
            //                         match event.state {
            //                             winit::event::ElementState::Pressed => {
            //                                 world.keys.insert(glfw::Key::A, true);
            //                             }
            //                             winit::event::ElementState::Released => {
            //                                 world.keys.insert(glfw::Key::A, false);
            //                             }
            //                         }
            //                     }
            //                     KeyCode::KeyS => {
            //                         match event.state {
            //                             winit::event::ElementState::Pressed => {
            //                                 world.keys.insert(glfw::Key::S, true);
            //                             }
            //                             winit::event::ElementState::Released => {
            //                                 world.keys.insert(glfw::Key::S, false);
            //                             }
            //                         }
            //                     }
            //                     KeyCode::KeyD => {
            //                         match event.state {
            //                             winit::event::ElementState::Pressed => {
            //                                 world.keys.insert(glfw::Key::D, true);
            //                             }
            //                             winit::event::ElementState::Released => {
            //                                 world.keys.insert(glfw::Key::D, false);
            //                             }
            //                         }
            //                     }

            //                     _ => {}
            //                 }
            //             }
            //         }
            //     }
            // }

            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = self.state.as_mut() {
                    state.render();
                    state.window.request_redraw();
                }
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize(physical_size);
                }
            }
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::panic;
    use std::assert_eq;

    use super::*;
    use input_parser::Parse;
    use std::sync::mpsc::channel;
    use std::thread;
    use url::Url;

    // Utility function to enforce a 3 second timeout on each test
    fn run_with_timeout<F>(f: F) where F: FnOnce() + std::panic::UnwindSafe + Send + 'static {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let _ = tx.send(std::panic::catch_unwind(f));
        });

        match rx.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(Ok(_)) => (), // Test finished successfully
            Ok(Err(e)) => std::panic::resume_unwind(e), // Test panicked, propagate
            Err(_) => panic!("Test timed out (took longer than 3 seconds)"),
        }
    }

    #[test]
    fn browser_can_create_and_open_tabs() {
        let url = Url::parse("https://example.com").unwrap().value;
        let mut browser = Browser::new();

        browser.open_tab(&url).unwrap();

        assert_eq!(browser.tabs.len(), 1);
        let r = &browser.tabs[0];
        let doc = r.document.as_ref().unwrap();
        assert_eq!(*doc.url, url);
    }
}
