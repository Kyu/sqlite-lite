use std::io;
use std::io::Write;
use std::process::exit;
use crate::ExecuteResult::{ExecuteSuccess, ExecuteTableFull};

use crate::MetaCommandResult::{MetaCommandSuccess, MetaCommandUnrecognized};
use crate::PreparedStatementResult::{PreparedStatementSuccess, PreparedStatementSyntaxError, PreparedStatementUnrecognized};
use crate::StatementType::Invalid;


// https://stackoverflow.com/a/70222282/3875151
macro_rules! size_of_attribute {
    ($t:ident :: $field:ident) => {{
        let m = core::mem::MaybeUninit::<$t>::uninit();
        // According to https://doc.rust-lang.org/stable/std/ptr/macro.addr_of_mut.html#examples,
        // you can dereference an uninitialized MaybeUninit pointer in addr_of!
        // Raw pointer deref in const contexts is stabilized in 1.58:
        // https://github.com/rust-lang/rust/pull/89551
        let p = unsafe {
            core::ptr::addr_of!((*(&m as *const _ as *const $t)).$field)
        };

        const fn size_of_raw<T>(_: *const T) -> usize {
            core::mem::size_of::<T>()
        }
        size_of_raw(p)
    }};
}

static ID_SIZE: usize = size_of_attribute!(SimpleRow::id);
static USERNAME_SIZE: usize = size_of_attribute!(SimpleRow::username);
static EMAIL_SIZE: usize = size_of_attribute!(SimpleRow::email);

static ID_OFFSET: usize = 0;
static USERNAME_OFFSET: usize = ID_OFFSET + ID_SIZE;
static EMAIL_OFFSET: usize = USERNAME_OFFSET + USERNAME_SIZE;

static ROW_SIZE: usize = ID_SIZE + USERNAME_SIZE + EMAIL_SIZE;

const PAGE_SIZE: usize = 4096;
const TABLE_MAX_PAGES: usize = 100;
static ROWS_PER_PAGE: usize = PAGE_SIZE / ROW_SIZE;
static TABLE_MAX_ROWS: usize = ROWS_PER_PAGE * TABLE_MAX_PAGES;


fn main() {
    let table: Table = Table::new();
    let mut input_buffer: InputBuffer = InputBuffer::new();

    loop {
        print_prompt();
        read_input(&mut input_buffer);

        let input = input_buffer.buffer.as_str().trim();
        let mut statement: PreparedStatement = PreparedStatement::new();

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
                PreparedStatementSyntaxError => {
                    println!("Syntax error in: {:?}", input);
                    continue;
                }
            }
        }

        execute_prepared_statement(statement);
        println!("Executed!");
    }
}

fn read_input(input_buffer: &mut InputBuffer) {
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

fn do_prepared_statements(input: &str, statement: &mut PreparedStatement) -> PreparedStatementResult {
    if input.to_uppercase().starts_with("SELECT") {
        statement.statement_type = StatementType::Select;
    } else if input.to_uppercase().starts_with("INSERT") {
        statement.statement_type = StatementType::Insert;
        let split_info = input.split_whitespace().collect::<Vec<&str>>();

        if split_info.len() < 4 {
            return PreparedStatementSyntaxError
        }

        statement.row.id = split_info.get(1).unwrap().parse::<u32>().unwrap_or(0);
        statement.row.username = split_info.get(2).unwrap().parse().unwrap();
        statement.row.email = split_info.get(3).unwrap().parse().unwrap();
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

fn set_row_slot(table: Table, row: SimpleRow, row_num: usize) {
    let page_num: usize = row_num / ROWS_PER_PAGE;

    table.pages[page_num]; // void* page =
    // if page is null
    // page = table->pages[page_num] = malloc(PAGE_SIZE);
    let row_offset: usize = row_num % ROWS_PER_PAGE;
    let byte_offset: usize = row_offset * ROW_SIZE;
    // return page + byte_offset;
}

fn set_row_slot(table: crate::Table, row: crate::SimpleRow) {
    set_row_slot(table, row, table.num_rows + 1);
}

fn execute_insert(statement: PreparedStatement, mut table: Table) -> ExecuteResult {
    if table.num_rows >= TABLE_MAX_ROWS {
        return ExecuteTableFull;
    }

    let to_insert: SimpleRow = statement.row;
    set_row_slot(table, to_insert, table.num_rows);

    table.num_rows += 1;
    ExecuteSuccess
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

struct SimpleRow {
    id: u32,
    username: String,
    email: String
}

enum StatementType {
    Invalid,
    Insert,
    Select
}

struct PreparedStatement {
    statement_type: StatementType,
    row: SimpleRow,
}

impl PreparedStatement {
    pub fn new() -> Self {
        let row = SimpleRow {
            id: 0,
            username: "".to_string(),
            email: "".to_string(),
        };

        PreparedStatement {
            statement_type: Invalid,
            row
        }
    }
}

struct Table {
    num_rows: usize,
    pages: [[SimpleRow; PAGE_SIZE]; TABLE_MAX_PAGES]
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
    PreparedStatementSyntaxError,
    PreparedStatementUnrecognized
}

enum ExecuteResult {
    ExecuteSuccess,
    ExecuteTableFull
}
