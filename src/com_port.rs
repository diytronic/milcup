use std::{
    fmt,
    io::{ self, Write },
    boxed::Box,
};

use serialport::{
    prelude::*,
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "[{}]", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        // eprintln!("IO error {}", err);
        Error::Io(err)
    }
}

pub type ComPort = Box<dyn SerialPort>;

pub trait IOMethods {
    fn write_buf(&mut self, buf: Vec<u8>) -> Result<(), Error>;
    fn write_str(&mut self, buf: &'static str) -> Result<(), Error>;
    fn write_u32(&mut self, buf: u32) -> Result<(), Error>;
    fn read_buf(&mut self, len: usize) -> Result<Vec<u8>, Error>;
    fn read_str(&mut self, len: usize) -> Result<String, Error>;
    fn read_u32(&mut self) -> Result<u32, Error>;
    fn read_byte(&mut self) -> Result<u8, Error>;
}

impl IOMethods for ComPort {
    fn write_buf(&mut self, buf: Vec<u8>) -> Result<(), Error> {
        // println!("Write buf: {:0>2X?} {}", buf, String::from_utf8_lossy(&buf));
        debug!("Write buf: {:0>2X?}", buf);
        self.write(&buf)?;
        std::io::stdout().flush().unwrap();
        return Ok(());
    }

    fn write_str(&mut self, buf: &'static str) -> Result<(), Error> {
        return self.write_buf(buf.as_bytes().to_vec());
    }

    fn write_u32(&mut self, buf: u32) -> Result<(), Error> {
        return self.write_buf(buf.to_le_bytes().to_vec());
    }

    fn read_buf(&mut self, len: usize) -> Result<Vec<u8>, Error> {
        let mut buf: Vec<u8> = vec![0; len];
        self.read(buf.as_mut_slice())?;
        // println!("Read buf: {:0>2X?} {}", buf, std::str::from_utf8_unchecked(&buf));
        // println!("Read buf: {:0>2X?} {}", buf, String::from_utf8_lossy(&buf));
        debug!(" Read buf: {:0>2X?}", buf);
        return Ok(buf);
    }

    fn read_str(&mut self, len: usize) -> Result<String, Error> {
        let res = self.read_buf(len)?;
        return Ok(String::from_utf8_lossy(&res).to_string());
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        let res = self.read_buf(4)?;
        return Ok(u32::from_le_bytes([res[0], res[1], res[2], res[3]]));
    }

    fn read_byte(&mut self) -> Result<u8, Error> {
        let res = self.read_buf(1)?;
        return Ok(res[0]);
    }
}

