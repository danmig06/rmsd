use rusb::{self as usb, GlobalContext};
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;
use std::fs::File;
use crate::log;

const MASS_STORAGE_CLASS_ID: u8 = 0x8;
const MASS_STORAGE_SUBCLASS_ID: u8 = 0x6;
const MASS_STORAGE_PROTOCOL_ID: u8 = 0x50;
const MASS_STORAGE_CBW_EXPECTED_SIZE: usize = 31;

#[derive(Serialize, Deserialize, Debug)]
#[repr(u8)]
pub enum Direction {
        HostToDevice = 0x00,
        DeviceToHost = 0x80
}

#[derive(Serialize, Debug)]
#[repr(C)]
pub struct CommandBlockWrapper {
        signature: u32,
        transaction_id: u32,
        length: u32,
        direction: u8,
        logical_unit_number: u8,
        command_length: u8,
        command_data: [u8; 16]
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum CommandStatus {
        Success    = 0x0,
        Error      = 0x1,
        PhaseError = 0x2,
        Reserved   = 0xFF
}

#[derive(Deserialize)]
#[repr(C)]
pub struct CommandStatusWrapper {
        signature: u32,
        transaction_id: u32,
        residue: u32,
        status: u8
}

#[derive(Debug)]
pub struct Device {
        pub generic_device: usb::Device<GlobalContext>,
        handle: Option<usb::DeviceHandle<GlobalContext>>,
        in_endpoint: u8,
        out_endpoint: u8, 
        selected_interface: u8
}

#[allow(dead_code)]
impl Device {
        pub fn name(&self) -> usb::Result<String> {
                let handle = match self.handle.as_ref() {
                        Some(h) => { h },
                        None => { 
                                let res = self.generic_device.open();
                                if res.is_err() {
                                        let e = res.unwrap_err();
                                        log::error!("name(): failed to open generic_device, cause: {}", e);
                                        if e == rusb::Error::Access {
                                                println!("Not enough privileges, aborting...");
                                                std::process::exit(1);
                                        }
                                        return Ok(String::from(""));
                                }
                                &res.unwrap()
                        }
                };
                let dev_descriptor = self.generic_device.device_descriptor();
                if dev_descriptor.is_err() {
                        log::error!("name(): failed to get device descriptor, cause: {}", dev_descriptor.unwrap_err());
                        return Ok(String::from(""));
                }
                handle.read_product_string_ascii(&dev_descriptor.unwrap())
        }

        pub fn open(&mut self) -> usb::Result<()> {
                self.handle = match self.generic_device.open() {
                        Ok(dev) => { 
                                if usb::supports_detach_kernel_driver() {
                                        dev.set_auto_detach_kernel_driver(true).unwrap();
                                }
                                dev.claim_interface(self.selected_interface).unwrap_or_else(|e| log::error!("open(): failed to claim interface, cause: {}", e));
                                Some(dev) 
                        },
                        Err(e) => {
                                log::error!("open(): failed to open generic_device, cause: {}", e); 
                                None 
                        }
                };
                Ok(())
        }

        pub fn send_command(&self, command_block: &[u8], direction: Direction, outcoming_bytes: u32) -> usb::Result<bool> {
                assert!(self.handle.is_some());
                let mut cbw: CommandBlockWrapper = CommandBlockWrapper { signature: 0x43425355,  transaction_id: 0, length: outcoming_bytes, logical_unit_number: 0, direction: direction as u8, command_length: command_block.len() as u8, command_data: [0; 16]};
                let handle = self.handle.as_ref().unwrap();
                cbw.command_data[..command_block.len()].copy_from_slice(command_block);
                let bytes_written = handle.write_bulk(self.in_endpoint, bincode::serialize(&cbw).unwrap().as_slice(), Duration::from_millis(0)).unwrap_or_else(|e| { log::error!("send_command(): failed to perform bulk write, cause: {}", e); 0});
                log::debug!("send_command(): sending CBW (31 bytes), {} bytes were written to the device endpoint (address = {})", bytes_written, self.in_endpoint);
                if bytes_written == MASS_STORAGE_CBW_EXPECTED_SIZE {
                        Ok(true)
                } else {
                        Ok(false)
                }
        }

        pub fn status(&self, residue: Option<&mut u32>) -> usb::Result<Option<CommandStatus>> {
                assert!(self.handle.is_some());
                let csw: CommandStatusWrapper;
                let handle = self.handle.as_ref().unwrap();
                let mut buf = [0u8; size_of::<CommandStatusWrapper>()];
                
                let bytes_read = handle.read_bulk(self.out_endpoint, &mut buf, Duration::from_millis(0)).unwrap_or_else(|e| { log::error!("status(): failed to perform bulk read, cause: {}", e); 0 });
                if bytes_read < 13 {
                        log::warning!("status(): Device returned only {} bytes instead of 13", bytes_read);
                        return Ok(None)
                }

                csw = bincode::deserialize(&buf).unwrap();
                log::debug!("status(): CSW ({bytes_read} bytes) successfully received, residue is {} bytes of data", csw.residue);
                match residue {
                        Some(r) => { *r = csw.residue; }
                        None => {  }
                };
                Ok(Some(match csw.status {
                        0 => { CommandStatus::Success },
                        1 => { CommandStatus::Error },
                        2 => { CommandStatus::PhaseError },
                        _ => { CommandStatus::Reserved }
                }))
        }

        pub fn query_capacity(&self, sector_count: Option<&mut u32>, sector_size: Option<&mut u32>) -> usb::Result<Option<CommandStatus>> {
                assert!(self.handle.is_some());
                let mut command_block: [u8; 10] = [0; 10];
                command_block[0] = 0x25;
                self.send_command(&command_block, Direction::DeviceToHost, 8)?;
                let handle = self.handle.as_ref().unwrap();
                let mut buf = [0u8; size_of::<u64>()];
                let bytes_read = handle.read_bulk(self.out_endpoint, &mut buf, Duration::from_millis(0)).unwrap_or_else(|e| { log::error!("query_capacity(): failed to perform bulk read, cause: {}", e); 0 });
                if bytes_read < buf.len() {
                        log::warning!("query_capacity(): Device returned only {} bytes instead of {}", bytes_read, buf.len());
                        return Ok(None)
                }
                if sector_count.is_some() {
                        *sector_count.unwrap() = u32::from_be_bytes(buf[..4].try_into().unwrap());
                }
                if sector_size.is_some() {
                        *sector_size.unwrap() = u32::from_be_bytes(buf[4..].try_into().unwrap());
                }
                self.status(None)
        }

        pub fn ready(&self) -> usb::Result<bool> {
                let cb = [0u8; 6];
                self.send_command(&cb, Direction::DeviceToHost, 0)?;
                let status = self.status(None).unwrap_or_else(|e| { log::error!("ready(): failed to get status, cause: {}", e); None });
                if status.is_none() {
                        return Ok(false);
                }
                Ok(status.unwrap() == CommandStatus::Success)
        }

        pub fn storage_read(&self, data: &mut [u8], start: u32, data_size: &mut usize) -> usb::Result<Option<CommandStatus>> {
                assert!((data.len() % 512) == 0);
                let success = self.initiate_storage_transfer(Direction::DeviceToHost, start, (data.len() / 512) as u16);
                log::debug!("storage_write(): starting transfer, result: {:?}", success);
                if success {
                        assert!(self.handle.is_some());
                        let handle = self.handle.as_ref().unwrap();
                        *data_size = handle.read_bulk(self.out_endpoint, data, Duration::from_millis(0)).unwrap_or_else(|e| { log::error!("storage_read(): bulk read failed, cause: {}", e); 0 });
                } else {
                        return Ok(None);
                }

                self.status(None)
        }

        pub fn storage_write(&self, data: &[u8], start: u32) -> usb::Result<Option<CommandStatus>> {
                assert!((data.len() % 512) == 0);
                let success = self.initiate_storage_transfer(Direction::HostToDevice, start, (data.len() / 512) as u16);
                log::debug!("storage_write(): starting transfer, result: {:?}", success);
                if success {
                        assert!(self.handle.is_some());
                        let handle = self.handle.as_ref().unwrap();
                        handle.write_bulk(self.in_endpoint, data, Duration::from_millis(0)).unwrap_or_else(|e| { log::error!("storage_write(): bulk write failed, cause: {}", e); 0 });
                } else {
                        return Ok(None);
                }

                self.status(None)
        }

        pub fn flash_image_from_file(&self, filename: &PathBuf, buffer_size: usize, preferred_size: Option<u32>, progress_cb: fn(u32, u32) -> ()) -> std::io::Result<bool> {
                let file_handle = File::open(filename);
                if file_handle.is_err() {
                        let error = file_handle.unwrap_err();
                        log::error!("flash_from_file(): failed to open file {:?}, cause: {}", filename, error);
                        return Ok(false);
                }
                let mut file = file_handle.unwrap();
                let file_size = std::fs::metadata(filename)?.len();
                if file_size % 512 != 0 {
                        log::error!("flash_from_file(): image size is not divisible by 512, therefore it is invalid");
                        return Ok(false);
                }
                let mut device_capacity = 0;
                self.query_capacity(Some(&mut device_capacity), None).unwrap_or_else(|e| { log::warning!("flash_from_file(): failed to determine device capacity, flashing process may fail due to the device not being big enough, cause: {}", e); None});
                if file_size > u64::from(device_capacity * 512) {
                        log::error!("flash_from_file(): Device has not enough space, unable to flash image");
                        return Ok(false);
                }
                let output_size = match preferred_size {
                        Some(sz) => {
                                if sz > (file_size / 512) as u32 {
                                        log::error!("preferred size ({sz} sectors) is greater than the size of the input image ({} sectors)", file_size / 512);
                                        std::process::exit(1);
                                };
                                sz
                        },
                        None => {
                                (file_size / 512) as u32
                        }
                };
                log::debug!("beginning to write image {:?} to device...", filename);
                let mut write_buffer = vec![0u8; buffer_size * 512];
                let mut current_sector: u32 = 0;
                let mut bytes_read: usize;
                'write_image: loop {
                        progress_cb(current_sector, output_size);
                        bytes_read = file.read(&mut write_buffer)?;
                        if bytes_read == 0 || current_sector > output_size {
                                break 'write_image;
                        }
                        self.storage_write(&write_buffer[..bytes_read], current_sector).unwrap();
                        current_sector += (bytes_read / 512) as u32;
                } 
                Ok(true)
        }

        pub fn clone_drive_to_file(&self, filename: &PathBuf, buffer_size: usize, preferred_size: Option<u32>, progress_cb: fn(u32, u32) -> ()) -> std::io::Result<bool> {
                let file_handle = File::create(filename);
                if file_handle.is_err() {
                        let error = file_handle.unwrap_err();
                        log::error!("clone_drive_to_file(): failed to create file {:?}, cause {}", filename, error);
                        return Ok(false);
                }
                let mut file = file_handle.unwrap();
                let mut device_capacity: u32 = 0;
                self.query_capacity(Some(&mut device_capacity), None).unwrap();
                let mut read_buffer = vec![0u8; buffer_size * 512];
                let output_size = match preferred_size {
                        Some(sz) => {
                                if sz > device_capacity {
                                        log::error!("preferred size ({sz} sectors) is greater than the target device's capacity ({device_capacity} sectors)");
                                        std::process::exit(1);
                                };
                                sz
                        },
                        None => {
                                device_capacity
                        }
                };
                log::debug!("cloning drive of {device_capacity} sectors ({output_size} sectors will be copied)...");
                let mut bytes_read: usize = 0;
                let mut current_sector: u32 = 0;
                for i in 0..u32::div_ceil(output_size, buffer_size as u32) {
                        current_sector = i * buffer_size as u32;
                        self.storage_read(&mut read_buffer, current_sector, &mut bytes_read).unwrap();
                        file.write(&read_buffer[..bytes_read])?;
                        progress_cb(current_sector, output_size);
                }
                progress_cb(current_sector + (bytes_read / 512) as u32, output_size);
                print!("\n");
                Ok(true)
        }

        fn initiate_storage_transfer(&self, direction: Direction, start_sector: u32, count: u16) -> bool {
                let mut cb = [0u8; 10];
                cb[0] = match direction {
                        Direction::HostToDevice => { 0x2A },
                        Direction::DeviceToHost => { 0x28 }
                };
                cb[2..6].copy_from_slice(&start_sector.to_be_bytes());
                cb[7..9].copy_from_slice(&count.to_be_bytes());
                let success = self.send_command(&cb, direction, u32::from(count) * 512).unwrap_or_else(|e| { log::error!("storage_write(): failed to send command, cause: {}", e); false });
                success
        }
}

#[allow(dead_code)]
pub fn list_devices() -> Vec<Device> {
        log::debug!("list_devices(): scanning...");
        let devices = usb::devices().unwrap();
        let mut list: Vec<Device> = vec![];
        for dev in devices.iter() {
                let dev_descriptor = dev.device_descriptor().unwrap();
                log::debug!("list_devices(): found {:?} | class: {}, subclass: {}, protocol: {}", dev, dev_descriptor.class_code(), dev_descriptor.sub_class_code(), dev_descriptor.protocol_code());
                let config_desc = dev.active_config_descriptor().unwrap();
                for interface in config_desc.interfaces() {
                        log::debug!("list_devices(): scanning interface {:?} for device {:#?}", interface.number(), dev);
                        let if_desc = interface.descriptors().nth(0).unwrap();
                        if if_desc.class_code() == MASS_STORAGE_CLASS_ID 
                        && if_desc.sub_class_code() == MASS_STORAGE_SUBCLASS_ID 
                        && if_desc.protocol_code() == MASS_STORAGE_PROTOCOL_ID {
                                let mut d = Device { generic_device: dev.clone(), handle: None, in_endpoint: 0, out_endpoint: 0, selected_interface: interface.number() };
                                for e in if_desc.endpoint_descriptors() {
                                        if e.address() & (Direction::DeviceToHost as u8) != 0 {
                                                d.out_endpoint = e.address();
                                        } else {
                                                d.in_endpoint = e.address();
                                        }
                                };
                                let device_name = d.name().unwrap_or(String::from("Unknown Device"));
                                log::debug!("list_devices(): [{} at {:#?}] Mass Storage Class interface found (input at endpoint {}, output at endpoint {})", device_name, dev, d.in_endpoint, d.out_endpoint);
                                list.push(d);
                        }
                }
        };
        list
}
