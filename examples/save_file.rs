use std::str::from_utf8;

use intelhex::{IntelHexFile, util};
use util::display_file_info;

pub fn main() { 
    match IntelHexFile::load(
        ":08A455002E2F5F6E6963655F45"
    ) {
        Ok(mut file) => {
            display_file_info(&file, 1);

            let file_path = format!(
                "{}.hex", 
                from_utf8(&file.records[0].data[..]).unwrap()
            );

            println!("Saving file: {} ...", file_path);

            let _ = file.save_file(&file_path);
        },
        Err(error) => {
            println!("{:?}", error);
            return
        }
    };
}
