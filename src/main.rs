use std::io;
use std::io::Write;

fn main() {
    let mut input_buffer = InputBuffer::new();

    'main_loop: loop {
        print_prompt();
        read_input(&mut input_buffer);

        let input = input_buffer.buffer.as_str().trim();

        match input {
            ".exit" => break 'main_loop,
            _ => {
                println!("Unrecognized command {:?}", input);
            }
        }
    }


}

struct InputBuffer {
    buffer: String
}

impl InputBuffer {
    pub fn new() -> Self {
        InputBuffer {
            buffer: "".to_string(),
        }
    }
}

fn read_input(mut input_buffer: &mut InputBuffer) {
    input_buffer.buffer = "".to_string();
    io::stdin()
        .read_line(&mut input_buffer.buffer)
        .expect("Could not parse input!");
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}
