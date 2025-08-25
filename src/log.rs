use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

static LOG_LEVEL: AtomicUsize = AtomicUsize::new(LogLevel::Info as usize);

pub fn set_log_level(level: LogLevel) {
    LOG_LEVEL.store(level as usize, Ordering::Relaxed);
}

pub fn get_log_level() -> LogLevel {
    match LOG_LEVEL.load(Ordering::Relaxed) {
        1 => LogLevel::Error,
        2 => LogLevel::Warn,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        _ => LogLevel::Info,
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        if $crate::log::get_log_level() as usize >= $crate::log::LogLevel::Error as usize {
            eprintln!("[ERROR] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        if $crate::log::get_log_level() as usize >= $crate::log::LogLevel::Warn as usize {
            eprintln!("[WARN] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if $crate::log::get_log_level() as usize >= $crate::log::LogLevel::Info as usize {
            println!("[INFO] {}", format!($($arg)*));
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if $crate::log::get_log_level() as usize >= $crate::log::LogLevel::Debug as usize {
            println!("[DEBUG] {}", format!($($arg)*));
        }
    };
}
