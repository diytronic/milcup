use std::{
    io::{
        self, 
        Write,  
        prelude::*, 
        BufReader,
        Error,
        ErrorKind,
    },
    time::Duration,
    boxed::Box,
};

use serialport::{
    prelude::*,
    SerialPortType,
};

pub fn check_port(mut port: Box<dyn SerialPort>) -> Result<String, Error> {
    let mut buf: Vec<u8> = vec![0; 512];
    // println!("{:?}", buf);
    match port.write(&buf) {
        Ok(_) => std::io::stdout().flush().unwrap(),
        Err(e) => return Err(Error::new(ErrorKind::Other, "Unable to write data")),
    }

    buf = vec![0; 3];
    match port.read(buf.as_mut_slice()) {
        Ok(t) => io::stdout().write_all(&buf[..t]).unwrap(),
        Err(e) => {
            buf = vec![0; 512];
            match port.write(&buf) {
                Ok(_) => std::io::stdout().flush().unwrap(),
                Err(e) => return Err(Error::new(ErrorKind::Other, "Unable to write data 2!")),
            }

            buf = vec![0; 3];
            match port.read(buf.as_mut_slice()) {
                Ok(t) => io::stdout().write_all(&buf[..t]).unwrap(),
                Err(e) => return Err(Error::new(ErrorKind::Other, "Unable to read 512 ones")),
            }
        }
    }

    return Ok("wow".to_string());
}
