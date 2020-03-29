use structopt::StructOpt;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;
use std::io::{self, Write};
// use std::fs;
// use std::env;
use std::fs::File;
use std::io::Read;

mod hex;
mod com;

// Baud rate 
// 9600,19200,57600,115200

#[derive(StructOpt)]
#[structopt(about = "Milandr 1986 firmware uploader", rename_all = "kebab-case")]
struct Cli {
    // #[structopt(default_value = "auto", short = "p", long = "port")]
    // port_name: String,
    #[structopt(default_value = "9200", short = "b", long = "baud")]
    baud_rate: String,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn probe_port() -> Result<String, serialport::Error> {
    match serialport::available_ports() {
        Ok(ports) => {
            let mdr_ports = ports
                .into_iter()
                .filter(|port| match &port.port_type {
                    SerialPortType::UsbPort(info) => {
                        println!("    Type: USB");
                        println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                        println!(
                            "     Serial Number: {}",
                            info.serial_number.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "      Manufacturer: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "           Product: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                        true
                    },
                    _ => false
                }) 
                .collect::<Vec<SerialPortInfo>>();

            match mdr_ports.len() {
                0 => { 
                    return Err(serialport::Error::new( serialport::ErrorKind::Unknown, "No ports found")); 
                },
                1 => {
                    return Ok(mdr_ports.last().unwrap().port_name.clone())
                },
                n => {
                    println!("Found {} ports:", n);
                    return Err(serialport::Error::new( serialport::ErrorKind::Unknown, "open() not implemented for platform")); 
                }
            };
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
            return Err(e)
        }
    }
}


fn main() {
    let args = Cli::from_args();

    let mut settings: SerialPortSettings = Default::default();

    settings.timeout = Duration::from_millis(1000);

    if let Ok(rate) = args.baud_rate.parse::<u32>() {
        settings.baud_rate = rate.into();
        println!("Baud rate: {}", settings.baud_rate);
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", args.baud_rate);
        ::std::process::exit(1);
    }

    // let hex_file = "1986_BOOT_UART.hex";
    // let hex_file = "LDM-K1986BE92QI_LIGHT.HEX";
    // match hex::read_hex_file(0x80000, std::path::Path::new(hex_file)) {
    //     Ok(hex_file) => {
    //         println!("Hex buffer length {}", hex_file.buffer.len());
    //     },
    //     Err(error) => {
    //         eprintln!("Error: '{}'", error);
    //         ::std::process::exit(1);
    //     }
    // };
    //
    // try to discover port
    let port_name = match probe_port() {
        Ok(name) => name,
        Err(error) => {
            eprintln!("Error: '{}'", error);
            ::std::process::exit(1);
        }
    };

    // match serialport::open_with_settings(&args.port_name, &settings) {
    match serialport::open_with_settings(&port_name, &settings) {
        Ok(port) => {
            // // read firmware file
            // let mut f = File::open(&args.path).expect("no file found");
            // let buf_len = f.metadata().unwrap().len() as usize;
            // let mut buffer = vec![0; buf_len];
            // f.read(&mut buffer).expect("buffer overflow");
            //
            // println!("Writing {} bytes on {} at {} baud:", buf_len, port_name, settings.baud_rate);
            //
            // match port.write(&buffer) {
            //     Ok(_) => {
            //         println!("Done!");
            //         std::io::stdout().flush().unwrap();
            //     }
            //     Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            //     Err(e) => eprintln!("{:?}", e)
            // }
            //
            println!("Checking port");
            match com::check_port(port) {
                Ok(_) => println!("ok"),
                Err(e) =>  eprintln!("{:?}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
