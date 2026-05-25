#![feature(duration_millis_float)]

use std::error::Error;
use std::io;
#[cfg(unix)]
use std::os::fd::{FromRawFd, OwnedFd};

#[cfg(windows)]
use shitty::native::PtyProcess;
use shitty::native::spawn_pty_process;
use winit::event_loop::EventLoop;

mod exec {
    pub mod conf;
    pub mod term;
    pub mod util;
}

use exec::conf::*;
use exec::term::*;
use exec::util::*;

macro_rules! eprintlns {
    ($($fmt:expr $(, $args:expr)*);+ $(;)?) => {{
        $(
            eprintln!($fmt $(, $args)*);
        )+
    }};
}

#[cfg(unix)]
fn parent_main(master: OwnedFd, child: libc::pid_t) -> Result<(), Box<dyn Error>> {
    shitty::set_logger(|args| println!("\r\x1b[KTerminal: {:?}", args));
    EventLoop::new()?.run_app(&mut App::new(master, child)).map_err(|e| Box::new(e) as Box<dyn Error>)
}

#[cfg(windows)]
fn parent_main(process: PtyProcess) -> Result<(), Box<dyn Error>> {
    shitty::set_logger(|args| println!("\r\x1b[KTerminal: {:?}", args));
    EventLoop::new()?.run_app(&mut App::new(process)).map_err(|e| Box::new(e) as Box<dyn Error>)
}

fn help() {
    eprintlns!(
        "Usage: terminal [OPTIONS] [--] [COMMAND [ARGS...]]";
        "";
        "Options:";
        "-s, --shell SHELL   Shell to use when no command is provided";
        "-h, --help          Show this help text";
        "";
        "Examples:";
        "terminal";
        "terminal --shell zsh -- -l";
        "terminal /bin/bash -lc 'echo hello'";
    );
}

#[cfg(windows)]
const DEFAULT_SHELL_ENV: &str = "COMSPEC";
#[cfg(not(windows))]
const DEFAULT_SHELL_ENV: &str = "SHELL";

fn launch(shell: Option<&str>, args: Option<&[&str]>) -> Result<(), Box<dyn Error>> {
    let default_shell = env_or_default(DEFAULT_SHELL_ENV, DEFAULT_SHELL);
    let shell = shell.unwrap_or(&default_shell);
    eprintln!("Launching shell: {shell} with args: {:?}", args.unwrap_or(DEFAULT_ARGS));

    #[cfg(unix)]
    {
        let (master, child) =
            spawn_pty_process(&shell, args.unwrap_or(DEFAULT_ARGS)).map_err(std::io::Error::from_raw_os_error)?;
        parent_main(unsafe { OwnedFd::from_raw_fd(master) }, child)
    }

    #[cfg(windows)]
    {
        let process =
            spawn_pty_process(&shell, args.unwrap_or(DEFAULT_ARGS)).map_err(std::io::Error::from_raw_os_error)?;
        parent_main(process)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1")
    };

    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut args = args.iter().peekable();

    // If no arguments are provided, launch the default shell.
    if args.peek().is_none() {
        return launch(None, None);
    }

    // If first arg doesn't start with '-', treat it as command
    if let Some(first) = args.peek().cloned() {
        if !first.starts_with('-') {
            let prog = args.next().unwrap();
            let args = args.map(|s| s.as_str()).collect::<Vec<_>>();
            return launch(Some(prog), Some(&args));
        }
    }

    // Otherwise parse options
    let mut shell = env_or_default(DEFAULT_SHELL_ENV, DEFAULT_SHELL);
    let mut command = None;
    let mut command_args = Vec::new();
    let mut passthrough = false;

    while let Some(arg) = args.next() {
        if passthrough {
            command_args.push(arg);
            continue;
        }

        if arg == "--" {
            passthrough = true;
            continue;
        }

        if arg == "-h" || arg == "--help" {
            help();
            return Ok(());
        }

        if arg == "-s" || arg == "--shell" {
            shell = args
                .next()
                .unwrap_or_else(|| {
                    eprintln!("Error: Missing value for {arg}");
                    std::process::exit(1);
                })
                .to_string();
            continue;
        }

        if let Some(shell_arg) = arg.strip_prefix("--shell=") {
            if shell_arg.is_empty() {
                return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, "Missing value for --shell")));
            }
            shell = shell_arg.to_string();
            continue;
        }

        if arg.starts_with('-') {
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, format!("Unknown argument: {arg}"))));
        }

        command = Some(arg);
        command_args.extend(args);
        break;
    }

    let (program, args_vec) = match command {
        Some(program) => (program, command_args.iter().map(|s| s.as_str()).collect::<Vec<_>>()),
        None => (&shell, DEFAULT_ARGS.iter().map(|s| *s).collect::<Vec<_>>()),
    };

    launch(Some(program), Some(&args_vec))
}
