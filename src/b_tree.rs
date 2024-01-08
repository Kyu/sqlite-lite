use std::fs::File;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;

use crate::{EMAIL_OFFSET, EMAIL_SIZE, ID_OFFSET, ID_SIZE, PAGE_SIZE, ROW_SIZE, ROWS_PER_PAGE, TABLE_MAX_PAGES, USERNAME_OFFSET, USERNAME_SIZE};

pub(crate) struct Table {
    pub(crate) num_rows: usize,
    pub(crate) rows: Vec<SimpleRow>,
    pub(crate) file_ok: bool,
    file: File
}

impl Table {
    pub(crate) fn new(db_file_name: &String) -> Table {
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

    pub(crate) fn drop(&mut self) {
        self.write_rows_to_file();
    }
}

pub(crate) struct SimpleRow {
    id: u32,
    username: [char; USERNAME_SIZE],
    username_len: usize,
    email: [char; EMAIL_SIZE],
    email_len: usize
}

impl SimpleRow {
    pub(crate) fn new() -> Self {
        SimpleRow {
            id: 0,
            username: ['\0'; USERNAME_SIZE],
            username_len: 0,
            email: ['\0'; EMAIL_SIZE],
            email_len: 0
        }
    }

    pub(crate) fn get_username(&self) -> String {
        // println!("{}", self.username_len);
        // let s = String::from_iter(&self.username[0..self.username_len]);
        // println!("{} {}", s, s.len());
        return String::from_iter(&self.username[0..self.username_len]);
    }

    pub(crate) fn set_username(&mut self, new_username: String) -> bool {
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

    pub(crate) fn get_email(&self) -> String {
        return String::from_iter(&self.email[0..self.email_len]);
    }

    pub(crate) fn set_email(&mut self, new_email: String) -> bool {
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

    pub(crate) fn get_id(&self) -> u32 {
        return self.id;
    }

    pub(crate) fn set_id(&mut self, new_id: u32) -> bool {
        self.id = new_id;
        return true;
    }
}

pub(crate) struct Cursor<'a> {
    pub(crate) table: &'a Table,
    pub(crate) row_num: usize,
    pub(crate) end_of_table: bool // Indicates a position one past the last element
}

impl Cursor<'_> {

    pub(crate) fn table_start(table: &mut Table) -> Cursor {
        let end_of: bool = table.num_rows == 0;

        Cursor {
            table,
            row_num: 0,
            end_of_table: end_of,
        }
    }

    pub(crate) fn table_end(table: &mut Table) -> Cursor {
        let num_rows = table.num_rows;
        Cursor {
            table,
            row_num: num_rows,
            end_of_table: true
        }
    }

    pub(crate) fn advance(&mut self) {
        self.row_num += 1;

        if self.row_num >= self.table.num_rows {
            self.end_of_table = true;
        }
    }
}