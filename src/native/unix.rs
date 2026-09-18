use alloc::ffi::CString;
use alloc::vec::Vec;

pub type PtyProcess = (i32, libc::pid_t);

#[repr(C)]
struct Winsize {
    ws_row: u16,
    ws_col: u16,
    ws_xpixel: u16,
    ws_ypixel: u16,
}

fn openpty() -> Result<(i32, i32), i32> {
    let mut master = -1i32;
    let mut slave = -1i32;

    cvt(unsafe {
        libc::openpty(&mut master, &mut slave, core::ptr::null_mut(), core::ptr::null(), core::ptr::null())
    })?;

    assert!(master >= 0 && slave >= 0);

    Ok((master, slave))
}

/// Spawns a child process attached to a newly created PTY.
///
/// Returns the PTY master fd and child pid on success.
/// `path` is the executable used by `execvp`, and `args` are passed as argv.
pub fn spawn_pty_process(path: &str, args: &[&str]) -> Result<PtyProcess, i32> {
    let (master, slave) = openpty()?;

    match unsafe { libc::fork() } {
        0 => child_exec(master, slave, path, args),
        pid if pid > 0 => {
            unsafe { libc::close(slave) };
            Ok((master, pid))
        }
        _ => Err(errno()),
    }
}

/// Updates the PTY window size associated with `master_fd`.
///
/// `rows`/`cols` are terminal cell dimensions, while `xpixel`/`ypixel`
/// are pixel dimensions reported to the child process.
pub fn set_pty_winsize(master_fd: i32, rows: u16, cols: u16, xpixel: u16, ypixel: u16) -> Result<(), i32> {
    let ws = Winsize {
        ws_row: rows,
        ws_col: cols,
        ws_xpixel: xpixel,
        ws_ypixel: ypixel,
    };
    cvt(unsafe { libc::ioctl(master_fd, libc::TIOCSWINSZ, &ws as *const Winsize as *const libc::c_void) }).map(|_| ())
}

fn child_prepare(master: i32, slave: i32) -> Result<(), i32> {
    unsafe {
        cvt(libc::close(master))?;
        cvt(libc::setsid())?;
        cvt(libc::dup2(slave, libc::STDIN_FILENO))?;
        cvt(libc::dup2(slave, libc::STDOUT_FILENO))?;
        cvt(libc::dup2(slave, libc::STDERR_FILENO))?;
        cvt(libc::close(slave))?;
    }
    Ok(())
}

/// Executes the child side of a PTY fork.
///
/// This function never returns: it either successfully replaces the process image via `execvp`,
/// or exits the child process with a non-zero code when setup fails.
fn child_exec(master: i32, slave: i32, path: &str, args: &[&str]) -> ! {
    if child_prepare(master, slave).is_err() {
        unsafe { libc::exit(1) };
    }

    let c_path = match CString::new(path) {
        Ok(v) => v,
        Err(_) => unsafe { libc::exit(1) },
    };
    let c_args = match args.iter().map(|&s| CString::new(s)).collect::<Result<Vec<_>, _>>() {
        Ok(v) => v,
        Err(_) => unsafe { libc::exit(1) },
    };
    let mut argv: Vec<*const i8> = c_args.iter().map(|s| s.as_ptr()).collect();
    argv.push(core::ptr::null());

    unsafe {
        libc::execvp(c_path.as_ptr(), argv.as_ptr());
        libc::exit(127);
    }
}

/// Converts c-style return values into `Result`.
fn cvt(ret: i32) -> Result<i32, i32> {
    if ret == -1 { Err(errno()) } else { Ok(ret) }
}

fn errno() -> i32 {
    unsafe { *libc::__errno_location() }
}
