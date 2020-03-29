use std::{
    fs,
    cmp,
    io::{prelude::*, BufReader},
    path::Path,
};

use ihex::{
    reader::*,
    record::Record,
};

pub struct HexFile {
    pub dwadrbot: u32,
    pub ilboot: u32,
    pub size: u16,
    pub buffer: Vec<u8>
}

pub fn read_hex_file(max_size: usize, filename: &Path) -> Result<HexFile, String> {
    let data = fs::read_to_string(filename).expect("no such file");
    let mut reader = Reader::new(&data);

    // check addresses
    reader = Reader::new(&data);
    let file_base_addr = reader.fold(0, |curr_addr, rec| 
        match rec {
            Ok(Record::ExtendedLinearAddress(addr)) => curr_addr + addr,
            Ok(Record::ExtendedSegmentAddress(addr)) => curr_addr + addr,
            _ => curr_addr,
        }
    );

    // file max addr
    reader = Reader::new(&data);
    let file_max_offset = reader.fold(0, |curr_offset, rec| 
        match rec {
            Ok(Record::Data {offset, value}) => cmp::max(offset, curr_offset),
            _ => curr_offset,
        }
    );

    // file max addr
    reader = Reader::new(&data);
    let file_data_len = reader.fold(0, |curr_len, rec| 
        match rec {
            Ok(Record::Data {offset, value}) => value.len() + curr_len,
            _ => curr_len,
        }
    );

    println!(" Write addr: 0x{:0>6X?}", file_base_addr);
    println!("   Max addr: 0x{:0>6X?}", file_max_offset);
    println!("Data length: 0x{:0>6X?}", file_data_len);
    println!("Max  length: 0x{:0>6X?}", max_size);

    if file_data_len > max_size {
        return Err("Oversize".to_string());
    }

    reader = Reader::new(&data);
    let x = reader.map(|r| 
        match r {
            Ok(Record::Data {offset, value}) => println!("Data: offset: 0x{:X?} len: {}", offset, value.len()),
            Ok(Record::ExtendedLinearAddress(addr)) => println!("Extended Linear address: 0x{:X?}", addr),
            Ok(Record::StartLinearAddress(addr)) => println!("Start Linear address: {}", addr),
            Ok(Record::ExtendedSegmentAddress(addr)) => println!("Extended segment address: 0x{:X?}", addr),
            Ok(Record::StartSegmentAddress {cs, ip}) => println!("Start segment address: {} {}", cs, ip),
            Ok(Record::EndOfFile) => println!("END"),
            Ok(rec) => eprintln!("Unknown record"),
            Err(err) => eprintln!("{}", err)
        }
    );

         println!("Reader count: {}", x.count());

    // let lines = buf
    //     .lines()
    //     // .map(|str| Record::from_record_string(str))
    //     // .collect();
    //     .collect::<Vec<Result<Record>>>();
    //
    // println!("Lines: {}", lines.join("\n"));

    let hex_file = HexFile {
        dwadrbot: 1,
        ilboot: 2,
        size: 3,
        buffer: vec![1, 2, 3],
    };

    return Ok(hex_file);
}


