use std::{
    io::{
        self, 
        Write,  
        prelude::*, 
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
pub enum FlashError {
    Io(io::Error),
}

impl From<io::Error> for FlashError {
    fn from(err: io::Error) -> FlashError {
        FlashError::Io(err)
    }
}

pub fn check_port(port: &mut Box<dyn SerialPort>) -> Result<(), FlashError> {
    // write 512 zero bytes
    let mut buf: Vec<u8> = vec![0; 512];
    port.write(&buf)?;
    std::io::stdout().flush().unwrap();

    // try to read 3 ones
    buf = vec![0; 3];
    port.read(buf.as_mut_slice())?;

    // println!("Read {:?}", buf);

    return Ok(());
}
