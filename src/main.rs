use structopt::StructOpt;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;
// use std::io::{self, Write};
// use std::fs;
// use std::env;
// use std::fs::File;
// use std::io::Read;

mod hex;
mod com;
mod com_port;

// use com::FlashError;

// Baud rate 
// 9600,19200,57600,115200

#[derive(StructOpt)]
#[structopt(about = "Milandr 1986 firmware uploader", rename_all = "kebab-case")]
struct Cli {
    // #[structopt(default_value = "auto", short = "p", long = "port")]
    // port_name: String,
    #[structopt(default_value = "115200", short = "b", long = "baud")]
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

// fn sync() -> Result<(), FlashError> {
//     Ok(())
// }

fn main() {
    let args = Cli::from_args();

    let mut settings: SerialPortSettings = Default::default();

    // 9600,19200,57600,115200
    settings.timeout = Duration::from_millis(3000);
    settings.baud_rate = 9600; // initial baud rate

   
    // let hex_file = "LDM-K1986BE92QI_LIGHT.HEX";
    // match hex::read_hex_file(0x80000, std::path::Path::new(hex_file)) {
    //     Ok(hex_file) => {
    //         println!("Hex buffer length {}", hex_file.buf.len());
    //     },
    //     Err(error) => {
    //         eprintln!("Error: '{}'", error);
    //         ::std::process::exit(1);
    //     }
    // };
    
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
        Ok(mut port) => {
            // // read firmware file
            // let mut f = File::open(&args.path).expect("no file found");
            // let buf_len = f.metadata().unwrap().len() as usize;
            // let mut buffer = vec![0; buf_len];
            // f.read(&mut buffer).expect("buffer overflow");
            //
            println!("Checking port");
            match com::check_port(&mut port) {
                Ok(_) => println!("ok"),
                Err(e) => {
                    eprintln!("{:?}", e);
                    ::std::process::exit(1);
                }
            }

            if let Ok(rate) = args.baud_rate.parse::<u32>() {
                println!("Set baud rate {}", rate);
                match com::set_baud_rate(&mut port, rate) {
                    Ok(_) => println!("ok baud rate set success"),
                    Err(e) => {
                        eprintln!("{:?}", e);
                        ::std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: Invalid baud rate '{}' specified", args.baud_rate);
                ::std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }

    if let Ok(rate) = args.baud_rate.parse::<u32>() {
        settings.baud_rate = rate.into();
        // settings.timeout = Duration::from_millis(3000);
        println!("Baud rate: {}", settings.baud_rate);
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", args.baud_rate);
        ::std::process::exit(1);
    }

    // reopen with new baud rate
    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            println!("Open port with baud rate {}", settings.baud_rate);
            
            println!("Read baud rate");
            match com::read_baud_rate(&mut port) {
                Ok(_) => println!("ok"),
                Err(e) => {
                    eprintln!("{:?}", e);
                    ::std::process::exit(1);
                }
            }

            println!("Boot load");

            let hex_file = "1986_BOOT_UART.hex";
            match hex::read_hex_file(std::path::Path::new(hex_file)) {
                Ok(hex_file) => {
                    println!("Hex buffer length {}", hex_file.buf.len());

                    match com::boot_load(&mut port, hex_file) {
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

            println!("Read board info");
            match com::read_info(&mut port) {
                Ok(str) => println!("ok {}", str),
                Err(e) => {
                    eprintln!("{:?}", e);
                    ::std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
