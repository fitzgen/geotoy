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
struct Attractor {
    attractor: [f32; 2],
}

impl From<Point> for Attractor {
    fn from(p: Point) -> Attractor {
        Attractor {
            attractor: [p.x, p.y],
        }
    }
}

implement_vertex!(Attractor, attractor);

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
    center: Point,
    corners: [Point; 6],
    midpoints: [Point; 12],
}

const CORNER: Kind = Kind { kind: 0 };
const MID: Kind = Kind { kind: 1 };

impl Hexagon {
    fn corners(center: Point, size: f32) -> [Point; 6] {
        let mut corners: [Point; 6] = Default::default();
        for i in 0..6 {
            corners[i] = flat_hex_corner(center, size, i);
        }
        corners
    }

    fn midpoints(corners: &[Point; 6]) -> [Point; 12] {
        let mut midpoints: [Point; 12] = Default::default();
        for i in 0..6 {
            midpoints[2 * i] = corners[i].midpoint(&corners[(i + 1) % corners.len()]);
            midpoints[2 * i + 1] = corners[i].midpoint(&corners[(i + 1) % corners.len()]);
        }
        midpoints
    }

    fn new(center: Point, size: f32) -> Hexagon {
        let corners = Self::corners(center, size);
        let midpoints = Self::midpoints(&corners);
        Hexagon {
            center,
            corners,
            midpoints,
        }
    }

    fn add_to_mesh(
        self,
        points: &mut Vec<Point>,
        lines: &mut Vec<u32>,
        kinds: &mut Vec<Kind>,
        attractors: &mut Vec<Attractor>,
    ) {
        points.push(self.center);
        // These two are unused for the center point, since it doesn't show up
        // in any lines. However, to avoid offset errors, there must be
        // something here to avoid offset errors in the parallel vecs.
        attractors.push(self.center.into());
        kinds.push(CORNER);

        let corners_idx = points.len();
        points.extend(self.corners.iter().cloned());
        kinds.extend(iter::repeat(CORNER).take(self.corners.len()));
        attractors.extend(iter::repeat::<Attractor>(self.center.into()).take(self.corners.len()));

        let midpoints_idx = points.len();
        points.extend(self.midpoints.iter().cloned());
        kinds.extend(iter::repeat(MID).take(self.midpoints.len()));
        for i in 0..6 {
            attractors.push(self.corners[i].into());
            attractors.push(self.corners[(i + 1) % self.corners.len()].into());

            // Corner to first midpoint.
            lines.push((corners_idx + i) as u32);
            lines.push((midpoints_idx + (i * 2 + 1) % self.midpoints.len()) as u32);

            // Other corner to second midpoint.
            lines.push((corners_idx + (i + 1) % self.corners.len()) as u32);
            lines.push((midpoints_idx + (i * 2)) as u32);
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

    let (points, lines, attractors, kinds): (Vec<Point>, Vec<u32>, Vec<Attractor>, Vec<Kind>) =
        hexagons(rows, cols, size)
            .map(|mut hex| {
                hex.center.x -= 1.0;
                hex.center.y -= 1.0;
                for p in &mut hex.corners {
                    p.x -= 1.0;
                    p.y -= 1.0;
                }
                for p in &mut hex.midpoints {
                    p.x -= 1.0;
                    p.y -= 1.0;
                }
                hex
            })
            .fold(
                (vec![], vec![], vec![], vec![]),
                |(mut points, mut lines, mut attractors, mut kinds), hex| {
                    hex.add_to_mesh(&mut points, &mut lines, &mut kinds, &mut attractors);

                    (points, lines, attractors, kinds)
                },
            );

    let points_vb = glium::VertexBuffer::new(&display, &points)?;
    let attractors_vb = glium::VertexBuffer::new(&display, &attractors)?;
    let kinds_vb = glium::VertexBuffer::new(&display, &kinds)?;
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::LinesList, &lines)?;

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140

                in float x;
                in float y;

                in vec2 attractor;
                in uint kind;

                uniform float a;
                uniform float b;

                void main() {
                    float multiplier = kind == uint(0) ? b : a;

                    vec2 p = vec2(x, y);
                    vec2 v = attractor - p;
                    gl_Position = vec4(p + multiplier * v, 0.0, 1.0);
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

    // the main loop
    events_loop.run_forever(|event| {
        let mut need_draw = false;
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                glutin::WindowEvent::KeyboardInput { input, .. }
                    if input.state == glutin::ElementState::Pressed =>
                {
                    if let Some(keycode) = input.virtual_keycode {
                        match keycode {
                            VirtualKeyCode::Escape => return glutin::ControlFlow::Break,
                            VirtualKeyCode::G => {
                                draw_grid = !draw_grid;
                                need_draw = true;
                            }
                            _ => {},
                        }
                    }
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
                    a = ((position.0 as f32) / (width as f32) - 0.5) * 10.0;
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
                a,
                b,
                draw_grid,
                &points_vb,
                &attractors_vb,
                &kinds_vb,
                &index_buffer,
            ).unwrap()
        }
        glutin::ControlFlow::Continue
    });

    Ok(())
}
