use std::thread;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::sync::Mutex;
use std::sync::Arc;

use serialport::prelude::*;
use serialport::open_with_settings;

use clap::{App, Arg, ArgMatches};


fn main() {
    let matches = App::new("serial_passthrough")
        .version("0.1")
        .author("Noah Ryan")
        .about("Forward bytes between serial interfaces, logging the results")
        .arg(Arg::with_name("INPORT")
                  .help("Input serial port")
                  .short("i")
                  .long("input")
                  .required(true)
                  .multiple(false)
                  .empty_values(false))
        .arg(Arg::with_name("OUTSTREAM")
                  .help("Output serial port")
                  .short("o")
                  .long("output")
                  .required(true)
                  .multiple(false)
                  .empty_values(false))
        .arg(Arg::with_name("BAUD")
                  .help("Baud rate")
                  .short("b")
                  .long("baud")
                  .required(false)
                  .multiple(false)
                  .default_value("115200")
                  .empty_values(false))
        // TODO could add other serial parameters
        .get_matches();

    run(matches);
}

fn run(matches: ArgMatches) {
    let baud = matches.value_of("BAUD").unwrap().parse::<u32>().unwrap();

    let first_port_name = matches.value_of("INPORT").unwrap().to_string();

    let first_port =
        open_with_settings( &first_port_name,
                            &SerialPortSettings { baud_rate: baud,
                                                  data_bits: DataBits::Eight,
                                                  flow_control: FlowControl::None,
                                                  parity: Parity::None,
                                                  stop_bits: StopBits::One,
                                                  timeout: Duration::from_millis(10),
    }).unwrap();
    let first_port = Arc::new(Mutex::new(first_port));


    let second_port_name = matches.value_of("OUTPORT").unwrap().to_string();
    let second_port =
        open_with_settings( &second_port_name,
                            &SerialPortSettings { baud_rate: baud,
                                                  data_bits: DataBits::Eight,
                                                  flow_control: FlowControl::None,
                                                  parity: Parity::None,
                                                  stop_bits: StopBits::One,
                                                  timeout: Duration::from_millis(10),
    }).unwrap();
    let second_port = Arc::new(Mutex::new(second_port));

    let first_file = File::create(format!("{}.bin", first_port_name)).unwrap();
    let second_file = File::create(format!("{}.bin", second_port_name)).unwrap();


    let thread_first_port = first_port.clone();
    let thread_second_port = second_port.clone();
    let first_port_thread = thread::spawn(move || {
        forward_serial_port(thread_first_port, thread_second_port, first_file);
    });

    let thread_first_port = first_port.clone();
    let thread_second_port = second_port.clone();
    let second_port_thread = thread::spawn(move || {
        forward_serial_port(thread_second_port, thread_first_port, second_file);
    });

    first_port_thread.join().unwrap();
    second_port_thread.join().unwrap();
}

fn forward_serial_port(first_port: Arc<Mutex<Box<dyn SerialPort>>>,
                       second_port: Arc<Mutex<Box<dyn SerialPort>>>,
                       mut file: File) {
    let mut buffer = Vec::new();

    loop {
        {
            first_port.lock().unwrap().read(&mut buffer).unwrap();
            file.write(&mut buffer).unwrap();
            second_port.lock().unwrap().write(&mut buffer).unwrap();
        }
    }
}
