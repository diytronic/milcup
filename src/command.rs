/// Board interface commands
///
use std::io;
use std::fmt;

use indicatif::ProgressIterator;

use crate::{
    com_port::{
        self,
        ComPort,
        IOMethods,
    },
    firmware::HexFile,
};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    SerialPort(com_port::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "[{}]", err),
            Error::SerialPort(ref err) => write!(f, "Serial port error: {}", err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<com_port::Error> for Error {
    fn from(err: com_port::Error) -> Error {
        Error::SerialPort(err)
    }
}

/// Check if com port alive
///
/// Send 512 zero bytes and check if port answer
///
pub fn check_port(port: &mut ComPort) -> Result<(), Error> {
    port.write_buf(vec![0; 512])?; // write 512 zero bytes
    port.read_buf(3)?; // try to read 3 ones

    return Ok(());
}

/// Set UART baud rate 
///
/// Initial connection happens on speed of 9600 so we can setup better
/// baud rate with this function.
///
/// After success reconnect required with given baud rate
pub fn set_baud_rate(port: &mut ComPort, baud_rate: u32) -> Result<(), Error> {
    // write 'B' b1 b2 b3 0x0
    // where b1 low byte of baud rate 
    //       b3 hight byte of baud rate value
    port.write_str("B")?;
    port.write_u32(baud_rate)?;
    port.write_buf(vec![0xD])?;

    // try to read any response value
    port.read_buf(1)?;

    return Ok(());
}

pub fn read_baud_rate(port: &mut ComPort) -> Result<Vec<u8>, Error> {
    port.write_buf(vec![0xD])?;

    let resp = port.read_buf(3)?;
    if resp != [0xD, 0xA, 0x3E] {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error setting baud rate")));
    }

    return Ok(resp);
}

/// Upload UART boot loader to board RAM
///
/// Custom boot loader loaded into RAM to provide additional capabilities
/// of loading real program code to flash memory
///
/// Boot loader uploaded to base address 0x20000000
///
pub fn boot_load(port: &mut ComPort, data: HexFile) -> Result<(), Error> {
    // println!("Writing boot code to {:0>8X?}", data.addr);
    // println!("Data size is {} bytes", data.size);

    // set address where to put boot loader
    port.write_str("L")?;
    port.write_u32(data.addr)?; // address to load code to 
    port.write_u32(data.size)?; // size of data
    if port.read_str(1)? != "L" {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error setting baud rate")));
    }

    // write boot loader code file 1986_BOOT_UART.hex
    port.write_buf(data.buf)?;
    if port.read_str(1)? != "K" {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error writing boot code")));
    }
    
    // read and compare
    // TODO: read and check throught all the data
    port.write_str("Y")?;
    port.write_u32(data.addr)?;  // address to load code to 
    port.write_u32(0x8u32)?;     // not sure what is it

    let resp = port.read_buf(10)?; 
    if resp[0] != ('Y' as u8) && resp[9] != ('K' as u8) {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error reading written code")));
    }

    // run code
    port.write_str("R")?;
    port.write_u32(data.addr)?; // address to load code to 
    port.write_u32(data.size)?; // size of data
    if port.read_str(1)? != "R" {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error running boot code")));
    }
    
    return Ok(());
}

/// Read boot loader info 
///
/// Really this is a last 12 bytes of boot loader
/// Normally it should return 1986BOOTUART string
///
pub fn read_info(port: &mut ComPort) -> Result<String, Error> {
    port.write_str("I")?;
    let res = port.read_str(12)?;

    return Ok(res);
}

/// Erase
///
/// Full chip erase
///
pub fn erase(port: &mut ComPort) -> Result<(), Error> {
    // set address where to put boot loader
    port.write_str("E")?;
    // pause 1000
    if port.read_str(1)? != "E" {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error setting baud rate")));
    }

    let addr = port.read_u32()?;
    let data = port.read_u32()?;

    if (addr == 0x08020000) && (data == 0xffffffff) {
        return Ok(());
    } else {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, format!("Chip erase fail addr=0x{:0>4X?} data={:0>4X?}", addr, data))));
    }
}

/// Upload real firmware to flash
///
pub fn program(port: &mut ComPort, data: &HexFile) -> Result<(), Error> {
    // println!("Writing program code to {:0>8X?}", data.addr);
    // println!("Data size is {} bytes", data.size);

    // set address where to put program
    port.write_str("A")?;
    port.write_u32(data.addr)?; // address to load code to 
    if port.read_byte()? != 0x08 {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error setting flash address")));
    }

    // write code by 256 byte length chunks
    // let mut iter = data.buf.chunks(256);
    // while match iter.next() { 
    //     None => false,
    //     Some(wbuf) => {
    //       let res =  write_program_chunk(port, wbuf)?;
    //       res
    //     }
    // } {};

    // write code by 256 byte length chunks
    let iter = data.buf.chunks(256);
    for wbuf in iter.progress() { 
       write_program_chunk(port, wbuf)?;
    };
    
    return Ok(());
}

/// Verify uploaded firmware
///
pub fn verify(port: &mut ComPort, data: &HexFile) -> Result<(), Error> {
    // set address where to put program
    port.write_str("A")?;
    port.write_u32(data.addr)?; // address to load code to 
    if port.read_byte()? != 0x08 {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error setting flash address")));
    }

    // // write code by 256 byte length chunks
    // let mut iter = data.buf.chunks(256);
    // while match iter.next() { 
    //     None => false,
    //     Some(wbuf) => {
    //       let res =  verify_program_chunk(port, wbuf)?;
    //       res
    //     }
    // } {};

    let iter = data.buf.chunks(256);
    for wbuf in iter.progress() { 
       verify_program_chunk(port, wbuf)?;
    };
    
    return Ok(());
}


/// calculate checksum of data chunk
///
/// checksum is nothing more than just a sum of bytes summed with overflow
///
fn checksum(buf : &[u8]) -> u8 {
  return  buf.iter().fold(0, |acc, &x| acc.wrapping_add(x));
}

fn write_program_chunk(port: &mut ComPort, buf : &[u8]) ->  Result<bool, Error>  {
    // if not a full 256 bytes buffer - fill the rest with 0xFF
    let mut wbuf = buf.to_vec().clone();

    let diff = 256 - buf.len(); // number of bytes up to 256
    if diff > 0 {
        debug!("Add {} of zero bytes up to 256 bytes length", diff);
        wbuf.append(&mut vec![0x00; diff]);
    }

    debug!("Writing chunk");
    port.write_str("P")?;
    port.write_buf(wbuf.to_vec())?;

    let sum : u8 = checksum(&wbuf);    // calcuate by written data
    let rsum : u8 = port.read_byte()?; // return from UART

    debug!("Checking control sum {:0>2X?} == {:0>2X?}", sum, rsum);
    if rsum != sum {
        return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error checking control sum")));
    }

    Ok(true)
}

fn verify_program_chunk(port: &mut ComPort, buf : &[u8]) ->  Result<bool, Error>  {
    debug!("Verify chunk");
    
    // check 32 chunks of 8 bytes blocks
    // let mut iter = buf.chunks(8);
    // while match iter.next() { 
    //     None => false,
    //     Some(vbuf) => {
    //         port.write_str("V")?;
    //         // std::thread::sleep(Duration::from_secs(1));
    //         let rbuf = port.read_buf(8)?;
    //         debug!("Verify -> {:0>2X?}", vbuf);
    //         debug!("       <- {:0>2X?}", rbuf);
    //
    //         let buf_len = vbuf.len();
    //         if rbuf[0..buf_len] != vbuf[0..buf_len] {
    //             return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error verify block")));
    //         }
    //
    //         true
    //     }
    // } {};

    let iter = buf.chunks(8);
    for vbuf in iter { 
        port.write_str("V")?;
        // std::thread::sleep(Duration::from_secs(1));
        let rbuf = port.read_buf(8)?;
        debug!("Verify -> {:0>2X?}", vbuf);
        debug!("       <- {:0>2X?}", rbuf);

        let buf_len = vbuf.len();
        if rbuf[0..buf_len] != vbuf[0..buf_len] {
            return Err(Error::Io(io::Error::new(io::ErrorKind::Other, "Error verify block")));
        }
    }

    debug!("");

    Ok(true)
}
