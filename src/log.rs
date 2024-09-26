use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[allow(dead_code)]
pub enum Level {
        Info  = 0,
        Warn  = 1,
        Error = 2,
        Debug = 4,
        None  = -1
}

pub const COLOR_INFO: &str = "\x1b[34;2m";
pub const COLOR_WARN: &str = "\x1b[33m";
pub const COLOR_ERROR: &str = "\x1b[31;1m";
pub const COLOR_DEBUG: &str = "\x1b[0;2m";
pub const COLOR_RESET: &str = "\x1b[0m";

thread_local!{
        static _LOG_LEVEL: Cell<Level> = Cell::new(Level::None)
}

pub fn set_level(level: Level) {
        _LOG_LEVEL.with(|val| val.set(level));
}

pub fn level() -> Level {
        _LOG_LEVEL.with(|val| val.get())
}


pub fn level_from(str: &str) -> Level {
        match str.to_ascii_uppercase().as_ref() {
                "INFO"             => { Level::Info },
                "WARNING" | "WARN" => { Level::Warn },
                "ERROR"            => { Level::Error },
                "DEBUG"            => { Level::Debug },
                "NONE"             => { Level::None },
                _                  => { Level::Error }
        }
}
#[macro_export]
macro_rules! info {
        ($($e:expr), *) => {
                if log::level() >= log::Level::Info {
                        print!("{}[INFO] ", log::COLOR_INFO);
                        println!($($e), *);
                        print!("{}", log::COLOR_RESET);
                }
        };
}
pub(crate) use info;

#[macro_export]
macro_rules! warning {
        ($($e:expr), +) => {
                if log::level() >= log::Level::Warn {
                        print!("{}[WARNING] ", log::COLOR_WARN);
                        println!($($e), *);
                        print!("{}", log::COLOR_RESET);
                }
        };
}
pub(crate) use warning;

#[macro_export]
macro_rules! error {
        ($($e:expr), +) => {
                if log::level() >= log::Level::Error {
                        print!("{}[ERROR] ", log::COLOR_ERROR);
                        println!($($e), *);
                        print!("{}", log::COLOR_RESET);
                }
        };
}
pub(crate) use error;

#[macro_export]
macro_rules! debug {
        ($($e:expr), +) => {
                if log::level() >= log::Level::Debug {
                        print!("{}[DEBUG] ", log::COLOR_DEBUG);
                        println!($($e), *);
                        print!("{}", log::COLOR_RESET);
                }
        };
}
pub(crate) use debug;