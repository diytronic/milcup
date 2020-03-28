use structopt::StructOpt;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;
use std::io::{self, Write};

#[derive(StructOpt)]
#[structopt(about = "Milandr 1986 firmware uploader", rename_all = "kebab-case")]
struct Cli {
    #[structopt(default_value = "auto", short = "p", long = "port")]
    port_name: String,
    #[structopt(default_value = "115200", short = "b", long = "baud")]
    baud_rate: String,
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

fn auto_port_name() -> Result<String, serialport::Error> {
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
                    return Err(serialport::Error::new(
                            serialport::ErrorKind::Unknown,
                            "No ports found",
                    )); 
                },
                1 => {
                    return Ok(mdr_ports.last().unwrap().port_name.clone())
                },
                n => {
                    println!("Found {} ports:", n);
                    return Err(serialport::Error::new(
                            serialport::ErrorKind::Unknown,
                            "open() not implemented for platform",
                    )); 
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

    settings.timeout = Duration::from_millis(10);
    if let Ok(rate) = args.baud_rate.parse::<u32>() {
        settings.baud_rate = rate.into();
    } else {
        eprintln!("Error: Invalid baud rate '{}' specified", args.baud_rate);
        ::std::process::exit(1);
    }

    // println!("{}", auto_port_name().unwrap())
    let port_name = auto_port_name().unwrap();

    // match serialport::open_with_settings(&args.port_name, &settings) {
    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", port_name, settings.baud_rate);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
            ::std::process::exit(1);
        }
    }
}
