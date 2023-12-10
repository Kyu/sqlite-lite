use std::io;
use std::io::Write;
use std::process::exit;

use crate::MetaCommandResult::{MetaCommandSuccess, MetaCommandUnrecognized};
use crate::PreparedStatementResult::{PreparedStatementSuccess, PreparedStatementUnrecognized};
use crate::StatementType::Invalid;

fn main() {
    let mut input_buffer = InputBuffer::new();

    loop {
        print_prompt();
        read_input(&mut input_buffer);

        let input = input_buffer.buffer.as_str().trim();
        let mut statement = PreparedStatement::new();

        if input.starts_with(".") {
            match do_meta_command(input) {
                MetaCommandSuccess => {}
                MetaCommandUnrecognized => {
                    println!("Unrecognized command {:?}", input);
                    continue;
                }
            }
        } else {
            match do_prepared_statements(input, &mut statement) {
                PreparedStatementSuccess => {}
                PreparedStatementUnrecognized => {
                    println!("Unrecognized statement {:?}", input);
                    continue;
                }
            }
        }

        execute_prepared_statement(statement);
        println!("Executed!");
    }


}

fn read_input(mut input_buffer: &mut InputBuffer) {
    input_buffer.buffer = "".to_string();
    io::stdin()
        .read_line(&mut input_buffer.buffer)
        .expect("Could not parse input!");
}

fn do_meta_command(command: &str) -> MetaCommandResult{
    match command {
        ".exit" => {
            exit(0);
        },
        ".test" => println!("Test worked!"),
        _ => return MetaCommandUnrecognized
    }

    MetaCommandSuccess
}

fn do_prepared_statements(input: &str, mut statement: &mut PreparedStatement) -> PreparedStatementResult {
    if input.to_uppercase().starts_with("SELECT") {
        statement.statement_type = StatementType::Select;
    } else if input.to_uppercase().starts_with("INSERT") {
        statement.statement_type = StatementType::Insert;
    } else {
        return PreparedStatementUnrecognized
    }

    PreparedStatementSuccess
}

fn execute_prepared_statement(statement: PreparedStatement) {
    match statement.statement_type {
        Invalid => {}
        StatementType::Insert => {
            println!("This is where we would do an insert.")
        }
        StatementType::Select => {
            println!("This is where we would do a select.")
        }
    }
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

enum StatementType {
    Invalid,
    Insert,
    Select
}

struct PreparedStatement {
    statement_type: StatementType
}

impl PreparedStatement {
    pub fn new() -> Self {
        PreparedStatement {
            statement_type: Invalid
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

enum MetaCommandResult {
    MetaCommandSuccess,
    MetaCommandUnrecognized
}

enum PreparedStatementResult {
    PreparedStatementSuccess,
    PreparedStatementUnrecognized
}
