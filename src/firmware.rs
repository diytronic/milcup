use std::{
    fs,
    fmt,
    path::Path,
    io::{
        self,
    },
};

use ihex::{
    reader::*,
    record::Record,
};

pub struct HexFile {
    pub addr: u32,
    pub size: u32,
    pub buf: Vec<u8>
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            // CliError::NotFound => write!(f, "No matching cities with a \
            //                                  population were found."),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<Error> for std::io::Error {
    fn from(err : Error) -> std::io::Error {
        return std::io::Error::new(std::io::ErrorKind::Other, "Firmware Error");
    }
}

pub fn read_hex_file(filename: &Path) -> Result<HexFile, Error> {
    let data = fs::read_to_string(filename).expect("no such file");
    let firmware = parse_hex_buffer(&data)?;
    return Ok(firmware);
}

pub fn parse_hex_buffer(data: &str) -> Result<HexFile, Error> {
    // check addresses
    let mut reader = Reader::new(data);
    let base_addr : u16 = reader.fold(0, |curr_addr, rec| 
        match rec {
            Ok(Record::ExtendedLinearAddress(addr)) => curr_addr + addr,
            Ok(Record::ExtendedSegmentAddress(addr)) => curr_addr + addr,
            _ => curr_addr,
        }
    );

    // file base addr
    reader = Reader::new(data);
    let data_addr : u16 = reader.filter(|rec| 
        match rec {
            Ok(Record::Data {offset: _, value: _}) => true,
            _ => false
        }
    ).take(1).map(|rec| 
        match rec {
            Ok(Record::Data {offset, value: _}) => offset,
            _ => 0
        }
    ).last().unwrap();

    // file max addr
    reader = Reader::new(data);
    let data_size : u32 = reader.fold(0u32, |curr_len, rec| 
        match rec {
            Ok(Record::Data {offset: _, value}) => (value.len() as u32) + curr_len,
            _ => curr_len,
        }
    );

    println!("  Base addr: 0x{:0>6X?}", base_addr);
    println!("  Data addr: 0x{:0>6X?}", data_addr);
    // println!("   Max addr: 0x{:0>6X?}", file_max_offset);
    println!("Data length: 0x{:0>6X?}", data_size);
    println!("           0x08000000");
    println!("           0x20000000");

    // if file_data_len > max_size {
    //     return Err("Oversize".to_string());
    // }
    //
    // data
    reader = Reader::new(data);
    let file_data = reader.fold(Vec::<u8>::new(), |mut data, rec| 
        match rec {
            Ok(Record::Data {offset: _, mut value}) => {
                data.append(&mut value);
                data
            },
            _ => data,
        }
    );

    reader = Reader::new(&data);
    let x = reader.map(|r| 
        match r {
            Ok(Record::Data {offset, value}) => println!("Data: offset: 0x{:X?} len: {}", offset, value.len()),
            Ok(Record::ExtendedLinearAddress(addr)) => println!("Extended Linear address: 0x{:X?}", addr),
            Ok(Record::StartLinearAddress(addr)) => println!("Start Linear address: {}", addr),
            Ok(Record::ExtendedSegmentAddress(addr)) => println!("Extended segment address: 0x{:X?}", addr),
            Ok(Record::StartSegmentAddress {cs, ip}) => println!("Start segment address: {} {}", cs, ip),
            Ok(Record::EndOfFile) => println!("END"),
            // Ok(_) => eprintln!("Unknown record"),
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

    let base_addr_buf = base_addr.to_be_bytes();
    let data_addr_buf = data_addr.to_be_bytes();
    let addr_buf : [u8; 4] = [base_addr_buf[0], base_addr_buf[1], data_addr_buf[0], data_addr_buf[1]];

    let hex_file = HexFile {
        addr: u32::from_be_bytes(addr_buf),
        size: data_size,
        buf: file_data,
    };

    println!("Load addr: 0x{:0>6X?}", hex_file.addr);

    return Ok(hex_file);
}


