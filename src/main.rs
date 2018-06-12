#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::{
    Blend,
    BlendingFunction,
    DrawParameters,
    LinearBlendingFactor,
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

/// A corner point that's pulled by the center attractor into the inside of the tile
const INTERNAL: Kind = Kind { kind: 0 };
/// The midpoint of the hexagon edge, possibly pulled by the corner attractor
const MID: Kind = Kind { kind: 1 };
/// A fixed corner on the hexagon (ignored attractor)
const CORNER: Kind = Kind { kind: 2 };

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
        triangles: &mut Vec<u32>,
        kinds: &mut Vec<Kind>,
        attractors: &mut Vec<Attractor>,
    ) {
        points.push(self.center);
        // These two are unused for the center point, since it doesn't show up
        // in any lines. However, to avoid offset errors, there must be
        // something here to avoid offset errors in the parallel vecs.
        attractors.push(self.center.into());
        kinds.push(INTERNAL);

        let internals_idx = points.len();
        points.extend(self.corners.iter().cloned());
        kinds.extend(iter::repeat(INTERNAL).take(self.corners.len()));
        attractors.extend(iter::repeat::<Attractor>(self.center.into()).take(self.corners.len()));
        assert_eq!(points.len(), kinds.len());
        assert_eq!(points.len(), attractors.len());

        let midpoints_idx = points.len();
        points.extend(self.midpoints.iter().cloned());
        kinds.extend(iter::repeat(MID).take(self.midpoints.len()));
        for i in 0..(self.midpoints.len() / 2) {
            attractors.push(self.corners[i].into());
            attractors.push(self.corners[(i + 1) % self.corners.len()].into());
        }
        assert_eq!(points.len(), kinds.len());
        assert_eq!(points.len(), attractors.len());

        let corners_idx = points.len();
        points.extend(self.corners.iter().cloned());
        kinds.extend(iter::repeat(CORNER).take(self.corners.len()));
        attractors.extend(iter::repeat::<Attractor>(self.center.into()).take(self.corners.len())); // ignored
        assert_eq!(points.len(), kinds.len());
        assert_eq!(points.len(), attractors.len());

        for i in 0..6 {
            // Internal to first midpoint.
            lines.push((internals_idx + i) as u32);
            lines.push((midpoints_idx + (i * 2 + 1) % self.midpoints.len()) as u32);

            // Other internal to second midpoint.
            lines.push((internals_idx + (i + 1) % self.corners.len()) as u32);
            lines.push((midpoints_idx + (i * 2)) as u32);

            // Internal-midpoint-corner
            triangles.push((internals_idx + i) as u32);
            triangles.push((midpoints_idx + (i * 2 + 1) % self.midpoints.len()) as u32);
            triangles.push((corners_idx + i) as u32);

            // Internal-second midpoint-second corner
            triangles.push((internals_idx + (i + 1) % self.corners.len()) as u32);
            triangles.push((midpoints_idx + (i * 2)) as u32);
            triangles.push((corners_idx + (i + 1) % self.corners.len()) as u32);
        }
    }
}

fn hexagons(rows: usize, columns: usize, size: f32) -> impl Iterator<Item = Hexagon> {
    coordinates(rows, columns)
        .map(move |coord| coord.center(size))
        .map(move |center| Hexagon::new(center, size))
}

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
    lines_ib: &'a glium::IndexBuffer<u32>,
    triangles_ib: &'a glium::IndexBuffer<u32>,
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
                        color: [1.0, 1.0, 1.0f32],
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

    let (points, lines, triangles, attractors, kinds): (Vec<Point>, Vec<u32>, Vec<u32>, Vec<Attractor>, Vec<Kind>) =
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
                (vec![], vec![], vec![], vec![], vec![]),
                |(mut points, mut lines, mut triangles, mut attractors, mut kinds), hex| {
                    hex.add_to_mesh(&mut points, &mut lines, &mut triangles, &mut kinds, &mut attractors);

                    (points, lines, triangles, attractors, kinds)
                },
            );

    let points_vb = glium::VertexBuffer::new(&display, &points)?;
    let attractors_vb = glium::VertexBuffer::new(&display, &attractors)?;
    let kinds_vb = glium::VertexBuffer::new(&display, &kinds)?;
    let lines_ib = glium::IndexBuffer::new(&display, PrimitiveType::LinesList, &lines)?;
    let triangles_ib = glium::IndexBuffer::new(&display, PrimitiveType::TrianglesList, &triangles)?;

    let lines_program = program!(&display,
        140 => {
            vertex: include_str!("star.v.glsl"),
            fragment: include_str!("lines.f.glsl"),
        },
    )?;

    let triangles_program = program!(&display,
        140 => {
            vertex: include_str!("star.v.glsl"),
            fragment: include_str!("triangles.f.glsl"),
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
        events_loop.poll_events(|event| {
            match event {
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
                                VirtualKeyCode::T => {
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
            }
        });

        if should_quit {
            return Ok(());
        }

        if need_draw {
            draw(&draw_context)?;
        }
    }
}
