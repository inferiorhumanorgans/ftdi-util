extern crate libusb;

use libusb::{request_type, Direction, RequestType, Recipient};
use std::time::Duration;

extern crate clap;
use clap::{App, AppSettings, Arg, SubCommand};

const FTDI_VENDOR : &str = "1027"; // 0x0403
const FTDI_CHIPIX_PID : &str = "24597"; // 0x6015
const FTDI_GET_LATENCY : u8 = 10;
const FTDI_SET_LATENCY : u8 = 9;

fn list_devices() {
    // From https://github.com/dcuddeback/libusb-rs
    let context = libusb::Context::new().unwrap();

    for device in context.devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        println!("Bus {:03} Device {:03} ID {:05}:{:05} (hex: {:04x}:{:04x})",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id(),
            device_desc.vendor_id(),
            device_desc.product_id());
    }
}

fn get_latency(device: &libusb::DeviceHandle, port_number: u16) {
    let request = request_type(Direction::In, RequestType::Vendor, Recipient::Device);
    let mut response : [u8; 1] = [0];
    device.read_control(request, FTDI_GET_LATENCY, 0, port_number, &mut response, Duration::from_secs(1)).expect("Failed to query adapter");

    println!("Latency is: {} ms", response[0])
}

fn set_latency(device: &libusb::DeviceHandle, port_number: u16, latency: u16) {
    let request = request_type(Direction::Out, RequestType::Vendor, Recipient::Device);
    let mut response : [u8; 1] = [0];
    device.write_control(request, FTDI_SET_LATENCY, latency, port_number, &mut response, Duration::from_secs(1)).expect("Failed to query adapter");
}

fn main() {
    let matches = App::new("ftdi-util")
                        .about("Configures an FTDI USB-RS232 dongle")
                        .author("Alex Zepeda")
                        .version("0.1")
                        .setting(AppSettings::SubcommandRequiredElseHelp)
                        .arg(Arg::with_name("vid")
                                    .help("USB vendor ID")
                                    .short("v")
                                    .long("vendor")
                                    .takes_value(true)
                                    .default_value(FTDI_VENDOR)
                                    .required(true))
                        .arg(Arg::with_name("pid")
                                    .help("USB product ID to use")
                                    .short("p")
                                    .long("product")
                                    .takes_value(true)
                                    .default_value(FTDI_CHIPIX_PID)
                                    .required(true))
                        .arg(Arg::with_name("port")
                                    .help("Endpoint index")
                                    .short("i")
                                    .long("port")
                                    .takes_value(true)
                                    .default_value("0")
                                    .required(true))
                        .subcommand(SubCommand::with_name("list-devices")
                                    .about("Lists all known USB devices"))
                        .subcommand(SubCommand::with_name("get-latency")
                                    .about("Displays the latency timer for the selected dongle"))
                        .subcommand(SubCommand::with_name("set-latency")
                                    .about("Sets the latency timer for the selected dongle in milliseconds")
                                    .arg(Arg::with_name("latency")
                                                .help("Latency in milliseconds")
                                                .short("l")
                                                .long("latency")
                                                .takes_value(true)
                                                .required(true)))
                        .get_matches();

    let vid : u16 = matches.value_of("vid").expect("VID not specified").parse().expect("VID must be an integer between 0 and 65535");
    let pid : u16 = matches.value_of("pid").expect("PID not specified").parse().expect("PID must be an integer between 0 and 65535");
    let port : u16 = matches.value_of("port").expect("Port not specified").parse().expect("Port must be an integer between 0 and 255");

    if let Some("list-devices") = matches.subcommand_name() {
        list_devices();
        return;
    }

    let context = libusb::Context::new().unwrap();
    let device = context.open_device_with_vid_pid(vid, pid).expect("Couldn't find FTDI adapter");

    match matches.subcommand() {
        ("get-latency", Some(_sub_matches)) => get_latency(&device, port),
        ("set-latency", Some(sub_matches)) => {
            let latency : u8 = sub_matches.value_of("latency")
                .expect("latency must be specified")
                .parse()
                .expect("latency must be an integer between 0 and 255");
            set_latency(&device, port, latency as u16)
        }
        _ => unreachable!()
    }
}
