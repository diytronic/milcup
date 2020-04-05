use structopt::StructOpt;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;
// use std::io::{self, Write};
// use std::fs;
// use std::env;
// use std::fs::File;
// use std::io::Read;

use std::{
    io::{
        self, 
        Error,
        ErrorKind,
    },
};

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
fn probe_port() -> Result<String, serialport::Error> {
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
        0 => Err(serialport::Error::new(serialport::ErrorKind::Unknown, "No ports found")),
        1 => Ok(mdr_ports.last().unwrap().port_name.clone()),
        n => Err(serialport::Error::new( serialport::ErrorKind::Unknown, "open() not implemented for platform"))
    };
}

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    SerialPort(serialport::Error),
    FlashError(command::Error),
    FirmwareError(firmware::Error),
}

impl From<serialport::Error> for AppError {
    fn from(err: serialport::Error) -> AppError {
        AppError::SerialPort(err)
    }
}

impl From<command::Error> for AppError {
    fn from(err: command::Error) -> AppError {
        AppError::FlashError(err)
    }
}

impl From<firmware::Error> for AppError {
    fn from(err: firmware::Error) -> AppError {
        AppError::FirmwareError(err)
    }
}

fn main() -> Result<(), AppError> {
    let args = Cli::from_args();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(3000);
    settings.baud_rate = 9600; // initial baud rate
    
    let port_name = if args.port_name == "auto" {
        probe_port()?
    } else {
        args.port_name
    };

    println!("Discovered port {}", port_name);

    let mut port = serialport::open_with_settings(&port_name, &settings)?;

    println!("Checking port");
    command::check_port(&mut port)?;

    println!("Set baud rate {}", args.baud_rate);
    command::set_baud_rate(&mut port, args.baud_rate);

    std::mem::drop(port);

    settings.baud_rate = args.baud_rate;

    // reopen with new baud rate
    let mut port = serialport::open_with_settings(&port_name, &settings)?;
    println!("Open port with baud rate {}", settings.baud_rate);

    println!("Read baud rate");
    command::read_baud_rate(&mut port)?;

    println!("Boot load");

    let boot_loader = include_str!("../firmware/1986_BOOT_UART.hex");
    let hex_file = firmware::parse_hex_buffer(boot_loader)?;

    println!("Hex buffer length {}", hex_file.buf.len());

    command::boot_load(&mut port, hex_file)?;
    println!("ok");

    println!("Read board info");
    match command::read_info(&mut port) {
        Ok(str) => println!("ok {}", str),
        Err(e) => {
            eprintln!("{:?}", e);
            ::std::process::exit(1);
        }
    }

    // Erase
    println!("Erase chip");
    match command::erase(&mut port) {
        Ok(_) => println!("ok"),
        Err(e) => {
            eprintln!("{:?}", e);
            ::std::process::exit(1);
        }
    }

    // Program
    println!("Program chip");
    match firmware::read_hex_file(std::path::Path::new(&args.path)) {
        Ok(hex_file) => {
            println!("Hex buffer length {}", hex_file.buf.len());

            match command::program(&mut port, &hex_file) {
                Ok(_) => println!("ok"),
                Err(e) => {
                    eprintln!("{:?}", e);
                    ::std::process::exit(1);
                }
            }

            // Verify
            println!("Verify chip");
            match command::verify(&mut port, &hex_file) {
                Ok(_) => println!("ok"),
                Err(e) => {
                    eprintln!("{:?}", e);
                    ::std::process::exit(1);
                }
            }
        },
        Err(error) => {
            eprintln!("Error: '{}'", error);
            ::std::process::exit(1);
        }
    };

    return Ok(());
}
