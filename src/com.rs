use std::{
    io::{
        self, 
        Write,  
        prelude::*, 
        // BufReader,
        Error,
        ErrorKind,
    },
    // time::Duration,
    boxed::Box,
};

use serialport::{
    prelude::*,
    // SerialPortType,
};

// use hex::HexFile;

#[derive(Debug)]
pub enum FlashError {
    Io(io::Error),
}

impl From<io::Error> for FlashError {
    fn from(err: io::Error) -> FlashError {
        FlashError::Io(err)
    }
}

/// Function to check if port alive
///
/// Send 512 zero bytes and check if port answer
///
pub fn check_port(port: &mut Box<dyn SerialPort>) -> Result<(), FlashError> {
    write_buf(port, vec![0; 512])?; // write 512 zero bytes
    read_buf(port, 3)?; // try to read 3 ones

    return Ok(());
}

/// Set UART baud rate 
///
/// Initial connection happens on speed of 9600 so we can setup better
/// baud rate with this function.
///
/// After success reconnect required with given baud rate
pub fn set_baud_rate(port: &mut Box<dyn SerialPort>, baud_rate: u32) -> Result<(), FlashError> {
    // write 'B' b1 b2 b3 0x0
    // where b1 low byte of baud rate 
    //       b3 hight byte of baud rate value
    let bb : [u8; 4] = baud_rate.to_le_bytes();
    write_buf(port, vec!['B' as u8])?;
    write_buf(port, baud_rate.to_le_bytes().to_vec())?;
    write_buf(port, vec![0xD])?;

    // try to read any response value
    read_buf(port, 1)?;

    return Ok(());
}

pub fn read_baud_rate(port: &mut Box<dyn SerialPort>) -> Result<Vec<u8>, FlashError> {
    write_buf(port, vec![0xD])?;
    let resp = read_buf(port, 3)?;

    // println!("{:0>2X?}", res);
    // println!("{:0>2X?}", vec!(0xD, 0xA, 0x3E));

    if resp != vec!(0xD, 0xA, 0x3E) {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error setting baud rate")));
    }

    return Ok(resp);
}

/// Load UART boot loader
///
/// Custom boot loader loaded into RAM to provide additional capabilities
/// of loading real program code to flash memory
///
/// Boot loader uploaded to base address 0x20000000
///
pub fn boot_load(port: &mut Box<dyn SerialPort>, data: crate::hex::HexFile) -> Result<(), FlashError> {
    println!("Writing boot code to {:0>8X?}", data.addr);
    println!("Data size is {} bytes", data.size);

    write_buf(port, vec!('L' as u8))?;
    write_buf(port, data.addr.to_le_bytes().to_vec())?; // address to load code to 
    write_buf(port, data.size.to_le_bytes().to_vec())?;    // size of data

    if read_buf(port, 1)? != "L".as_bytes() {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error setting baud rate")));
    }

    // write boot loader code 1986_BOOT_UART.hex
    write_buf(port, data.buf)?;
    if read_buf(port, 1)? != "K".as_bytes() {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error writing boot code")));
    }
    
    // read and compare
    // TODO: read and check throught all the data
    write_buf(port, vec!('Y' as u8))?;
    write_buf(port, data.addr.to_le_bytes().to_vec())?;  // address to load code to 
    write_buf(port, (0x8 as u32).to_le_bytes().to_vec())?; // not sure what is it

    let resp = read_buf(port, 10)?; 
    if resp[0] != ('Y' as u8) && resp[9] != ('K' as u8) {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error reading written code")));
    }

    // run code
    write_buf(port, vec!('R' as u8))?;
    write_buf(port, data.addr.to_le_bytes().to_vec())?; // address to load code to 
    write_buf(port, data.size.to_le_bytes().to_vec())?;    // size of data

    if read_buf(port, 1)? != "R".as_bytes() {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error running boot code")));
    }
    
    return Ok(());
}

pub fn read_info(port: &mut Box<dyn SerialPort>) -> Result<String, FlashError> {
    write_buf(port, vec!('I' as u8));
    let res = read_buf(port, 12)?;

    return Ok(String::from_utf8_lossy(&res).to_string());
}

// ------------------------

fn write_buf(port: &mut Box<dyn SerialPort>, buf: Vec<u8>) -> Result<(), FlashError> {
    println!("Write buf: {:0>2X?} {}", buf, String::from_utf8_lossy(&buf));
    port.write(&buf)?;
    std::io::stdout().flush().unwrap();
    return Ok(());
}

fn read_buf(port: &mut Box<dyn SerialPort>, len: usize) -> Result<Vec<u8>, FlashError> {
    let mut buf: Vec<u8> = vec![0; len];
    port.read(buf.as_mut_slice())?;
    // println!("Read buf: {:0>2X?} {}", buf, std::str::from_utf8_unchecked(&buf));
    println!("Read buf: {:0>2X?} {}", buf, String::from_utf8_lossy(&buf));
    return Ok(buf);
}

