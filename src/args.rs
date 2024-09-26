use std::path::PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
pub struct Arguments {
        #[command(subcommand)]
        pub command: Command,
}

#[allow(non_camel_case_types)]
#[derive(Subcommand)]
pub enum Command {
        /// Flash (or burn) the image to the target device
        flash(FlashOperationArgs),
        /// Clone the device's storage to the target image path
        clone(CloneOperationArgs),
        /// List all the available devices and exit
        list(ListOperationArgs),
}

#[derive(Args)]
pub struct FlashOperationArgs {
        /// Specify the input disc image
        #[arg(short, long)]
        pub image: PathBuf,
        /// Set the log level, it's recommended not to change this value (options: None, Info, Warning/Warn, Error, Debug)
        #[arg(short, long, global=true, default_value = "Error")]
        pub log_level: String,
        /// Add the device's port to the filter, this information alone should be unique and thus suffice to identify a single device
        #[arg(short = 'p', long = "port", global=true)]
        pub device_port: Option<u8>,
        /// Add the device's bus (the controller to which the it is connected) to the filter, this information alone should be unique and thus suffice to identify a single device
        #[arg(short = 'b', long = "bus", global=true)]
        pub device_bus: Option<u8>,
        /// Add the device's name (as in the string with which the device identifies itself as a product) to the filter, this information is not unique and in some cases you could still have duplicates
        #[arg(short = 'n', long, global=true)]
        pub device_name: Option<String>,
        /// Skip prompts, always answer 'y'
        #[arg(short = 'y', global=true, action)]
        pub skip_prompts: bool,
        /// Set the size of the buffer used for writing in sectors (up to 65535), higher values may achieve faster speeds
        #[arg(long, default_value_t = 32, global=true)]
        pub buffer_size: usize,
        /// Set the number of sectors to copy from the input image, this value must be less than the image's size
        #[arg(short, long, global=true)]
        pub sector_count: Option<u32>,
}

#[derive(Args)]
pub struct CloneOperationArgs {
        /// Specify the output disc image
        #[arg(short, long)]
        pub image: PathBuf,
        /// Set the log level, it's recommended not to change this value
        #[arg(short, long, global=true, default_value = "Error")]
        pub log_level: String,
        /// Add the device's port to the filter, this information alone should be unique and thus suffice to identify a single device
        #[arg(short = 'p', long = "port", global=true)]
        pub device_port: Option<u8>,
        /// Add the device's bus (the controller to which the it is connected) to the filter, this information alone should be unique and thus suffice to identify a single device
        #[arg(short = 'b', long = "bus", global=true)]
        pub device_bus: Option<u8>,
        /// Add the device's name (as in the string with which the device identifies itself as a product) to the filter, this information is not unique and in some cases you could still have duplicates
        #[arg(short = 'n', long, global=true)]
        pub device_name: Option<String>,
        /// Skip prompts, always answer 'y'
        #[arg(short = 'y', global=true, action)]
        pub skip_prompts: bool,
        /// Set the size of the buffer used for writing in sectors (up to 65535), higher values may achieve faster speeds
        #[arg(long, default_value_t = 32, global=true)]
        pub buffer_size: usize,
        /// Set the number of sectors to copy from the device, this value must be less than the device's capacity
        #[arg(short, long, global=true)]
        pub sector_count: Option<u32>,
}

#[derive(Args)]
pub struct ListOperationArgs {        
        /// Set the log level, it's recommended not to change this value
        #[arg(short, long, global=true, default_value = "Error")]
        pub log_level: String,
        /// Add the device's port to the filter, this information alone should be unique and thus suffice to identify a single device
        #[arg(short = 'p', long = "port", global=true)]
        pub device_port: Option<u8>,
        /// Add the device's bus (the controller to which the it is connected) to the filter, this information alone should be unique and thus suffice to identify a single device
        #[arg(short = 'b', long = "bus", global=true)]
        pub device_bus: Option<u8>,
        /// Add the device's name (as in the devices's product ID) to the filter, this information is not unique and in some cases you could still have duplicates
        #[arg(short = 'n', long, global=true)]
        pub device_name: Option<String>,
}
