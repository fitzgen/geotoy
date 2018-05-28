#[macro_use]
extern crate glium;

use glium::index::PrimitiveType;
use glium::{glutin, Surface};

use std::f32::consts::PI;

#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
struct Point {
    x: f32,
    y: f32,
}

implement_vertex!(Point, x, y);

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
    points: [Point; 6],
    lines: [u32; 12],
}

impl Hexagon {
    fn points(center: Point, size: f32) -> [Point; 6] {
        let mut points: [Point; 6] = Default::default();
        for i in 0..6 {
            points[i] = flat_hex_corner(center, size, i);
        }
        points
    }

    fn lines() -> [u32; 12] {
        [0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 0]
    }

    fn new(center: Point, size: f32) -> Hexagon {
        Hexagon {
            points: Self::points(center, size),
            lines: Self::lines(),
        }
    }
}

fn hexagons(rows: usize, columns: usize, size: f32) -> impl Iterator<Item = Hexagon> {
    coordinates(rows, columns)
        .map(move |coord| coord.center(size))
        .map(move |center| Hexagon::new(center, size))
}

// struct Hexagons {

// }

fn main() -> Result<(), Box<std::error::Error>> {
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &events_loop)?;

    let rows = 5;
    let cols = 5;

    let size = (1.0 - -1.0) / ((cols - 1) as f32 * 1.5);

    let (points, lines): (Vec<_>, Vec<_>) = hexagons(rows, cols, size)
        .enumerate()
        .map(|(i, hex)| {
            let offset = i * hex.points.len();
            let lines: Vec<_> = hex.lines
                .iter()
                .cloned()
                .map(|idx| idx + (offset as u32))
                .collect();
            (hex.points, lines)
        })
        .unzip();

    let points: Vec<_> = points
        .into_iter()
        .flat_map(|points| {
            points
                .iter()
                .cloned()
                .map(|p| {
                    Point {
                        x: p.x - 1.0,
                        y: p.y - 1.0,
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let lines: Vec<_> = lines.into_iter().flat_map(|lines| lines).collect();

    let vertex_buffer = glium::VertexBuffer::new(&display, &points)?;
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::LinesList, &lines)?;

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                in float x;
                in float y;
                void main() {
                    gl_Position = vec4(x, y, 0.0, 1.0);
                }
            ",

            fragment: "
                #version 140
                out vec4 f_color;
                void main() {
                    f_color = vec4(1.0, 1.0, 1.0, 1.0);
                }
            "
        },
    )?;

    // Here we draw the black background and triangle to the screen using the previously
    // initialised resources.
    //
    // In this case we use a closure for simplicity, however keep in mind that most serious
    // applications should probably use a function that takes the resources as an argument.
    let draw = || {
        // building the uniforms
        let uniforms = uniform! {
        //     matrix: [
        //         [1.0, 0.0, 0.0, 0.0],
        //         [0.0, 1.0, 0.0, 0.0],
        //         [0.0, 0.0, 1.0, 0.0],
        //         [0.0, 0.0, 0.0, 1.0f32]
        //     ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish()
    };

    // Draw the triangle to the screen.
    draw()?;

    // the main loop
    events_loop.run_forever(|event| {
        match event {
            glutin::Event::WindowEvent { event, .. } => match event {
                // Break from the main loop when the window is closed.
                glutin::WindowEvent::Closed => return glutin::ControlFlow::Break,
                // Redraw the triangle when the window is resized.
                glutin::WindowEvent::Resized(..) => draw().unwrap(),
                _ => (),
            },
            _ => (),
        }
        glutin::ControlFlow::Continue
    });

    Ok(())
}
