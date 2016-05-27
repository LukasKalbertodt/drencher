//! Interactive solver via Terminal.
//!
//! This solver will interactively ask the user to choose a color
//! and adds this color to the solution vector. The user is presented
//! with the new board state.

use board::Board;
use color::Color;
use super::{Solver, Solution};
// use std::io::{self, Write};
use glium::{self, glutin, DisplayBuild, Surface};
use glium::glutin::{ElementState, Event, VirtualKeyCode};

// constants that modify the appearance
const MARGIN: f32 = 0.1;
const CELL_DISTANCE: f32 = 0.0;


/// Type definition for the solver.
pub struct Human;

impl Solver for Human {
    // implement this to avoid printing all board states again
    fn prints_output(&self) -> bool { true }

    fn solve(&self, mut board: Board) -> Result<Solution, Solution> {
        let display = glutin::WindowBuilder::new()
            .with_title("Drencher")
            .with_vsync()
            .with_srgb(Some(false))
            .with_pixel_format(24, 8)
            .build_glium()
            .expect("creating OpenGL window failed!");

        // calculate positions
        let (width, height) = display.get_framebuffer_dimensions();
        let (mut xmin, mut ymin, mut xmax, mut ymax) =
            get_positions(width, height);
        println!("{:?}", (xmin, xmin, xmax, ymax));


        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        let shape = vec![
            Vertex { position: [0.0, 0.0] },
            Vertex { position: [0.0, 1.0] },
            Vertex { position: [1.0, 0.0] },
            Vertex { position: [1.0, 1.0] },
        ];

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

        let vertex_shader_src = r#"
            #version 140

            in vec2 position;

            uniform vec2 pos;
            uniform vec2 scale;

            void main() {
                gl_Position = vec4(position * scale + pos, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color;

            uniform vec3 field_color;

            void main() {
                // perform srgb conversion (don't know why I need to 0.o)
                vec3 tmp = field_color;
                color = vec4(pow(tmp.x, 2.4), pow(tmp.y, 2.4), pow(tmp.z, 2.4), 1.0);
            }
        "#;

        let program = glium::Program::from_source(
            &display,
            vertex_shader_src,
            fragment_shader_src,
            None
        ).unwrap();

        let mut solution = Solution::new();

        'a: loop {
            let mut target = display.draw();
            target.clear_color(0.02, 0.02, 0.05, 1.0);

            let x_distance = (xmax - xmin) / (board.size() as f32);
            let y_distance = (ymax - ymin) / (board.size() as f32);
            for x in 0..board.size() {
                for y in 0..board.size() {

                    target.draw(
                        &vertex_buffer,
                        &indices,
                        &program,
                        &uniform! {
                            scale: [
                                x_distance * (1.0 - CELL_DISTANCE),
                                y_distance * (1.0 - CELL_DISTANCE)
                            ],
                            pos: [
                                xmin + (x as f32) * x_distance,
                                ymax - ((y + 1) as f32) * y_distance,
                            ],
                            field_color: board[(x, y)].as_rgb(),
                        },
                        &Default::default()
                    ).unwrap();
                }
            }

            target.finish().unwrap();

            for ev in display.poll_events() {
                match ev {
                    Event::Closed |
                    Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Escape))
                        => break 'a,
                    Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Return))
                        if board.is_drenched() => return Ok(solution),
                    Event::KeyboardInput(ElementState::Pressed, _, Some(vkc))
                        if !board.is_drenched() =>
                    {
                        let color = match vkc {
                            VirtualKeyCode::Key1 => Some(0),
                            VirtualKeyCode::Key2 => Some(1),
                            VirtualKeyCode::Key3 => Some(2),
                            VirtualKeyCode::Key4 => Some(3),
                            VirtualKeyCode::Key5 => Some(4),
                            VirtualKeyCode::Key6 => Some(5),
                            _ => None,
                        }.map(|n| Color::new(n));

                        if let Some(color) = color {
                            if solution.last() != Some(&color) {
                                solution.push(color);
                                board.drench(color);
                            }
                        }
                    }
                    Event::Resized(width, height) => {
                        let (nxmin, nymin, nxmax, nymax) =
                            get_positions(width, height);
                        xmin = nxmin;
                        ymin = nymin;
                        xmax = nxmax;
                        ymax = nymax;
                    },

                    _ => ()
                }
            }
        }


        // // print initial board state
        // println!("+++++ Initial board:");
        // println!("{}", b);

        // // while the user still inputs a color...
        // while let Some(color) = prompt_color() {
        //     println!("+++++ drenching {}", color);

        //     out.push(color);
        //     b.drench(color);
        //     println!("{}", b);

        //     if b.is_drenched() {
        //         return Ok(out);
        //     }
        // }

        // // apparently we didn't solve the board, but we don't have any more
        // // inputs
        Err(solution)
    }
}

fn get_positions(width: u32, height: u32) -> (f32, f32, f32, f32) {
    let xmargin = if width > height {
        ((width - height) as f32) / (width as f32)
    } else {
        0.0
    };
    let ymargin = if height > width {
        ((height - width) as f32) / (height as f32)
    } else {
        0.0
    };

    let xmin = (-1.0 + xmargin) * (1.0 - MARGIN);
    let xmax = ( 1.0 - xmargin) * (1.0 - MARGIN);
    let ymin = (-1.0 + ymargin) * (1.0 - MARGIN);
    let ymax = ( 1.0 - ymargin) * (1.0 - MARGIN);

    (xmin, ymin, xmax, ymax)
}

// fn prompt_color() -> Option<Color> {
//     // While the user gives us invalid input, we simply loop
//     loop {
//         // show all possible colors
//         print!("Color to drench with next?");
//         print!(" ({}->{}", 1, Color::new(0));
//         for n in 1..6 {
//             let c = Color::new(n);
//             print!(", {}->{}", n + 1, c);
//         }
//         print!(")");

//         // flush and return `None` if it wasn't sucessful (very unlikely)
//         let res = io::stdout().flush();
//         if res.is_err() {
//             println!("Wasn't able to flush stdout!");
//             return None;
//         }

//         // Read the users input and try to parse it as `u8`. Meanings of the
//         // values of `maybe_num`:
//         // - Err(true) -> an IO error occured
//         // - Err(false) -> a parsing error occured
//         // - Ok(_) -> the parsed `u8`
//         let mut buffer = String::new();
//         let maybe_num = io::stdin()
//             .read_line(&mut buffer)
//             .map_err(|_| true)
//             .map(|_| buffer.trim())
//             .and_then(|line| line.parse().map_err(|_| false));

//         match maybe_num {
//             Err(true) => return None,
//             Err(false) => println!("Not a number!"),
//             Ok(0) => println!("Colors are 1-indexed!"),
//             Ok(n) => return Some(Color::new(n - 1)),
//         }
//     }
// }
