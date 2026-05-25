use std::collections::VecDeque;
use std::io::Write;
use std::num::NonZeroU32;
use std::ops::AddAssign;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::fd::{AsRawFd, OwnedFd};
#[cfg(unix)]
use std::process;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, ERROR_BROKEN_PIPE, ERROR_NO_DATA, GetLastError, HANDLE};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{ReadFile, WriteFile};
#[cfg(windows)]
use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

use arboard::Clipboard;
#[cfg(unix)]
use shitty::native::set_pty_winsize;
#[cfg(windows)]
use shitty::native::{PtyProcess, set_pty_winsize};
use shitty::{BufMode, Drawable, Event, Terminal};
use softbuffer::{Context, Surface};
use winit::application::ApplicationHandler;
use winit::event::{Ime, MouseScrollDelta, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Icon, ImePurpose, Window, WindowAttributes, WindowId};

use crate::exec::conf::*;
use crate::exec::util::*;

static CLIPBOARD: LazyLock<Mutex<Clipboard>> = LazyLock::new(|| Mutex::new(Clipboard::new().unwrap()));

#[allow(unused_variables)]
fn init_terminal(ansi_sender: Sender<Vec<u8>>) -> Terminal {
    #[cfg(feature = "font-abglyph")]
    let font = {
        let font_bytes = include_bytes!("../../docs/assets/NotoSans-VariableFont_wdth,wght.ttf") as &[u8];
        shitty::AbGlyphFont::new(FONT_SIZE, font_bytes)
    };
    #[cfg(all(not(feature = "font-abglyph"), feature = "font-unifont"))]
    let font = shitty::Unifont::new();
    let mut terminal = Terminal::new(0, 0, BufMode::Double, vec![font]);
    terminal.set_history_size(1024);
    terminal.set_scroll_speed(5);
    {
        let mut cb = terminal.callbacks();
        cb.clipboard_get = Some(Box::new(move || CLIPBOARD.lock().unwrap().get_text().ok()));
        cb.clipboard_set = Some(Box::new(move |text| {
            CLIPBOARD.lock().unwrap().set_text(text).ok();
        }));
        cb.pty_write = Some(Box::new(move |data| ansi_sender.send(data.to_vec()).unwrap()));
    }
    terminal
}

pub struct App {
    #[cfg(unix)]
    master: Arc<OwnedFd>,
    #[cfg(unix)]
    child: libc::pid_t,
    #[cfg(windows)]
    process: PtyProcess,
    terminal: Arc<Mutex<Terminal>>,
    ansi_receiver: Option<Receiver<Vec<u8>>>,
    window: Option<Rc<Window>>,
    old_width: usize,
    old_height: usize,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    surface_width: usize,
    surface_height: usize,
    dirty: Arc<AtomicBool>,
    input_begin_time: Instant,
    input_duration: Duration,
    output_duration: Arc<Mutex<Duration>>,
    frame_duration: Duration,
    frame_times: VecDeque<Instant>,
    max_frame_history: usize,
    frame_counter: usize,
    alloc_info_print_time: Option<Instant>,
    app_start_time: Instant,
}

impl Drop for App {
    #[cfg(unix)]
    fn drop(&mut self) {
        if self.child <= 0 {
            return;
        }

        unsafe {
            let _ = libc::kill(-self.child, libc::SIGHUP);
            let _ = libc::kill(self.child, libc::SIGTERM);

            let mut status: libc::c_int = 0;
            loop {
                let r = libc::waitpid(self.child, &mut status as *mut libc::c_int, 0);
                if r == -1 {
                    break;
                }
                if r == self.child {
                    break;
                }
            }
        }
    }

    #[cfg(windows)]
    fn drop(&mut self) {
        let pid = self.process.child_pid;
        if pid == 0 {
            return;
        }
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid);
            if let Ok(handle) = handle {
                let _ = TerminateProcess(handle, 0);
                let _ = CloseHandle(handle);
            }
        }
    }
}

impl App {
    pub fn new(
        #[cfg(unix)] master: OwnedFd,
        #[cfg(unix)] child: libc::pid_t,
        #[cfg(windows)] process: PtyProcess,
    ) -> Self {
        let (ansi_sender, ansi_receiver) = channel();
        App {
            #[cfg(unix)]
            master: master.into(),
            #[cfg(unix)]
            child,
            #[cfg(windows)]
            process,
            terminal: Mutex::new(init_terminal(ansi_sender)).into(),
            ansi_receiver: Some(ansi_receiver),
            window: None,
            old_width: 0,
            old_height: 0,
            surface: None,
            surface_width: 0,
            surface_height: 0,
            dirty: AtomicBool::new(false).into(),
            input_begin_time: Instant::now(),
            input_duration: Duration::ZERO,
            output_duration: Arc::new(Duration::ZERO.into()),
            frame_duration: Duration::ZERO,
            frame_times: VecDeque::with_capacity(60),
            max_frame_history: 60,
            frame_counter: 0,
            alloc_info_print_time: None,
            app_start_time: Instant::now(),
        }
    }

    fn init(&mut self, el: &ActiveEventLoop) {
        let attr = WindowAttributes::default().with_title("Terminal").with_inner_size(DISPLAY_SIZE);
        let window = Rc::new(el.create_window(attr).unwrap());
        window.set_window_icon(Icon::from_rgba(ICON_RGBA.to_vec(), ICON_SIZE, ICON_SIZE).ok());
        window.set_ime_allowed(true);
        window.set_ime_purpose(ImePurpose::Terminal);
        self.window = Some(window.clone());

        let context = Context::new(window.clone()).unwrap();
        self.surface = Some(Surface::new(&context, window.clone()).unwrap());
        self.surface_width = DISPLAY_SIZE.width as usize;
        self.surface_height = DISPLAY_SIZE.height as usize;
        self.resize_if_needed();

        #[cfg(unix)]
        {
            use io_uring::{IoUring, opcode, types};

            const BUF_SIZE: usize = 4096;

            let output_duration = self.output_duration.clone();
            let master = self.master.clone();
            let terminal = self.terminal.clone();
            let dirty = self.dirty.clone();
            std::thread::spawn(move || {
                let mut buf = vec![0u8; BUF_SIZE].into_boxed_slice();
                let mut ring = IoUring::new(2).unwrap();
                let master_fd = master.as_raw_fd();

                let read_e = opcode::Read::new(types::Fd(master_fd), buf.as_mut_ptr(), BUF_SIZE as u32).build();
                unsafe { ring.submission().push(&read_e).unwrap() };

                loop {
                    ring.submit_and_wait(1).unwrap();

                    {
                        let mut cq = ring.completion();
                        cq.sync();

                        for cqe in cq {
                            let result = cqe.result();
                            let time = Instant::now();

                            if result > 0 {
                                terminal.lock().unwrap().process(&buf[..result as usize]);
                                dirty.store(true, Ordering::Relaxed);
                            } else if result == 0 {
                                process::exit(0);
                            } else {
                                let err = -result;
                                if err == libc::EIO {
                                    process::exit(0);
                                }
                                eprintln!("io_uring read error: errno={}", err);
                                process::exit(1);
                            }
                            output_duration.lock().as_mut().unwrap().add_assign(Instant::now() - time);
                        }
                    }

                    {
                        let mut sq = ring.submission();
                        let read_e = opcode::Read::new(types::Fd(master_fd), buf.as_mut_ptr(), BUF_SIZE as u32).build();
                        unsafe { sq.push(&read_e).unwrap() };
                        sq.sync();
                    }
                }
            });

            let output_duration = self.output_duration.clone();
            let master = self.master.clone();
            let ansi_receiver = self.ansi_receiver.take().unwrap();
            std::thread::spawn(move || {
                while let Ok(key) = ansi_receiver.recv() {
                    let time = Instant::now();
                    let _ = unsafe { libc::write(master.as_raw_fd(), key.as_ptr() as *const libc::c_void, key.len()) };
                    output_duration.lock().as_mut().unwrap().add_assign(Instant::now() - time);
                }
            });
        }

        #[cfg(windows)]
        {
            let output_duration = self.output_duration.clone();
            let output_read = self.process.output_read.0 as usize;
            let terminal = self.terminal.clone();
            let dirty = self.dirty.clone();
            std::thread::spawn(move || {
                let output_read = HANDLE(output_read as *mut std::ffi::c_void);
                let mut temp = [0u8; 4096];
                loop {
                    let mut read: u32 = 0;
                    let ok = unsafe { ReadFile(output_read, Some(&mut temp), Some(&mut read), None) };
                    let time = Instant::now();
                    if ok.is_ok() {
                        if read == 0 {
                            eprintln!("winpty: output closed (read=0)");
                            break;
                        }
                        terminal.lock().unwrap().process(&temp[..read as usize]);
                        dirty.store(true, Ordering::Relaxed);
                    } else {
                        let err = unsafe { GetLastError() };
                        eprintln!("winpty: ReadFile failed err={}", err.0);
                        if err == ERROR_BROKEN_PIPE || err == ERROR_NO_DATA {
                            break;
                        }
                        eprintln!("Error reading from PTY: winerr={}", err.0);
                        break;
                    }
                    output_duration.lock().as_mut().unwrap().add_assign(Instant::now() - time);
                }
            });

            let output_duration = self.output_duration.clone();
            let input_write = self.process.input_write.0 as usize;
            let ansi_receiver = self.ansi_receiver.take().unwrap();
            std::thread::spawn(move || {
                let input_write = HANDLE(input_write as *mut std::ffi::c_void);
                while let Ok(key) = ansi_receiver.recv() {
                    let time = Instant::now();
                    let mut written: u32 = 0;
                    let _ = unsafe { WriteFile(input_write, Some(&key), Some(&mut written), None) };
                    output_duration.lock().as_mut().unwrap().add_assign(Instant::now() - time);
                }
            });
        }
    }

    fn resize_if_needed(&mut self) {
        let (width, height) = (self.surface_width, self.surface_height);
        if self.old_width == width && self.old_height == height {
            return;
        }
        (self.old_width, self.old_height) = (width, height);

        self.surface
            .as_mut()
            .unwrap()
            .resize(NonZeroU32::new(width as u32).unwrap(), NonZeroU32::new(height as u32).unwrap())
            .unwrap();

        {
            let mut terminal = self.terminal.lock().unwrap();
            terminal.resize(width as u32, height as u32);
            if let Err(err) = set_pty_winsize(
                #[cfg(unix)]
                self.master.as_raw_fd(),
                #[cfg(windows)]
                &self.process,
                terminal.rows() as u16,
                terminal.cols() as u16,
                width as u16,
                height as u16,
            ) {
                eprintln!("Failed to update PTY size: {err}");
            }
        }
    }

    fn flush(&mut self) {
        if !self.dirty.load(Ordering::Relaxed) {
            return;
        }

        self.resize_if_needed();

        self.dirty.store(false, Ordering::Relaxed);

        let mut buffer = self.surface.as_mut().unwrap().buffer_mut().unwrap();

        let mut drawable = Drawable::from_softbuffer(&mut buffer);

        self.terminal.lock().unwrap().flush(&mut drawable, self.app_start_time.elapsed());

        buffer.present().unwrap();

        self.frame_counter += 1;
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            self.init(el);
        }
    }

    fn new_events(&mut self, _: &ActiveEventLoop, _: StartCause) {
        self.input_begin_time = Instant::now();
        self.input_duration = Duration::ZERO;
        *self.output_duration.lock().unwrap() = Duration::ZERO;
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let frame_begin_time = Instant::now();

        self.flush();

        let frame_end_time = Instant::now();
        self.frame_duration = frame_end_time - frame_begin_time;
        self.frame_times.push_back(frame_end_time);

        if self.alloc_info_print_time.is_none() {
            clear_alloc_counts();
            self.alloc_info_print_time = Some(Instant::now());
        } else {
            if Instant::now() - self.alloc_info_print_time.unwrap() >= Duration::from_secs(1) {
                print_alloc_counts();
                clear_alloc_counts();
                self.alloc_info_print_time = Some(Instant::now());
            }
        }

        let target_fps =
            event_loop.primary_monitor().and_then(|m| m.refresh_rate_millihertz()).unwrap_or(60000) as f32 / 1000.0;
        let target_frame_duration = Duration::from_micros((1000000.0 / target_fps) as u64);

        self.max_frame_history = target_fps.ceil() as usize + 1;
        while self.frame_times.len() > self.max_frame_history {
            self.frame_times.pop_front();
        }

        let fps = if self.frame_times.len() > 4 {
            4.0 / (self.frame_times[self.frame_times.len() - 1] - self.frame_times[self.frame_times.len() - 5])
                .as_secs_f32()
        } else {
            0.0
        };

        let avg_fps = if self.frame_times.len() > 10 {
            (self.frame_times.len() - 1) as f32
                / (*self.frame_times.back().unwrap() - *self.frame_times.front().unwrap()).as_secs_f32()
        } else {
            0.0
        };

        let min_fps = if self.frame_times.len() > 10 {
            let mut min_fps = f32::MAX;
            for i in 0..self.frame_times.len() - 1 {
                let fps = 1.0 / (self.frame_times[i + 1] - self.frame_times[i]).as_secs_f32();
                if fps < min_fps {
                    min_fps = fps;
                }
            }
            min_fps
        } else {
            0.0
        };

        print!(
            "\r\x1b[KFPS:{:6.1} | Avg:{:6.1} | Min:{:6.1} | Frame:{:6.1}ms | Input:{:7.3}ms | Output:{:7.3}ms",
            fps,
            avg_fps,
            min_fps,
            self.frame_duration.as_millis_f32(),
            self.input_duration.as_millis_f32(),
            self.output_duration.lock().unwrap().as_millis_f32(),
        );
        std::io::stdout().flush().unwrap();

        let wait_until = ControlFlow::WaitUntil(self.input_begin_time + target_frame_duration);
        event_loop.set_control_flow(wait_until);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let begin_time = Instant::now();
        match event {
            WindowEvent::Resized(size) => {
                self.surface_width = size.width as usize;
                self.surface_height = size.height as usize;
                self.dirty.store(true, Ordering::Relaxed);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.terminal.lock().unwrap().handle_event(event);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.terminal.lock().unwrap().handle_event(modifiers);
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                self.terminal.lock().unwrap().user_input(text.as_bytes());
            }
            WindowEvent::CursorMoved { position, .. } => {
                let x = position.x as i32;
                let y = position.y as i32;
                self.terminal.lock().unwrap().handle_event(Event::PointerMove(x, y));
                self.dirty.store(true, Ordering::Relaxed);
            }
            WindowEvent::CursorEntered { .. } => {
                self.terminal.lock().unwrap().handle_event(Event::PointerEnter);
                self.dirty.store(true, Ordering::Relaxed);
            }
            WindowEvent::CursorLeft { .. } => {
                self.terminal.lock().unwrap().handle_event(Event::PointerLeave);
                self.dirty.store(true, Ordering::Relaxed);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta {
                    MouseScrollDelta::LineDelta(_, lines) => lines,
                    MouseScrollDelta::PixelDelta(delta) => delta.y as f32 * TOUCHPAD_SCROLL_MULTIPLIER,
                };
                if self.terminal.lock().unwrap().handle_event(Event::Scroll(lines as i32)) {
                    self.dirty.store(true, Ordering::Relaxed);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if self.terminal.lock().unwrap().handle_event((state, button)) {
                    self.dirty.store(true, Ordering::Relaxed);
                }
            }
            WindowEvent::ThemeChanged(theme) => {
                // TODO
                eprintln!("Theme changed: {:?}", theme);
            }
            WindowEvent::RedrawRequested => {
                // TODO
            }
            _ => {}
        }
        self.input_duration += Instant::now() - begin_time;
    }
}
