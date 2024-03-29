use render::DrawingContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod render;
pub mod utils;

fn event_handler(
    event: &Event<()>,
    _event_loop_window_target: &EventLoopWindowTarget<()>,
    control_flow: &mut ControlFlow,
    drawing_context: &mut DrawingContext,
) {
    // Discard events that don't belong to the window that we want to close.
    if let Event::WindowEvent { window_id, .. } = &event {
        if *window_id != drawing_context.window().id() {
            return;
        }
    }

    if let Event::WindowEvent { event, .. } = &event {
        let window_event = event;
        match window_event {
            // When the window is resized
            WindowEvent::Resized(physical_size) => {
                drawing_context.resize(*physical_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                drawing_context.resize(**new_inner_size);
            }
            // When the window is closed
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::CursorMoved { position, .. } => {
                #[cfg(feature = "debug_input")]
                log::trace!("Mouse moved to {:?}", position);

                let _ = drawing_context.set_cursor_middle();
                drawing_context
                    .input_manager
                    .update_mouse_position(position);
            }
            WindowEvent::MouseInput { state, button, .. } => drawing_context
                .input_manager
                .update_mouse_button(button, state),
            WindowEvent::KeyboardInput { input, .. } => {
                drawing_context.input_manager.update_key(input);
            }
            _ => {}
        }
        return;
    }

    match event {
        #[rustfmt::skip]
        #[cfg(target_arch = "wasm32")]
        Event::DeviceEvent { event: winit::event::DeviceEvent::MouseMotion { delta }, ..  } => {
            drawing_context.input_manager.update_mouse_delta(&delta);
        }
        Event::RedrawRequested(window_id) if *window_id == drawing_context.window().id() => {
            drawing_context.process_inputs();
            match drawing_context.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => drawing_context.reconfigure(),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => log::error!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            drawing_context.window().request_redraw();
        }
        _ => {}
    }
}

#[cfg(target_arch = "wasm32")]
fn init_log() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
}

#[cfg(target_arch = "wasm32")]
fn init_window(window: &winit::window::Window) {
    // Winit prevents sizing with CSS, so we have to set
    // the size manually when on web.
    use winit::dpi::PhysicalSize;
    window.set_inner_size(PhysicalSize::new(600, 400));

    use winit::platform::web::WindowExtWebSys;
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let dst = doc.get_element_by_id("body")?;
            let canvas = web_sys::Element::from(window.canvas());
            dst.append_child(&canvas).ok()?;
            Some(())
        })
        .expect("Couldn't append canvas to document body.");
}

/// Run the application
///
/// # Panics
///
/// - If the application fails to initialize.
/// - If the application fails to load the scene.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    #[cfg(target_arch = "wasm32")]
    init_log();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    init_window(&window);

    let mut drawing_context = DrawingContext::new(window).await;

    event_loop.run(move |event, event_loop_window_target, control_flow| {
        event_handler(
            &event,
            event_loop_window_target,
            control_flow,
            &mut drawing_context,
        );
    });
}
