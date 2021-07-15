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
    glutin::{self, event::VirtualKeyCode}, Surface,
    glutin::event_loop::ControlFlow::{self, Exit},
};

struct DrawContext {
    a: f32,
    b: f32,
    offset: (f32, f32),
    draw_grid: bool,
    draw_triangles: bool,
    draw_lines: bool,
    display: glium::Display,
    lines_program: glium::program::Program,
    triangles_program: glium::program::Program,
    points_vb: glium::VertexBuffer<Point>,
    attractors_vb: glium::VertexBuffer<Attractor>,
    kinds_vb: glium::VertexBuffer<Kind>,
    lines_ib: glium::IndexBuffer<u16>,
    triangles_ib: glium::IndexBuffer<u16>,
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
                    (&ctx.points_vb, &ctx.attractors_vb, &ctx.kinds_vb),
                    &ctx.lines_ib,
                    &ctx.lines_program,
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
                    (&ctx.points_vb, &ctx.attractors_vb, &ctx.kinds_vb),
                    &ctx.triangles_ib,
                    &ctx.triangles_program,
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
                    (&ctx.points_vb, &ctx.attractors_vb, &ctx.kinds_vb),
                    &ctx.lines_ib,
                    &ctx.lines_program,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new().with_inner_size(
        glium::glutin::dpi::LogicalSize::new(800, 800)
    );
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &event_loop)?;

    let mut size = display.gl_window().window().inner_size();

    let rows = 5;
    let cols = 5;

    let mesh_size = (1.0 - -1.0) / ((cols - 1) as f32 * 1.5);

    let (points, lines, triangles, attractors, kinds) = geotoy::mesh(rows, cols, mesh_size);

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
        a:                 0.1,
        b:                 0.6,
        offset:            (0.0, 0.0),
        draw_grid:         true,
        draw_triangles:    true,
        draw_lines:        true,
        display,
        lines_program,
        triangles_program,
        points_vb,
        attractors_vb,
        kinds_vb,
        lines_ib,
        triangles_ib,
    };

    let mut cursor_position_fn = CursorPositionFn::HexParams;

    draw(&draw_context)?;

    event_loop.run(move |event, _target, control_flow| {
        let mut need_draw = false;
        *control_flow = ControlFlow::Wait;

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::KeyboardInput { input, .. }
                    if input.state == glutin::event::ElementState::Pressed =>
                {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::Escape => *control_flow = Exit,
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
                glutin::event::WindowEvent::CloseRequested => *control_flow = Exit,
                glutin::event::WindowEvent::Resized(new_size) => {
                    size = new_size;
                    need_draw = true;
                }
                glutin::event::WindowEvent::CursorMoved { position, .. } => {
                    match cursor_position_fn {
                        CursorPositionFn::HexParams => {
                            draw_context.a = ((position.x as f32) / (size.width as f32) - 0.5) * 10.0;
                            draw_context.b = ((position.y as f32) / (size.height as f32) - 0.5) * 10.0;
                        }
                        CursorPositionFn::Offset => {
                            draw_context.offset.0 = (position.x as f32) / (size.width as f32) - 0.5;
                            draw_context.offset.1 = -((position.y as f32) / (size.height as f32) - 0.5);
                        }
                    }
                    need_draw = true;
                }
                _ => (),
            },
            _ => (),
        };

        if need_draw {
            draw(&draw_context).unwrap_or_else(|e| {
                eprintln!("{}", e);
                *control_flow = Exit;
            });
        }
    });
}
