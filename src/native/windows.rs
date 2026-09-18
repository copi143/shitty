use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem::{size_of, zeroed};
use core::ptr::null_mut;

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HANDLE_FLAG_INHERIT, HANDLE_FLAGS, INVALID_HANDLE_VALUE};
use windows::Win32::Security::SECURITY_ATTRIBUTES;
use windows::Win32::System::Console::{ClosePseudoConsole, CreatePseudoConsole, ResizePseudoConsole, COORD, HPCON};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    CreateProcessW, DeleteProcThreadAttributeList, GetProcessId, InitializeProcThreadAttributeList,
    UpdateProcThreadAttribute, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION,
    PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE, STARTUPINFOEXW, CREATE_NO_WINDOW, EXTENDED_STARTUPINFO_PRESENT,
};

const DEFAULT_CONSOLE_COLS: i16 = 120;
const DEFAULT_CONSOLE_ROWS: i16 = 40;
const PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE_PTR: usize = PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize;

/// Windows 伪终端（ Pseudoconsole）进程句柄。
///
/// Windows pseudoconsole process handle.
pub struct PtyProcess {
    /// 用于向子进程写入输入的管道写入端
    pub input_write: HANDLE,
    /// 用于从子进程读取输出的管道读取端
    pub output_read: HANDLE,
    /// 子进程的 PID
    pub child_pid: u32,
    hpc: HPCON,
    process_handle: HANDLE,
    thread_handle: HANDLE,
}

impl Drop for PtyProcess {
    fn drop(&mut self) {
        if !self.hpc.is_invalid() {
            unsafe { ClosePseudoConsole(self.hpc) };
            self.hpc = HPCON(0);
        }
        if self.input_write != INVALID_HANDLE_VALUE && !self.input_write.is_invalid() {
            unsafe { let _ = CloseHandle(self.input_write); };
            self.input_write = HANDLE(core::ptr::null_mut());
        }
        if self.output_read != INVALID_HANDLE_VALUE && !self.output_read.is_invalid() {
            unsafe { let _ = CloseHandle(self.output_read); };
            self.output_read = HANDLE(core::ptr::null_mut());
        }
        if self.thread_handle != INVALID_HANDLE_VALUE && !self.thread_handle.is_invalid() {
            unsafe { let _ = CloseHandle(self.thread_handle); };
            self.thread_handle = HANDLE(core::ptr::null_mut());
        }
        if self.process_handle != INVALID_HANDLE_VALUE && !self.process_handle.is_invalid() {
            unsafe { let _ = CloseHandle(self.process_handle); };
            self.process_handle = HANDLE(core::ptr::null_mut());
        }
    }
}

/// 创建一个 Windows 伪终端并生成子进程。
///
/// Create a Windows pseudoconsole and spawn a child process.
pub fn spawn_pty_process(path: &str, args: &[&str]) -> Result<PtyProcess, i32> {
    eprintln!("winpty: spawn path={path} args={:?}", args);
    eprintln!("winpty: cmdline={}", build_command_line(path, args));
    let mut output_read = HANDLE(core::ptr::null_mut());
    let mut output_write = HANDLE(core::ptr::null_mut());
    let mut input_read = HANDLE(core::ptr::null_mut());
    let mut input_write = HANDLE(core::ptr::null_mut());

    let sa = SECURITY_ATTRIBUTES {
        nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: true.into(),
    };

    if unsafe { CreatePipe(&mut output_read, &mut output_write, Some(&sa), 0) }.is_err() {
        eprintln!("winpty: CreatePipe output failed err={}", last_error());
        return Err(last_error());
    }
    if unsafe { CreatePipe(&mut input_read, &mut input_write, Some(&sa), 0) }.is_err() {
        eprintln!("winpty: CreatePipe input failed err={}", last_error());
        unsafe {
            let _ = CloseHandle(output_read);
            let _ = CloseHandle(output_write);
        }
        return Err(last_error());
    }

    if unsafe { windows::Win32::Foundation::SetHandleInformation(output_read, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }.is_err()
        || unsafe { windows::Win32::Foundation::SetHandleInformation(input_write, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }.is_err()
    {
        let err = last_error();
        eprintln!("winpty: SetHandleInformation failed err={}", err);
        unsafe {
            let _ = CloseHandle(output_read);
            let _ = CloseHandle(output_write);
            let _ = CloseHandle(input_read);
            let _ = CloseHandle(input_write);
        }
        return Err(err);
    }

    let hpc;
    let size = COORD {
        X: DEFAULT_CONSOLE_COLS,
        Y: DEFAULT_CONSOLE_ROWS,
    };
    let hr = unsafe { CreatePseudoConsole(size, input_read, output_write, 0) };
    if let Err(err) = hr {
        eprintln!("winpty: CreatePseudoConsole failed hr={}", err.code().0);
        unsafe {
            let _ = CloseHandle(input_read);
            let _ = CloseHandle(output_write);
            let _ = CloseHandle(output_read);
            let _ = CloseHandle(input_write);
        }
        return Err(err.code().0);
    }
    hpc = hr.unwrap();

    let mut attr_list_size = 0usize;
    unsafe {
        let _ = InitializeProcThreadAttributeList(None, 1, None, &mut attr_list_size);
    }
    let attr_units = (attr_list_size + size_of::<usize>() - 1) / size_of::<usize>();
    let mut attr_list_buf = vec![0usize; attr_units];
    let attr_list = LPPROC_THREAD_ATTRIBUTE_LIST(attr_list_buf.as_mut_ptr() as *mut _);
    if unsafe { InitializeProcThreadAttributeList(Some(attr_list), 1, None, &mut attr_list_size) }.is_err() {
        let err = last_error();
        eprintln!("winpty: InitializeProcThreadAttributeList failed err={}", err);
        unsafe {
            ClosePseudoConsole(hpc);
            let _ = CloseHandle(output_read);
            let _ = CloseHandle(input_write);
        }
        return Err(err);
    }

    let mut startup_info_ex: STARTUPINFOEXW = unsafe { zeroed() };
    startup_info_ex.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup_info_ex.lpAttributeList = attr_list;

    let hpc_param = hpc;
    if unsafe {
        UpdateProcThreadAttribute(
            attr_list,
            0,
            PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE_PTR,
            Some(&hpc_param as *const _ as *const core::ffi::c_void),
            size_of::<HPCON>(),
            None,
            None,
        )
    }
    .is_err()
    {
        let err = last_error();
        eprintln!("winpty: UpdateProcThreadAttribute failed err={}", err);
        unsafe {
            DeleteProcThreadAttributeList(attr_list);
            ClosePseudoConsole(hpc);
            let _ = CloseHandle(input_read);
            let _ = CloseHandle(output_write);
            let _ = CloseHandle(output_read);
            let _ = CloseHandle(input_write);
        }
        return Err(err);
    }

    let cmdline = build_command_line(path, args);
    let mut cmdline: Vec<u16> = cmdline.encode_utf16().chain(core::iter::once(0)).collect();

    let mut process_info: PROCESS_INFORMATION = unsafe { zeroed() };
    let created = unsafe {
        let cmd = PWSTR::from_raw(cmdline.as_mut_ptr());
        let result = CreateProcessW(
            PCWSTR::null(),
            Some(cmd),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            PCWSTR::null(),
            &mut startup_info_ex.StartupInfo,
            &mut process_info,
        );
        if result.is_ok() {
            eprintln!("winpty: CreateProcessW ok last_error={}", unsafe { GetLastError().0 });
        }
        result
    };
    if created.is_ok() {
        eprintln!("winpty: child pid={}", unsafe { GetProcessId(process_info.hProcess) });
    }
    unsafe {
        let _ = CloseHandle(input_read);
        let _ = CloseHandle(output_write);
    }
    unsafe { DeleteProcThreadAttributeList(attr_list) };

    if created.is_err() {
        let err = last_error();
        eprintln!("winpty: CreateProcessW failed err={}", err);
        unsafe {
            ClosePseudoConsole(hpc);
            let _ = CloseHandle(output_read);
            let _ = CloseHandle(input_write);
        }
        return Err(err);
    }
    eprintln!("winpty: CreateProcessW ok");

    let child_pid = unsafe { GetProcessId(process_info.hProcess) };
    Ok(PtyProcess {
        input_write,
        output_read,
        child_pid,
        hpc,
        process_handle: process_info.hProcess,
        thread_handle: process_info.hThread,
    })
}

/// 调整伪终端的窗口大小。
///
/// Resize the pseudoconsole window.
pub fn set_pty_winsize(process: &PtyProcess, rows: u16, cols: u16, _xpixel: u16, _ypixel: u16) -> Result<(), i32> {
    let size = COORD {
        X: cols as i16,
        Y: rows as i16,
    };
    let hr = unsafe { ResizePseudoConsole(process.hpc, size) };
    if hr.is_ok() { Ok(()) } else { Err(hr.unwrap_err().code().0) }
}

fn last_error() -> i32 {
    unsafe { GetLastError().0 as i32 }
}

fn build_command_line(path: &str, args: &[&str]) -> String {
    let mut cmd = String::new();
    cmd.push_str(&quote_arg(path));
    for arg in args {
        cmd.push(' ');
        cmd.push_str(&quote_arg(arg));
    }
    cmd
}

fn quote_arg(arg: &str) -> String {
    if arg.is_empty() {
        return "\"\"".to_string();
    }
    let needs_quote = arg.bytes().any(|c| matches!(c, b' ' | b'\t' | b'"'));
    if !needs_quote {
        return arg.to_string();
    }
    // Windows command-line escaping follows the CommandLineToArgvW-style rules:
    // backslashes before a quote must be doubled, and trailing backslashes inside
    // quotes must also be doubled so parsing round-trips correctly.
    let mut out = String::from("\"");
    let mut backslashes = 0usize;
    for ch in arg.chars() {
        if ch == '\\' {
            backslashes += 1;
            continue;
        }
        if ch == '"' {
            for _ in 0..(backslashes * 2 + 1) {
                out.push('\\');
            }
            out.push('"');
            backslashes = 0;
            continue;
        }
        for _ in 0..backslashes {
            out.push('\\');
        }
        backslashes = 0;
        out.push(ch);
    }
    for _ in 0..(backslashes * 2) {
        out.push('\\');
    }
    out.push('"');
    out
}
