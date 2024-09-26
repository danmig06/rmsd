mod mass_storage;
#[macro_use]
mod log;
use clap::Parser;
mod args;
mod util;
use util::{ acquire_target, filter_devices, do_progress_bar };

fn main() {
        rusb::set_log_level(rusb::LogLevel::Error);
        log::set_level(log::Level::Error);
        let arguments = args::Arguments::parse();
        let mut list = mass_storage::list_devices();
        if list.len() == 0 {
                println!("No Mass Storage Class devices detected");
                std::process::exit(1);
        }

        match arguments.command {
                args::Command::flash(args) => {
                        log::set_level(log::level_from(&args.log_level));
                        filter_devices(&mut list, args.device_name, args.device_bus, args.device_port);
                        let target: &mut mass_storage::Device = acquire_target(&mut list, args.skip_prompts, args.buffer_size);
                        target.flash_image_from_file(&args.image, args.buffer_size, args.sector_count, do_progress_bar).expect("Flashing operation failed, please retry");
                },
                args::Command::clone(args) => { 
                        log::set_level(log::level_from(&args.log_level));
                        filter_devices(&mut list, args.device_name, args.device_bus, args.device_port);
                        let target: &mut mass_storage::Device = acquire_target(&mut list, args.skip_prompts, args.buffer_size);
                        target.clone_drive_to_file(&args.image, args.buffer_size, args.sector_count, do_progress_bar).expect("Flashing operation failed, please retry");
                
                },
                args::Command::list(args) => {
                        log::set_level(log::level_from(&args.log_level));
                        filter_devices(&mut list, args.device_name, args.device_bus, args.device_port);
                        for (n, d) in list.iter().enumerate() {
                                println!("{}. '{}' at bus {}, port {}", n, d.name().unwrap(), d.generic_device.bus_number(), d.generic_device.port_number());
                        }
                }
        };
}
