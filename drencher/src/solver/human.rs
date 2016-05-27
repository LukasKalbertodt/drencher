//! Interactive solver via Terminal.
//!
//! This solver will interactively ask the user to choose a color
//! and adds this color to the solution vector. The user is presented
//! with the new board state.

use color::Color;
use board::Board;
use super::{Solver, Solution};
// use std::io::{self, Write};
use glium::{self, DisplayBuild, Surface};
use glium::glutin::{ElementState, Event, VirtualKeyCode};

/// Type definition for the solver.
pub struct Human;

impl Solver for Human {
    // implement this to avoid printing all board states again
    fn prints_output(&self) -> bool { true }

    fn solve(&self, _: Board) -> Result<Solution, Solution> {
        let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        let vertex1 = Vertex { position: [-0.5, -0.5] };
        let vertex2 = Vertex { position: [ 0.0,  0.5] };
        let vertex3 = Vertex { position: [ 0.5, -0.25] };
        let shape = vec![vertex1, vertex2, vertex3];

        let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let vertex_shader_src = r#"
            #version 140

            in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            out vec4 color;

            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program = glium::Program::from_source(
            &display,
            vertex_shader_src,
            fragment_shader_src,
            None
        ).unwrap();

        'a: loop {
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 1.0, 1.0);
            target.draw(&vertex_buffer, &indices, &program, &glium::uniforms::EmptyUniforms,
                        &Default::default()).unwrap();
            target.finish().unwrap();

            for ev in display.poll_events() {
                match ev {
                    Event::Closed |
                    Event::KeyboardInput(
                        ElementState::Pressed, _, Some(VirtualKeyCode::Escape)
                    ) => break 'a,
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
        Err(vec![])
    }
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
