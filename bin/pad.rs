use failure::Fail;
use std::fs::{self};
use std::io;
use std::path::Path;
use weresocool_error::Error;

fn main() {
    match run() {
        Ok(_) => {}
        e => {
            for cause in Fail::iter_causes(&e.unwrap_err()) {
                println!("Failure caused by: {}", cause);
            }
        }
    }
}

#[allow(unused_variables)]
fn run() -> Result<(), Error> {
    visit_dirs(Path::new("./mocks"));

    Ok(())
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &Path) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            //dbg!(&path.is_dir());
            if path.is_dir() {
                dbg!(path);
            }
        }
    }
    Ok(())
}
