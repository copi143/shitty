#![allow(unused_macros)]

#[macro_use]
#[cfg(feature = "logging")]
mod _0 {
    use core::fmt::Arguments;
    use core::sync::atomic::{AtomicI32, Ordering};

    pub static LOGGER: spin::Mutex<Option<fn(Arguments)>> = spin::Mutex::new(None);
    pub static LOG_LEVEL: AtomicI32 = AtomicI32::new(1);

    pub fn set_logger(logger: fn(Arguments)) {
        *crate::logger::LOGGER.lock() = Some(logger);
    }

    pub fn set_log_level(level: i32) {
        crate::logger::LOG_LEVEL.store(level, Ordering::Relaxed);
    }

    pub fn get_log_level() -> i32 {
        crate::logger::LOG_LEVEL.load(Ordering::Relaxed)
    }

    macro_rules! debug {
        ($($arg:tt)*) => {{
            if $crate::logger::LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) <= 0 {
                $crate::logger::println(format_args!($($arg)*));
            }
        }};
    }

    macro_rules! info {
        ($($arg:tt)*) => {{
            if $crate::logger::LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) <= 1 {
                $crate::logger::println(format_args!($($arg)*));
            }
        }};
    }

    macro_rules! warn {
        ($($arg:tt)*) => {{
            if $crate::logger::LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) <= 2 {
                $crate::logger::println(format_args!($($arg)*));
            }
        }};
    }

    macro_rules! error {
        ($($arg:tt)*) => {{
            if $crate::logger::LOG_LEVEL.load(core::sync::atomic::Ordering::Relaxed) <= 3 {
                $crate::logger::println(format_args!($($arg)*));
            }
        }};
    }

    pub fn println(args: Arguments) {
        if let Some(logger) = crate::logger::LOGGER.lock().as_ref() {
            logger(args);
        }
    }
}

#[macro_use]
#[cfg(not(feature = "logging"))]
mod _0 {
    use core::fmt::Arguments;

    pub fn set_logger(_logger: fn(Arguments)) {}

    pub fn set_log_level(_level: i32) {}

    pub fn get_log_level() -> i32 {
        0
    }

    macro_rules! debug {
        ($($arg:tt)*) => {{}};
    }

    macro_rules! info {
        ($($arg:tt)*) => {{}};
    }

    macro_rules! warn {
        ($($arg:tt)*) => {{}};
    }

    macro_rules! error {
        ($($arg:tt)*) => {{}};
    }

    pub fn println(_args: Arguments) {}
}

pub use _0::*;
