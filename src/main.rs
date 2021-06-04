use structopt::StructOpt;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;
// use std::fmt;
// use std::io;

use anyhow::{Context, Result, bail};

#[macro_use]
extern crate log;
extern crate env_logger;

mod firmware;
mod command;
mod com_port;

// Baud rate 
// 9600,19200,57600,115200

#[derive(StructOpt)]
#[structopt(about = "Milandr 1986 firmware uploader", rename_all = "kebab-case")]
struct Cli {
    #[structopt(default_value = "auto", short = "p", long = "port")]
    port_name: String,
    #[structopt(default_value = "115200", short = "b", long = "baud")]
    baud_rate: u32,
    // #[structopt(default_value = true, short = "p", long = "program")]
    // program: bool,
    // #[structopt(default_value = true, short = "e", long = "erase")]
    // erase: bool,
    // #[structopt(default_value = true, short = "v", long = "verify")]
    // verify: bool,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

/// Try to find available port automatically
///
/// Rule is pretty simple - if we have a single USB-COM port - use it.
/// In other cases - no ports available or more than 1 port - cause an error and prompt
/// to specify port explicitly
///
fn probe_port() -> Result<String, anyhow::Error> {
    let ports = serialport::available_ports()?;
    let mdr_ports = ports // we need only USB-COM ports
        .into_iter()
        .filter(|port| match &port.port_type {
            SerialPortType::UsbPort(info) => {
                println!("    Type: USB");
                println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                println!( "     Serial Number: {}",
                    info.serial_number.as_ref().map_or("", String::as_str)
                );
                println!( "      Manufacturer: {}",
                    info.manufacturer.as_ref().map_or("", String::as_str)
                );
                println!( "           Product: {}",
                    info.product.as_ref().map_or("", String::as_str)
                );
                true
            },
            _ => false
        }) 
        .collect::<Vec<SerialPortInfo>>();

    return match mdr_ports.len() {
        0 => bail!("There are no available COM ports found"),
        1 => Ok(mdr_ports.last().unwrap().port_name.clone()),
        n => {
            let names = mdr_ports.iter().map(|port| port.port_name.clone()).collect::<String>();
            bail!("{} COM ports found choose right one with --port key {}", n, names);
        }
    };
}

// pub enum AppError {
//     Io(io::Error),
//     SerialPort(serialport::Error),
//     CommandError(command::Error),
//     FirmwareError(firmware::Error),
// }
//
// impl From<serialport::Error> for AppError {
//     fn from(err: serialport::Error) -> AppError {
//         AppError::SerialPort(err)
//     }
// }
//
// impl From<command::Error> for AppError {
//     fn from(err: command::Error) -> AppError {
//         AppError::CommandError(err)
//     }
// }
//
// impl From<firmware::Error> for AppError {
//     fn from(err: firmware::Error) -> AppError {
//         AppError::FirmwareError(err)
//     }
// }
//
// impl fmt::Display for AppError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             AppError::Io(ref err) => write!(f, "IO error: {}", err),
//             AppError::SerialPort(ref err) => write!(f, "Serial port error: {}", err),
//             AppError::CommandError(ref err) => write!(f, "Command error: {}", err),
//             AppError::FirmwareError(ref err) => write!(f, "Firmware error: {}", err),
//         }
//     }
// }
//
// impl fmt::Debug for AppError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match *self {
//             AppError::Io(ref err) => write!(f, "IO error: {}", err),
//             AppError::SerialPort(ref err) => write!(f, "Serial port error: {}", err),
//             AppError::CommandError(ref err) => write!(f, "Command error: {}", err),
//             AppError::FirmwareError(ref err) => write!(f, "Firmware error: {}", err),
//         }
//     }
// }

impl std::error::Error for command::Error {
    fn description(&self) -> &str {
        "My custom error message"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl std::error::Error for firmware::Error {
    fn description(&self) -> &str {
        "My custom error message"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

fn main() -> Result<()> {
    env_logger::init();
    warn!("[root] warn");
    info!("[root] info");
    debug!("[root] debug");

    let args = Cli::from_args();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(3000);
    settings.baud_rate = 9600; // initial baud rate
    
    let port_name = if args.port_name == "auto" {
        probe_port().with_context(|| format!("Probe com port"))?
    } else {
        args.port_name
    };

    // println!("Discovered port {}", port_name);

    let mut port = serialport::open_with_settings(&port_name, &settings)
        .with_context(|| format!("Open COM port with default baud rate 9600"))?;

    // println!("Checking port");
    command::check_port(&mut port)
        .with_context(|| format!("Check COM port availability"))?;

    // println!("Set baud rate {}", args.baud_rate);
    command::set_baud_rate(&mut port, args.baud_rate)
        .with_context(|| format!("Set baud rate"))?;
        // .with_context(|| format!("Set baud rate {}", args.baud_rate))?;

    std::mem::drop(port); // close port 

    settings.baud_rate = args.baud_rate; // set new baud rate

    // and reopen with new baud rate
    let mut port = serialport::open_with_settings(&port_name, &settings)
        .with_context(|| format!("Reopen port with new baud rate"))?;
        // .with_context(|| format!("Reopen port with new baud rate {}", args.baud_rate))?;
    // println!("Open port with baud rate {}", settings.baud_rate);

    // println!("Read baud rate");
    command::read_baud_rate(&mut port)
        .with_context(|| format!("Read baud rate settings"))?;

    // println!("Boot load");

    let boot_loader = include_str!("../firmware/1986_BOOT_UART.hex");
    let hex_file = firmware::parse_hex_buffer(boot_loader)
        .with_context(|| format!("Parse boot loader code"))?;

    // println!("Hex buffer length {}", hex_file.buf.len());

    command::boot_load(&mut port, hex_file)
        .with_context(|| format!("Load boot loader code to board RAM"))?;
    // println!("ok");

    // println!("Read board info");
    command::read_info(&mut port)
        .with_context(|| format!("Read boot loader identifier string"))?;

    // Erase
    println!("Erase chip");
    command::erase(&mut port)
        .with_context(|| format!("Erase chip"))?;

    // Program
    // println!("Program chip");
    let program_code  = firmware::read_hex_file(std::path::Path::new(&args.path))
        .with_context(|| format!("Read firmware program code"))?;

    // println!("Hex buffer length {}", program_code.buf.len());

    command::program(&mut port, &program_code)
        .with_context(|| format!("Flash program firmware"))?;

    // Verify
    // println!("Verify chip");
    command::verify(&mut port, &program_code)
        .with_context(|| format!("Verify written data"))?;

    return Ok(());
}
