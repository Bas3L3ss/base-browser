use std::println;
use std::{ eprintln, sync::Arc };

use anyhow::{ Result, bail };
use input_parser::Parse;
use url::Url;
use window::{ State, Tab, TabId };
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;
use winit::window::{ Window, WindowId };
use winit::{ event::{ WindowEvent }, event_loop::{ ControlFlow, EventLoop } };

use crate::{ browser::BrowserShared };

pub struct Browser<'a> {
    pub state: Option<State<'a>>,
    pub shared: BrowserShared,
    pub tab_id: TabId,
}

impl<'a> Browser<'a> {
    pub fn new() -> Self {
        Self {
            tab_id: 0,
            shared: BrowserShared::new().unwrap_or_else(|e| {
                eprintln!("Failed to initialize BrowserShared: {}", e);
                std::process::exit(1);
            }),
            state: None,
        }
    }

    pub fn open_tab(&mut self, url_str: &str) -> Result<()> {
        let url = match Url::parse(&url_str) {
            Ok(parsed_out) => {
                for warning in &parsed_out.warnings {
                    url::print_warning::print_warning(warning);
                }
                parsed_out.value
            }
            Err(err) => {
                eprintln!("Error parsing URL: {err}");
                return Err(err.into());
            }
        };

        if let Some(state) = self.state.as_mut() {
            let mut tab = Tab::new(self.tab_id);
            self.tab_id += 1;
            tab.navigate(
                &url,
                &self.shared.ua_stylesheet,
                &self.shared.resource_loader,
                &mut self.shared.font_manager,
                &state.window
            )?;
            state.cur_tab = Some(tab.tab_id);
            state.tabs.push(tab);
            return Ok(());
        } else {
            bail!("No state found");
        }
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

        self.open_tab(&"http://127.0.0.1:5500/simple.html").unwrap();
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
                    state.render(&self.shared.font_manager);
                }
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(state) = self.state.as_mut() {
                    state.resize_physical(physical_size);
                }

                let size = if let Some(state) = self.state.as_ref() {
                    state.window.inner_size()
                } else {
                    return;
                };
                if let Some(state) = self.state.as_mut() {
                    if let Some(tab) = state.get_current_mut_tab() {
                        let _ = tab.rerender(
                            &self.shared.resource_loader,
                            &mut self.shared.font_manager,
                            size
                        );
                    }
                }
            }
            _ => (),
        }
    }
}
