# SQLite-lite  

## Building  
- `cargo build`  
A binary will be available in the `target/debug` directory  


## Running  
You can run the binary using `cargo run <db_file_name>`  
You can also copy the binary from the Building step, and run it using `./sqlite_lite <db_file_name>`  


## Testing  
In `tests`, run:  
- `python -m pip install -r requirements.txt`  
- `python -m ivoire main.py`  

Tests are inconsistent at the moment due to how stdin/stdout are handled. 
The `prints error message when table is full` tests also takes 7 minutes to run, so it is usually commented out.  


## Sources  
- [SQLite Docs](https://www.sqlite.org/arch.html)  
- [SQLite Designs](https://www.sqlite.org/zipvfs/doc/trunk/www/howitworks.wiki)  
- [Simple Database Tutorial](https://github.com/cstack/db_tutorial)  


## Disclaimer  
I don't have much experience in C, and this is my first program in Rust that isn't the tutorial, so please mind any wrong 
API usage.

## License  
This project uses the [MIT License](LICENSE)