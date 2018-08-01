extern crate geotoy;
#[macro_use]
extern crate glium;

use geotoy::{Attractor, Kind, Point};
use glium::index::PrimitiveType;
use glium::{
    Blend,
    BlendingFunction,
    DrawParameters,
    LinearBlendingFactor,
    glutin::{self, VirtualKeyCode}, Surface,
};

struct DrawContext<'a> {
    display: &'a glium::Display,
    lines_program: &'a glium::program::Program,
    triangles_program: &'a glium::program::Program,
    a: f32,
    b: f32,
    offset: (f32, f32),
    draw_grid: bool,
    draw_triangles: bool,
    draw_lines: bool,
    points_vb: &'a glium::VertexBuffer<Point>,
    attractors_vb: &'a glium::VertexBuffer<Attractor>,
    kinds_vb: &'a glium::VertexBuffer<Kind>,
    lines_ib: &'a glium::IndexBuffer<u16>,
    triangles_ib: &'a glium::IndexBuffer<u16>,
}

enum CursorPositionFn {
    Offset,
    HexParams,
}

fn draw(ctx: &DrawContext) -> Result<(), glium::SwapBuffersError> {
    let mut target = ctx.display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);

    let params =
        &DrawParameters {
            blend: Blend {
                color: BlendingFunction::Addition {
                    source: LinearBlendingFactor::SourceColor,
                    destination: LinearBlendingFactor::OneMinusSourceColor,
                },
                .. Default::default()
            },
            .. Default::default()
        };


    for (offset_x, offset_y) in [(0.0, 0.0), ctx.offset].iter() {
        let offset = [*offset_x, *offset_y];

        if ctx.draw_grid {
            target
                .draw(
                    (ctx.points_vb, ctx.attractors_vb, ctx.kinds_vb),
                    ctx.lines_ib,
                    ctx.lines_program,
                    &uniform! {
                        a: 0.0f32,
                        b: 0.0f32,
                        offset: offset,
                        color: [0.3, 0.3, 0.3f32],
                    },
                    params,
                )
                .unwrap();
        }

        if ctx.draw_triangles {
            target
                .draw(
                    (ctx.points_vb, ctx.attractors_vb, ctx.kinds_vb),
                    ctx.triangles_ib,
                    ctx.triangles_program,
                    &uniform! {
                        a: ctx.a,
                        b: ctx.b,
                        offset: offset,
                        color: [0.2, 0.1, 0.1f32],
                    },
                    params,
                )
                .unwrap();
        }

        if ctx.draw_lines {
            target
                .draw(
                    (ctx.points_vb, ctx.attractors_vb, ctx.kinds_vb),
                    ctx.lines_ib,
                    ctx.lines_program,
                    &uniform! {
                        a: ctx.a,
                        b: ctx.b,
                        offset: offset,
                        color: [1.0, 1.0, 1.0f32],
                    },
                    params,
                )
                .unwrap();
        }
    }

    target.finish()
}

fn main() -> Result<(), Box<std::error::Error>> {
    let mut width = 800;
    let mut height = 800;

    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new().with_dimensions(width, height);
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop)?;

    let rows = 5;
    let cols = 5;

    let size = (1.0 - -1.0) / ((cols - 1) as f32 * 1.5);

    let (points, lines, triangles, attractors, kinds) = geotoy::mesh(rows, cols, size);

    let points_vb = glium::VertexBuffer::new(&display, &points)?;
    let attractors_vb = glium::VertexBuffer::new(&display, &attractors)?;
    let kinds_vb = glium::VertexBuffer::new(&display, &kinds)?;
    let lines_ib = glium::IndexBuffer::new(&display, PrimitiveType::LinesList, &lines)?;
    let triangles_ib = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &triangles)?;

    let lines_program = program!(&display,
        140 => {
            vertex: geotoy::VERTEX_SHADER,
            fragment: geotoy::FRAGMENT_SHADER,
        },
    )?;

    let triangles_program = program!(&display,
        140 => {
            vertex: geotoy::VERTEX_SHADER,
            fragment: geotoy::FRAGMENT_SHADER,
        },
    )?;

    let mut draw_context = DrawContext {
        display:           &display,
        lines_program:     &lines_program,
        triangles_program: &triangles_program,
        a:                 0.1,
        b:                 0.6,
        offset:            (0.0, 0.0),
        draw_grid:         true,
        draw_triangles:    true,
        draw_lines:        true,
        points_vb:         &points_vb,
        attractors_vb:     &attractors_vb,
        kinds_vb:          &kinds_vb,
        lines_ib:          &lines_ib,
        triangles_ib:      &triangles_ib,
    };

    let mut cursor_position_fn = CursorPositionFn::HexParams;

    draw(&draw_context)?;

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
                                draw_context.draw_grid = !draw_context.draw_grid;
                                need_draw = true;
                            }
                            VirtualKeyCode::T | VirtualKeyCode::P => {
                                // KeyCode P is for "polygons", consistent with web version
                                draw_context.draw_triangles = !draw_context.draw_triangles;
                                need_draw = true;
                            }
                            VirtualKeyCode::L => {
                                draw_context.draw_lines = !draw_context.draw_lines;
                                need_draw = true;
                            }
                            VirtualKeyCode::O => {
                                cursor_position_fn = match cursor_position_fn {
                                    CursorPositionFn::HexParams => CursorPositionFn::Offset,
                                    CursorPositionFn::Offset    => CursorPositionFn::HexParams,
                                };
                            }
                            _ => {},
                        };
                    }
                }
                glutin::WindowEvent::Closed => should_quit = true,
                glutin::WindowEvent::Resized(w, h) => {
                    width = w;
                    height = h;
                    need_draw = true;
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    match cursor_position_fn {
                        CursorPositionFn::HexParams => {
                            draw_context.a = ((position.0 as f32) / (width as f32) - 0.5) * 10.0;
                            draw_context.b = ((position.1 as f32) / (height as f32) - 0.5) * 10.0;
                        }
                        CursorPositionFn::Offset => {
                            draw_context.offset.0 = (position.0 as f32) / (width  as f32) - 0.5;
                            draw_context.offset.1 = -((position.1 as f32) / (height as f32) - 0.5);
                        }
                    }
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
            draw(&draw_context)?;
        }
    }
}
