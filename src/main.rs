#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::{
    glutin::{self, VirtualKeyCode}, Surface,
};

use std::f32::consts::PI;
use std::iter;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn midpoint(&self, rhs: &Point) -> Point {
        Point {
            x: (self.x + rhs.x) / 2.0,
            y: (self.y + rhs.y) / 2.0,
        }
    }
}

implement_vertex!(Point, x, y);

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Center {
    center: [f32; 2],
}

implement_vertex!(Center, center);

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Kind {
    kind: u32,
}

implement_vertex!(Kind, kind);

fn flat_hex_corner(center: Point, size: f32, i: usize) -> Point {
    let angle_deg = 60.0 * (i as f32);
    let angle_rad = PI / 180.0 * angle_deg;
    Point {
        x: center.x + size * angle_rad.cos(),
        y: center.y + size * angle_rad.sin(),
    }
}

#[derive(Copy, Clone, Debug)]
struct EvenqCoordinate {
    x: usize,
    y: usize,
}

impl EvenqCoordinate {
    fn center(&self, size: f32) -> Point {
        let x = size * (3.0 / 2.0) * (self.x as f32);
        let y = size * 3f32.sqrt() * ((self.y as f32) - 0.5 * ((self.x & 1) as f32));
        Point { x, y }
    }
}

fn coordinates(rows: usize, columns: usize) -> impl Iterator<Item = EvenqCoordinate> {
    (0..rows).flat_map(move |row| (0..columns).map(move |col| EvenqCoordinate { x: row, y: col }))
}

struct Hexagon {
    center: Center,
    points: [Point; 12],
    kinds: [Kind; 12],
    lines: [u32; 24],
}

const CORNER: Kind = Kind { kind: 0 };
const MID: Kind = Kind { kind: 1 };

impl Hexagon {
    fn points(center: Point, size: f32) -> [Point; 12] {
        let mut points: [Point; 12] = Default::default();
        for i in 0..6 {
            points[2 * i] = flat_hex_corner(center, size, i);
        }
        for i in 0..6 {
            points[2 * i + 1] = points[2 * i].midpoint(&points[2 * (i + 1) % 12]);
        }
        points
    }

    fn lines() -> [u32; 24] {
        [
            0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 0,
        ]
    }

    fn new(center: Point, size: f32) -> Hexagon {
        Hexagon {
            center: Center {
                center: [center.x, center.y],
            },
            points: Self::points(center, size),
            kinds: [
                CORNER, MID, CORNER, MID, CORNER, MID, CORNER, MID, CORNER, MID, CORNER, MID,
            ],
            lines: Self::lines(),
        }
    }
}

fn hexagons(rows: usize, columns: usize, size: f32) -> impl Iterator<Item = Hexagon> {
    coordinates(rows, columns)
        .map(move |coord| coord.center(size))
        .map(move |center| Hexagon::new(center, size))
}

fn draw(
    display: &glium::Display,
    program: &glium::program::Program,
    b: f32,
    points_vb: &glium::VertexBuffer<Point>,
    centers_vb: &glium::VertexBuffer<Center>,
    kinds_vb: &glium::VertexBuffer<Kind>,
    index_buffer: &glium::IndexBuffer<u32>,
) -> Result<(), glium::SwapBuffersError> {
    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 0.0, 0.0);

    target
        .draw(
            (points_vb, centers_vb, kinds_vb),
            index_buffer,
            program,
            &uniform! {
                b: 0.0f32,
                color: [0.3, 0.3, 0.3f32],
            },
            &Default::default(),
        )
        .unwrap();

    target
        .draw(
            (points_vb, centers_vb, kinds_vb),
            index_buffer,
            program,
            &uniform! {
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

    let rows = 10;
    let cols = 10;

    let size = (1.0 - -1.0) / ((cols - 1) as f32 * 1.5);

    let (points, lines, centers, kinds): (Vec<Point>, Vec<u32>, Vec<Center>, Vec<Kind>) =
        hexagons(rows, cols, size)
            .enumerate()
            .map(|(i, mut hex)| {
                let offset = i * hex.points.len();
                for idx in hex.lines.iter_mut() {
                    *idx += offset as u32;
                }
                hex
            })
            .map(|mut hex| {
                hex.center.center[0] -= 1.0;
                hex.center.center[1] -= 1.0;
                for p in &mut hex.points {
                    p.x -= 1.0;
                    p.y -= 1.0;
                }
                hex
            })
            .fold(
                (vec![], vec![], vec![], vec![]),
                |(mut points, mut lines, mut centers, mut kinds), hex| {
                    points.extend(hex.points.iter().cloned());
                    lines.extend(hex.lines.iter().cloned());
                    centers.extend(iter::repeat(hex.center).take(hex.points.len()));
                    kinds.extend(hex.kinds.iter().cloned());
                    (points, lines, centers, kinds)
                },
            );

    let points_vb = glium::VertexBuffer::new(&display, &points)?;
    let centers_vb = glium::VertexBuffer::new(&display, &centers)?;
    let kinds_vb = glium::VertexBuffer::new(&display, &kinds)?;
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::LinesList, &lines)?;

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in float x;
                in float y;

                in vec2 center;
                in uint kind;

                uniform float b;

                void main() {
                    if (kind == uint(0)) {
                        vec2 p = vec2(x, y);
                        vec2 v = center - p;
                        gl_Position = vec4(p + b * v, 0.0, 1.0);
                    } else {
                        gl_Position = vec4(x, y, 0.0, 1.0);
                    }
                }
            ",

            fragment: "
                #version 140

                uniform vec3 color;

                out vec4 f_color;

                void main() {
                    f_color = vec4(color, 1.0);
                }
            "
        },
    )?;

    let mut b: f32 = 0.6;

    draw(
        &display,
        &program,
        b,
        &points_vb,
        &centers_vb,
        &kinds_vb,
        &index_buffer,
    )?;

    // the main loop
    events_loop.run_forever(|event| {
        let mut need_draw = false;
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::KeyboardInput { input, .. }
                    if input.virtual_keycode == Some(VirtualKeyCode::Escape) =>
                {
                    return glutin::ControlFlow::Break
                }
                // Break from the main loop when the window is closed.
                glutin::WindowEvent::Closed => return glutin::ControlFlow::Break,
                // Redraw the triangle when the window is resized.
                glutin::WindowEvent::Resized(w, h) => {
                    width = w;
                    height = h;
                    need_draw = true;
                }
                glutin::WindowEvent::CursorMoved { position, .. } => {
                    b = ((position.1 as f32) / (height as f32) - 0.5) * 10.0;
                    need_draw = true;
                }
                _ => (),
            },
            _ => (),
        }
        if need_draw {
            draw(
                &display,
                &program,
                b,
                &points_vb,
                &centers_vb,
                &kinds_vb,
                &index_buffer,
            ).unwrap()
        }
        glutin::ControlFlow::Continue
    });

    Ok(())
}
