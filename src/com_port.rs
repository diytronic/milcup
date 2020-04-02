use std::{
    io::{
        self, 
        Write,  
        // prelude::*, 
        // BufReader,
        // Error,
        // ErrorKind,
    },
    // time::Duration,
    boxed::Box,
};

use serialport::{
    prelude::*,
    // SerialPortType,
};

#[derive(Debug)]
pub enum ComPortError {
    Io(io::Error),
}

impl From<io::Error> for ComPortError {
    fn from(err: io::Error) -> ComPortError {
        ComPortError::Io(err)
    }
}

pub type ComPort = Box<dyn SerialPort>;

pub trait ComPortMethods {
    fn write_buf(&mut self, buf: Vec<u8>) -> Result<(), ComPortError>;
    fn write_str(&mut self, buf: &'static str) -> Result<(), ComPortError>;
    fn write_u32(&mut self, buf: u32) -> Result<(), ComPortError>;
    fn read_buf(&mut self, len: usize) -> Result<Vec<u8>, ComPortError>;
    fn read_str(&mut self, len: usize) -> Result<String, ComPortError>;
    fn read_u32(&mut self) -> Result<u32, ComPortError>;
}

impl ComPortMethods for ComPort {
    fn write_buf(&mut self, buf: Vec<u8>) -> Result<(), ComPortError> {
        println!("Write buf: {:0>2X?} {}", buf, String::from_utf8_lossy(&buf));
        self.write(&buf)?;
        std::io::stdout().flush().unwrap();
        return Ok(());
    }

    fn write_str(&mut self, buf: &'static str) -> Result<(), ComPortError> {
        return self.write_buf(buf.as_bytes().to_vec());
    }

    fn write_u32(&mut self, buf: u32) -> Result<(), ComPortError> {
        return self.write_buf(buf.to_le_bytes().to_vec());
    }

    fn read_buf(&mut self, len: usize) -> Result<Vec<u8>, ComPortError> {
        let mut buf: Vec<u8> = vec![0; len];
        self.read(buf.as_mut_slice())?;
        // println!("Read buf: {:0>2X?} {}", buf, std::str::from_utf8_unchecked(&buf));
        println!("Read buf: {:0>2X?} {}", buf, String::from_utf8_lossy(&buf));
        return Ok(buf);
    }

    fn read_str(&mut self, len: usize) -> Result<String, ComPortError> {
        let res = self.read_buf(len)?;
        return Ok(String::from_utf8_lossy(&res).to_string());
    }

    fn read_u32(&mut self) -> Result<u32, ComPortError> {
        let res = self.read_buf(4)?;
        return Ok(u32::from_le_bytes([res[0], res[1], res[2], res[3]]));
    }
}

