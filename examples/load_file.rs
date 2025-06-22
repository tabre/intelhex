use intelhex::{IntelHexFile, util};
use util::display_file_info;

pub fn main() {
    match IntelHexFile::load_file("examples/example.hex") {
        Ok(file) => {
            display_file_info(&file, 5);
        },
        Err(err) => println!("{:?}", err)
    }
}
