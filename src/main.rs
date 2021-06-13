extern crate lib;
mod file;
mod repl;

use std::env;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let path = Path::new(&args[1]);

        let exists = std::path::Path::exists(path);
        if !exists {
            panic!("The file {} doesn't exist.", exists)
        }

        file::exec_file(path)?;
    } else {
        repl::start();
    }

    Ok(())
}
