use std::{env, io};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;
use std::process::exit;

use crate::ExecuteResult::{ExecuteFailed, ExecuteSuccess, ExecuteTableFull};
use crate::MetaCommandResult::{MetaCommandSuccess, MetaCommandUnrecognized};
use crate::PreparedStatementResult::{PreparedStatementError, PreparedStatementSuccess, PreparedStatementSyntaxError, PreparedStatementUnrecognized};
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
    program_state.running = true;

    while program_state.running {
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
            program_state.running = false;
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
        if statement.row.id == 0 {
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

fn append_row_slot(table: &mut Table, row: SimpleRow) -> bool { // TODO make member function
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
    println!("({id}, {username}, {email})", id=row.id, username=row.get_username(), email=row.get_email())
}

fn print_prompt() {
    print!("db > ");
    io::stdout().flush().unwrap();
}

fn usage(exec_path: String) -> String {
    format!("Usage: {exec_path} <database_file_name>")
}

/*
struct Varchar {
    size: u8,
    value: Vec<char>
}

impl Varchar {
    pub fn new(size: u8) -> Self {
        Varchar {
            size,
            value: Vec::with_capacity(size as usize)
        }
    }

    fn set_value(&mut self, new_value: String) -> bool { // TODO add getters/setters https://stackoverflow.com/a/44879870/3875151
        if new_value.len() > self.size as usize {
            return false;
        }

        let mut index = 0;
        for chr in new_value.chars() {
            if self.value.len() <= index {
                self.value.push(chr);
            } else {
                self.value[index] = chr;
            }

            index += 1;
        }

        return true;
    }

    fn to_string(&self) -> String {
        self.value.iter().collect()
    }
}
*/

struct SimpleRow {
    id: u32,
    username: [char; USERNAME_SIZE],
    username_len: usize,
    email: [char; EMAIL_SIZE],
    email_len: usize
}

impl SimpleRow {
    pub fn new() -> Self {
        SimpleRow {
            id: 0,
            username: ['\0'; USERNAME_SIZE],
            username_len: 0,
            email: ['\0'; EMAIL_SIZE],
            email_len: 0
        }
    }

    fn get_username(&self) -> String {
        return String::from_iter(&self.username[0..self.username_len]);
    }

    fn set_username(&mut self, new_username: String) -> bool {
        if new_username.len() > self.username.len() {
            return false;
        }

        let username_bytes = new_username.as_bytes();

        for i in 0..self.username.len() {
            if i < new_username.len() {
                self.username[i] = username_bytes[i] as char;
            } else {
                self.username[i] = '\0';
            }
        }

        self.username_len = new_username.len();
        return true;
    }

    fn get_email(&self) -> String {
        return String::from_iter(&self.email[0..self.email_len]);
    }

    fn set_email(&mut self, new_email: String) -> bool {
        if new_email.len() > self.email.len() {
            return false;
        }

        let email_bytes = new_email.as_bytes();

        for i in 0..self.email.len() {
            if i < new_email.len() {
                self.email[i] = email_bytes[i] as char;
            } else {
                self.email[i] = '\0';
            }
        }

        self.email_len = new_email.len();
        return true;
    }

    fn get_id(&self) -> u32 {
        return self.id;
    }

    fn set_id(&mut self, new_id: u32) -> bool {
        self.id = new_id;
        return true;
    }
}

enum StatementType {
    Invalid,
    Insert,
    Select
}

struct ProgramState {
    running: bool
}

impl ProgramState {
    pub fn new() -> Self {
        ProgramState {
            running: false,
        }
    }
}

struct PreparedStatement {
    statement_type: StatementType,
    row: SimpleRow,
}

impl PreparedStatement {
    pub fn new() -> Self {
        let row = SimpleRow::new();

        PreparedStatement {
            statement_type: Invalid,
            row
        }
    }
}



struct Table {
    num_rows: usize,
    rows: Vec<SimpleRow>,
    file_ok: bool,
    file: File
}

impl Table {
    pub fn new(db_file_name: &String) -> Table {
        let file_existed = Path::new(db_file_name).exists();
        let db_file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(db_file_name);


        if ! file_existed {
            let res = db_file.as_ref().unwrap().set_len((PAGE_SIZE * TABLE_MAX_PAGES * ROW_SIZE / ROWS_PER_PAGE) as u64);
            if ! res.is_ok() {
                println!("Tried to create file {db_file_name}, could not!");
            }
        }

        if ! db_file.is_ok() {
            return Table {
                num_rows: 0,
                rows: Vec::with_capacity(0),
                file_ok: false,
                file: db_file.unwrap(),
            }
        }

        let mut new_table = Table {
            num_rows: 0,
            rows: Vec::with_capacity(TABLE_MAX_PAGES * PAGE_SIZE),
            file_ok: true,
            file: db_file.unwrap()
        };
        new_table.read_rows_from_file().expect("TODO: panic message");

        return new_table;
    }

    fn write_at_position(&self, buf: &Vec<u8>, position: u64) -> bool{
        let write = self.file.write_all_at(&buf, position);
        if ! write.is_ok() {
            println!("Problem writing to database file at position {}.", position);
            return false;
        }

        return true;
    }

    fn write_rows_to_file(&mut self) {
        if self.rows.len() == 0 {
            return;
        }
        // println!("Writing!");

        if !self.file_ok {
            println!("There was a problem when initially opening the database file, will not attempt to write to it!");
            return;
        }

        let mut rows_written = 0;

        for row in &self.rows {
            // We now write at the offset 10.
            // file.write_all_at(b"sushi", 10)?;
            let mut row_buf: Vec<u8> = vec![];
            row_buf.extend_from_slice(&row.id.to_le_bytes());
            self.write_at_position(&row_buf, (rows_written * ROW_SIZE + ID_OFFSET) as u64);

            row_buf = vec![];
            write!(row_buf, "{}", &row.get_username()).expect("Could not write to Vec buffer. Out of memory?");
            self.write_at_position(&row_buf, (rows_written * ROW_SIZE + USERNAME_OFFSET) as u64);

            row_buf = vec![];
            write!(row_buf, "{}", &row.get_email()).expect("Could not write to Vec buffer. Out of memory?");
            self.write_at_position(&row_buf, (rows_written * ROW_SIZE + EMAIL_OFFSET) as u64);

            rows_written += 1;
        }
    }

    fn seek_read(&mut self, offset: u64, buf: &mut Vec<u8>) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(buf)?;
        Ok(())
    }

    fn read_rows_from_file(&mut self) -> io::Result<()> {
        // println!("Reading!");

        let mut row_count = 0;
        loop {
            let mut v = vec![0; ID_SIZE];
            self.seek_read((row_count * ROW_SIZE + ID_OFFSET) as u64, &mut v)?;
            let id = u32::from_le_bytes(v.clone().try_into().expect("handle the error however you want"));
            if id == 0 {
                break;
            }

            v = vec![0; USERNAME_SIZE];
            self.seek_read((row_count * ROW_SIZE + USERNAME_OFFSET) as u64, &mut v)?;
            let username = String::from_utf8(v.clone()).unwrap_or("".to_string());

            v = vec![0; EMAIL_SIZE];
            self.seek_read((row_count * ROW_SIZE + EMAIL_OFFSET) as u64, &mut v)?;
            let email = String::from_utf8(v.clone()).unwrap_or("".to_string());

            let mut row = SimpleRow::new();

            row.set_id(id);

            row.set_username(username);
            row.set_email(email);



            // TODO move into one function
            self.rows.push(row);
            self.num_rows += 1;

            row_count += 1;
        }
        Ok(())
    }

    fn drop(&mut self) {
        self.write_rows_to_file();
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
    PreparedStatementSyntaxError,
    PreparedStatementUnrecognized,
    PreparedStatementError
}

enum ExecuteResult {
    ExecuteSuccess,
    ExecuteFailed,
    ExecuteTableFull
}
