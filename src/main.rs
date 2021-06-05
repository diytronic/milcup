use structopt::StructOpt;
use serialport::prelude::*;
use serialport::SerialPortType;
use std::time::Duration;
// use std::fmt;
// use std::error::Error;
// use std::io;

use anyhow::{Context, Result, bail};

use console::{style, Emoji};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};

// static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🚚 🔍 ", "🚚  ");
// 🚚

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
    // let pb = ProgressBar::new(100);
    let ports = serialport::available_ports()?;
    let mdr_ports = ports // we need only USB-COM ports
        .into_iter()
        .filter(|port| match &port.port_type {
            SerialPortType::UsbPort(info) => {
                debug!("    Type: USB");
                debug!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
                debug!( "     Serial Number: {}",
                    info.serial_number.as_ref().map_or("", String::as_str)
                );
                debug!( "      Manufacturer: {}",
                    info.manufacturer.as_ref().map_or("", String::as_str)
                );
                debug!( "           Product: {}",
                    info.product.as_ref().map_or("", String::as_str)
                );
                true
            },
            _ => false
        }) 
        .collect::<Vec<SerialPortInfo>>();

    return match mdr_ports.len() {
        0 => bail!("COM port not found"),
        1 => Ok(mdr_ports.last().unwrap().port_name.clone()),
        n => {
            let names = mdr_ports.iter().map(|port| port.port_name.clone()).collect::<String>();
            bail!("{} COM ports found choose right one with --port key {}", n, names);
        }
    };
}

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

// macro_rules! exec_step {
//     ($step:tt, $expression:expr) => {
//        $expression
//     };
// }

fn main() {
    if let Err(err) = try_main() {
        // error!("{:#?}", err);
        // error!("{:#?}", err);
        // eprintln!("Error: {:?}", err);
        // eprintln!("Error: {} [{}]", err.to_string(), err.root_cause());
        error!("{} [{}]", err.to_string(), err.root_cause());
        std::process::exit(1);
    }
}

// fn try_main() -> Result<(), anyhow::Error> {
fn try_main() -> Result<()> {
    env_logger::init();

    let mut step = 0;
    let mut print_step = |str: &str| {
        step += 1;
        println!("{} {}", style(format!("[{}/{}]", step, 10)).bold().dim(), str.to_string());
    };

    // warn!("[root] warn");
    // info!("[root] info");
    // debug!("[root] debug");
    // error!("[root] error");

    let args = Cli::from_args();

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(3000);
    settings.baud_rate = 9600; // initial baud rate
    
    let port_name = if args.port_name == "auto" {
        print_step("Probe COM port...");
        probe_port().with_context(|| format!("Probe com port"))?
    } else {
        args.port_name
    };

    print_step(format!("Using COM port {}", port_name).as_str());
    let mut port = serialport::open_with_settings(&port_name, &settings)
        .with_context(|| format!("Open COM port with default baud rate 9600"))?;

    // println!("Checking port");
    command::check_port(&mut port)
        .with_context(|| format!("Check COM port availability"))?;

    print_step(format!("Set baud rate {}", args.baud_rate).as_str());
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

    print_step("Writing boot loader");
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
    print_step("Erase chip");
    command::erase(&mut port)
        .with_context(|| format!("Erase chip"))?;

    // Program
    print_step("Writing firmware");
    let program_code  = firmware::read_hex_file(std::path::Path::new(&args.path))
        .with_context(|| format!("Read firmware program code"))?;

    // println!("Hex buffer length {}", program_code.buf.len());

    command::program(&mut port, &program_code)
        .with_context(|| format!("Flash program firmware"))?;

    // Verify
    print_step("Verify");
    command::verify(&mut port, &program_code)
        .with_context(|| format!("Verify written data"))?;

    return Ok(());
}
