/// Board interface commands
///
use std::{
    io::{
        self, 
        Error,
        ErrorKind,
    },
};

use crate::{
    com_port::{
        ComPortError,
        ComPortMethods,
        ComPort
    },
    hex:: {
        HexFile
    }
};

#[derive(Debug)]
pub enum FlashError {
    Io(io::Error),
    Com(ComPortError),
}

impl From<io::Error> for FlashError {
    fn from(err: io::Error) -> FlashError {
        FlashError::Io(err)
    }
}

impl From<ComPortError> for FlashError {
    fn from(err: ComPortError) -> FlashError {
        FlashError::Com(err)
    }
}

/// Check if com port alive
///
/// Send 512 zero bytes and check if port answer
///
pub fn check_port(port: &mut ComPort) -> Result<(), FlashError> {
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
pub fn set_baud_rate(port: &mut ComPort, baud_rate: u32) -> Result<(), FlashError> {
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

pub fn read_baud_rate(port: &mut ComPort) -> Result<Vec<u8>, FlashError> {
    port.write_buf(vec![0xD])?;
    let resp = port.read_buf(3)?;

    if resp != [0xD, 0xA, 0x3E] {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error setting baud rate")));
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
pub fn boot_load(port: &mut ComPort, data: HexFile) -> Result<(), FlashError> {
    println!("Writing boot code to {:0>8X?}", data.addr);
    println!("Data size is {} bytes", data.size);

    // set address where to put boot loader
    port.write_str("L")?;
    port.write_u32(data.addr)?; // address to load code to 
    port.write_u32(data.size)?; // size of data
    if port.read_str(1)? != "L" {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error setting baud rate")));
    }

    // write boot loader code file 1986_BOOT_UART.hex
    port.write_buf(data.buf)?;
    if port.read_str(1)? != "K" {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error writing boot code")));
    }
    
    // read and compare
    // TODO: read and check throught all the data
    port.write_str("Y")?;
    port.write_u32(data.addr)?;  // address to load code to 
    port.write_u32(0x8u32)?;     // not sure what is it

    let resp = port.read_buf(10)?; 
    if resp[0] != ('Y' as u8) && resp[9] != ('K' as u8) {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error reading written code")));
    }

    // run code
    port.write_str("R")?;
    port.write_u32(data.addr)?; // address to load code to 
    port.write_u32(data.size)?; // size of data
    if port.read_str(1)? != "R" {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error running boot code")));
    }
    
    return Ok(());
}

/// Read boot loader info 
///
/// Really this is a last 12 bytes of boot loader
/// Normally it should return 1986BOOTUART string
///
pub fn read_info(port: &mut ComPort) -> Result<String, FlashError> {
    port.write_str("I")?;
    let res = port.read_str(12)?;

    return Ok(res);
}

/// Erase
///
/// Full chip erase
///
pub fn erase(port: &mut ComPort) -> Result<(), FlashError> {
    // set address where to put boot loader
    port.write_str("E")?;
    // pause 1000
    if port.read_str(1)? != "E" {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error setting baud rate")));
    }

    let addr = port.read_u32()?;
    let data = port.read_u32()?;

    if (addr == 0x08020000) && (data == 0xffffffff) {
        return Ok(());
    } else {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, format!("Chip erase fail addr=0x{:0>4X?} data={:0>4X?}", addr, data))));
    }
}

/// Upload real firmware to flash
///
pub fn program(port: &mut ComPort, data: HexFile) -> Result<(), FlashError> {
    println!("Writing program code to {:0>8X?}", data.addr);
    println!("Data size is {} bytes", data.size);

    // set address where to put program
    port.write_str("A")?;
    port.write_u32(data.addr)?; // address to load code to 
    if port.read_byte()? != 0x08 {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error sending A command")));
    }

    // write code by 256 byte length chunks
    let mut iter = data.buf.chunks(256);
    while match iter.next() { 
        None => false,
        Some(wbuf) => {
          let res =  write_program_chunk(port, wbuf)?;
          res
        }
    } {};

    //
    // // read and compare
    // // TODO: read and check throught all the data
    // port.write_str("Y")?;
    // port.write_u32(data.addr)?;  // address to load code to 
    // port.write_u32(0x8u32)?;     // not sure what is it
    //
    // let resp = port.read_buf(10)?; 
    // if resp[0] != ('Y' as u8) && resp[9] != ('K' as u8) {
    //     return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error reading written code")));
    // }
    
    return Ok(());
}

/// calculate checksum of data chunk
///
/// checksum is nothing more than just a sum of bytes summed with overflow
///
fn checksum(buf : &[u8]) -> u8 {
  return  buf.iter().fold(0, |acc, &x| acc.wrapping_add(x));
}

fn write_program_chunk(port: &mut ComPort, buf : &[u8]) ->  Result<bool, FlashError>  {
    println!("Writing chunk");
    port.write_str("P")?;
    port.write_buf(buf.to_vec())?;

    // if not a full 256 bytes buffer - fill the rest with 0xFF
    let diff = 256 - buf.len();
    if diff > 0 {
        println!("Write rest {} of bytes", diff);
        port.write_buf(vec![0x00; diff])?;
    }

    let sum : u8 = checksum(buf);
    let rsum : u8 = port.read_byte()?; 

    println!("Checking control sum {:0>2X?} == {:0>2X?}", sum, rsum);
    if rsum != sum {
        return Err(FlashError::Io(Error::new(ErrorKind::Other, "Error control sum")));
    }

    Ok(true)
}
