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

#[derive(Debug)]
pub enum FlashError {
    Io(io::Error),
}

impl From<io::Error> for FlashError {
    fn from(err: io::Error) -> FlashError {
        FlashError::Io(err)
    }
}

pub fn check_port(port: &mut Box<dyn SerialPort>) -> Result<(), FlashError> {
    write_buf(port, vec![0; 512])?; // write 512 zero bytes
    read_buf(port, 3)?; // try to read 3 ones

    return Ok(());
}

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

pub fn read_baud_rate(port: &mut Box<dyn SerialPort>) -> Result<(), FlashError> {
    write_buf(port, vec![0xD])?;
    let res = read_buf(port, 3)?;

    println!("{:0>2X?}", res);
    println!("{:0>2X?}", vec!(0xD, 0xA, 0x3E));

    if res != vec!(0xD, 0xA, 0x3E) {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error setting baud rate")));
    }

    // println!("Read {:?}", buf);

    return Ok(());
}

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

pub fn boot_load(port: &mut Box<dyn SerialPort>) -> Result<(), FlashError> {
    write_buf(port, vec!('L' as u8));
    return Ok(());
}

pub fn read_info(port: &mut Box<dyn SerialPort>) -> Result<(), FlashError> {
    write_buf(port, vec!('I' as u8));
    let res = read_buf(port, 12)?;

    return Ok(());
}
