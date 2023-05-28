use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
    window::{WindowBuilder, WindowId},
};

fn event_handler(
    event: Event<()>,
    _event_loop_window_target: &EventLoopWindowTarget<()>,
    control_flow: &mut ControlFlow,
    target_window_id: WindowId,
) {
    // Discard events that don't belong to the window that we want to close.
    if let Event::WindowEvent { window_id, .. } = &event {
        if *window_id != target_window_id {
            return;
        }
    }

    match event {
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    }
}

pub fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    event_loop.run(move |event, event_loop_window_target, control_flow| {
        event_handler(event, event_loop_window_target, control_flow, window.id())
    });
}
