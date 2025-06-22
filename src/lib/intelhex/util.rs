use std::cmp::min;

use crate::file::IntelHexFile;

pub fn display_file_info(file: &IntelHexFile, mut n: usize) {
    let n_records = file.records.len();
    n = min(n, n_records);

    println!();
    println!("File:     {}", file.get_path());
    println!("Size:     {} bytes", file.size);
    println!("Bin Size: {} bytes", file.binary_size());
    println!("Records:  {}", n_records);
    println!();

    for i in 0..n {
        let record = &file.records[i];

        println!("\tIndex:    {}", i);
        println!("\tType:     0x{:X} ({:?})", record.rtype.to_u8(), record.rtype);
        println!("\tAddr:     0x{:X} ({})", record.addr, record.addr);
        println!("\tData:     0x{:X} ({:?})", record.data, record.data);
        println!("\tChecksum: 0x{:X} ({})", record.checksum, record.checksum);
        println!();
    }
}

pub fn twos_comp(input: u64) -> u8 {
    return (input as i64 * -1i64) as u8;
}
