extern crate geotoy;
#[macro_use]
extern crate glium;

use geotoy::{Attractor, Kind, Point};
use glium::index::PrimitiveType;
use glium::{
    glutin::{self, VirtualKeyCode}, Surface,
};

fn draw(
    display: &glium::Display,
    program: &glium::program::Program,
    a: f32,
    b: f32,
    draw_grid: bool,
    points_vb: &glium::VertexBuffer<Point>,
    attractors_vb: &glium::VertexBuffer<Attractor>,
    kinds_vb: &glium::VertexBuffer<Kind>,
    index_buffer: &glium::IndexBuffer<u32>,
) -> Result<(), glium::SwapBuffersError> {
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);

    if draw_grid {
        target
            .draw(
                (points_vb, attractors_vb, kinds_vb),
                index_buffer,
                program,
                &uniform! {
                    a: 0.0f32,
                    b: 0.0f32,
                    color: [0.3, 0.3, 0.3f32],
                },
                &Default::default(),
            )
            .unwrap();
    }

    target
        .draw(
            (points_vb, attractors_vb, kinds_vb),
            index_buffer,
            program,
            &uniform! {
                a: a,
                b: b,
                color: [1.0, 1.0, 1.0f32],
            },
            &Default::default(),
        )
        .unwrap();

    target.finish()
}

fn main() -> Result<(), Box<std::error::Error>> {
    let mut width = 2048;
    let mut height = 2048;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions(width, height);
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop)?;

    let rows = 5;
    let cols = 5;

    let size = (1.0 - -1.0) / ((cols - 1) as f32 * 1.5);

    let (points, lines, attractors, kinds) = geotoy::mesh(rows, cols, size);

    let points_vb = glium::VertexBuffer::new(&display, &points)?;
    let attractors_vb = glium::VertexBuffer::new(&display, &attractors)?;
    let kinds_vb = glium::VertexBuffer::new(&display, &kinds)?;
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::LinesList, &lines)?;

    let program = program!(&display,
        140 => {
            vertex: geotoy::VERTEX_SHADER,
            fragment: geotoy::FRAGMENT_SHADER,
        },
    )?;

    let mut a: f32 = 0.1;
    let mut b: f32 = 0.6;
    let mut draw_grid = true;

    draw(
        &display,
        &program,
        a,
        b,
        draw_grid,
        &points_vb,
        &attractors_vb,
        &kinds_vb,
        &index_buffer,
    )?;

    loop {
        let mut should_quit = false;
        let mut need_draw = false;
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::KeyboardInput { input, .. }
                    if input.state == glutin::ElementState::Pressed =>
                {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::Escape => should_quit = true,
                            VirtualKeyCode::G => {
                                draw_grid = !draw_grid;
                                need_draw = true;
                            }
                            _ => {}
                        }
                    }
                }
                glutin::WindowEvent::Closed => should_quit = true,
                glutin::WindowEvent::Resized(w, h) => {
                    width = w;
                    height = h;
                    need_draw = true;
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    a = ((position.0 as f32) / (width as f32) - 0.5) * 10.0;
                    b = ((position.1 as f32) / (height as f32) - 0.5) * 10.0;
                    need_draw = true;
                }
                _ => (),
            },
            _ => (),
        });

        if should_quit {
            return Ok(());
        }

        if need_draw {
            draw(
                &display,
                &program,
                a,
                b,
                draw_grid,
                &points_vb,
                &attractors_vb,
                &kinds_vb,
                &index_buffer,
            ).unwrap()
        }
    }
}
