use std::{env, io};
use std::io::{Write};
use std::process::exit;

use crate::b_tree::{SimpleRow, Table};
use crate::state::{ProgramState};
use crate::status::{ExecuteResult, MetaCommandResult, PreparedStatementResult, StatementType};
use crate::status::ExecuteResult::{ExecuteFailed, ExecuteSuccess, ExecuteTableFull};
use crate::status::MetaCommandResult::{MetaCommandSuccess, MetaCommandUnrecognized};
use crate::status::PreparedStatementResult::{PreparedStatementError, PreparedStatementSuccess, PreparedStatementSyntaxError, PreparedStatementUnrecognized};
use crate::status::StatementType::Invalid;

mod b_tree;
mod state;
mod status;

const  ID_SIZE: usize = 4;
const  USERNAME_SIZE: usize = 32;
const  EMAIL_SIZE: usize = 255;

static ID_OFFSET: usize = 0;
static USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
static EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;

static ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
static ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
static TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;


fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();

    if args.len() <= 0 {
        let exc_path: String;
        if args.len() != 0 {
            exc_path = args[0].to_string();
        } else {
            exc_path = "sqlite_lite".to_string();
        }
        println!("{}", usage(exc_path));
        println!("Goodbye!");
        exit(1);
    }

    let db_file_name = args[1].to_string();
    let mut table: Table = Table::new(&db_file_name);

    if ! table.file_ok {
        println!("File {file} doesn't exist and could not be created. Goodbye!", file=db_file_name);
        exit(1);
    }

    let mut program_state: ProgramState = ProgramState::new();
    let mut input_buffer: InputBuffer = InputBuffer::new();
    program_state.set_running(true);

    while program_state.get_running() {
        print_prompt();
        read_input(&mut input_buffer);

        let input = input_buffer.buffer.as_str().trim();
        let mut statement: PreparedStatement = PreparedStatement::new();

        if input.starts_with(".") {
            match do_meta_command(input, &mut program_state) {
                MetaCommandSuccess => {
                    continue;
                }
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
                PreparedStatementSyntaxError => {
                    println!("Syntax error in: {:?}", input);
                    continue;
                }
                PreparedStatementError => {
                    println!("Could not prepare statement: {:?}", input);
                    continue;
                }
            }
        }

        let exec_result: ExecuteResult = execute_prepared_statement(statement, &mut table);
        match exec_result {
            ExecuteSuccess => {
                println!("Execution Success!");
            }
            ExecuteFailed => {println!("Execution Failed!")}
            ExecuteTableFull => {println!("Table is full!")}
        }
    }

    table.drop();
    println!("Goodbye!");
    Ok(())
}

fn read_input(input_buffer: &mut InputBuffer) {
    input_buffer.buffer = "".to_string();
    io::stdin()
        .read_line(&mut input_buffer.buffer)
        .expect("Could not parse input!");
}

fn do_meta_command(command: &str, program_state: &mut ProgramState) -> MetaCommandResult{
    match command {
        ".exit" => {
            program_state.set_running(false);
        },
        ".test" => println!("Test worked!"),
        _ => return MetaCommandUnrecognized
    }

    MetaCommandSuccess
}

fn do_prepared_statements(input: &str, statement: &mut PreparedStatement) -> PreparedStatementResult {
    if input.to_uppercase().starts_with("SELECT") {
        statement.statement_type = StatementType::Select;
    } else if input.to_uppercase().starts_with("INSERT") {
        statement.statement_type = StatementType::Insert;
        let split_info = input.split_whitespace().collect::<Vec<&str>>();

        if split_info.len() < 4 {
            return PreparedStatementSyntaxError
        }

        statement.row.set_id(split_info.get(1).unwrap().parse::<u32>().unwrap_or(0));
        if statement.row.get_id() == 0 {
            return PreparedStatementSyntaxError
        }

        if ! statement.row.set_username(split_info.get(2).unwrap().parse().unwrap()) {
            return PreparedStatementError
        }
        if ! statement.row.set_email(split_info.get(3).unwrap().parse().unwrap()) {
            return PreparedStatementError
        }
    } else {
        return PreparedStatementUnrecognized
    }

    PreparedStatementSuccess
}

fn execute_prepared_statement(statement: PreparedStatement, table: &mut Table) -> ExecuteResult {
    let mut exec_result: ExecuteResult = ExecuteFailed;
    match statement.statement_type {
        Invalid => {}
        StatementType::Insert => {
            exec_result = execute_insert(statement, table);
        }
        StatementType::Select => {
            exec_result = execute_select(statement, table);
        }
    }

    exec_result
}

fn set_row_slot(table: &mut Table, row: SimpleRow, _row_num: usize) -> bool { // TODO make member function
    // let page_num: usize = row_num / ROWS_PER_PAGE;
    // let row_slot = row_num % ROWS_PER_PAGE;

    /*if table.rows.len() < row_num {
        println!("nah");
        return false
    } else if table.rows.len() == row_num {
        println!("push");
        table.rows.push(row);
    } else {
        println!("set");
        table.rows[row_num] = row;
    }
     */

    table.rows.push(row);
    table.num_rows += 1;

    true
}

fn _append_row_slot(table: &mut Table, row: SimpleRow) -> bool { // TODO make member function
    return set_row_slot(table, row, &table.num_rows + 1);
}

fn execute_insert(statement: PreparedStatement, table: &mut Table) -> ExecuteResult {
    if table.num_rows >= TABLE_MAX_ROWS {
        return ExecuteTableFull
    }

    let to_insert: SimpleRow = statement.row;
    if set_row_slot(table, to_insert, table.num_rows) {
        return ExecuteSuccess
    }

    ExecuteFailed
}

fn execute_select(_statement: PreparedStatement, table: &mut Table) -> ExecuteResult {
    for i in &table.rows {
        print_row(i);
    }

    ExecuteSuccess
}

fn print_row(row: &SimpleRow) { // TODO make member function
    println!("({id}, {username}, {email})", id=row.get_id(), username=row.get_username(), email=row.get_email())
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

fn usage(exec_path: String) -> String {
    format!("Usage: {exec_path} <database_file_name>")
}

struct PreparedStatement {
    statement_type: StatementType,
    row: SimpleRow,
}

impl PreparedStatement {
    fn new() -> Self {
        let row = SimpleRow::new();

        PreparedStatement {
            statement_type: Invalid,
            row
        }
    }
}

struct InputBuffer {
    buffer: String
}

impl InputBuffer {
    fn new() -> Self {
        InputBuffer {
            buffer: "".to_string(),
        }
    }
}
