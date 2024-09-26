use std::io::Read;
use crate::log;
use crate::mass_storage;
const BAR_WIDTH: usize = 100;

pub fn filter_devices(list: &mut Vec<mass_storage::Device>, name: Option<String>, bus: Option<u8>, port: Option<u8>) {
        for i in 0..list.len() {
                if name.is_some() {
                        if list[i].name().unwrap_or(String::new()) != *name.as_ref().unwrap() {
                                list.remove(i);
                                continue;
                        }
                }
                if bus.is_some() {
                        if list[i].generic_device.bus_number() != bus.unwrap() {
                                list.remove(i);
                                continue;
                        }
                }
                if port.is_some() {
                        if list[i].generic_device.port_number() != port.unwrap() {
                                list.remove(i);
                                continue;
                        }
                }
        }
}

pub fn do_progress_bar(current: u32, total: u32) -> () {
        let progress = current as f32 / total as f32;
        let progress_len = (progress * BAR_WIDTH as f32) as usize;
        let mut bar = vec!['='; progress_len as usize];
        bar.extend_from_slice(&vec![' '; BAR_WIDTH - progress_len as usize]);
        print!("\r[{}] - {:.2}% ({current} / {total} sectors copied)", String::from_iter(bar.iter()), progress * 100.0);
}

fn choose_target_if_dup(list: &mut Vec<mass_storage::Device>) -> &mut mass_storage::Device {
        if list.len() > 1 {
                println!("Multiple devices fit the specified filter, select which one to use for the operation:");
                for (n, d) in list.iter().enumerate() {
                        println!("\t{}. '{}' at bus {}, port {}", n, d.name().unwrap(), d.generic_device.bus_number(), d.generic_device.port_number());
                }
                let mut input: String = String::new();
                #[allow(unused_labels)]
                'select: loop {
                        print!("> ");
                        std::io::stdin().read_line(&mut input).unwrap_or_else(|_e| std::process::exit(255));
                        let dev_num = input.parse::<usize>();
                        if dev_num.is_ok() {
                                let n = dev_num.unwrap();
                                if n > 0 && n < list.len() {
                                        return &mut list[n];
                                }
                        }
                }
        }
        &mut list[0]
}

pub fn acquire_target(list: &mut Vec<mass_storage::Device>, skip_prompts: bool, buffer_size: usize) -> &mut mass_storage::Device {
        let target: &mut mass_storage::Device = choose_target_if_dup(list);
        if skip_prompts {
                println!("Device '{}' (bus {}, port {}) was automatically selected", target.name().unwrap(), target.generic_device.bus_number(), target.generic_device.port_number());
        } else {
                println!("Device '{}' (bus {}, port {}) has been selected, are you sure [Y/N]?", target.name().unwrap(), target.generic_device.bus_number(), target.generic_device.port_number());
                if !wait_confirm() {
                        std::process::exit(0);
                }
        }
        if buffer_size > u16::max_value() as usize {
                log::error!("buffer size is greater than 65535");
                std::process::exit(1);
        }
        target.open().unwrap();
        target
}

pub fn wait_confirm() -> bool {
        let mut input: [u8; 1] = [0];
        loop {
                print!("> ");
                std::io::stdin().read(&mut input).unwrap_or_else(|_e| std::process::exit(255));
                let chr = &input[0];
                if chr.to_ascii_uppercase() == b'Y' {
                        return true;
                } else if chr.to_ascii_uppercase() == b'N' {
                        return false;
                }
        }
}
