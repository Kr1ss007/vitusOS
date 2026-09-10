//! Terminow: GPU-Accelerated Spatial Terminal Emulator for vitusOS.
//!
//! Aligned with Part 36 of specification.
//! Features Mid Altitude Glass (20px Kawase Blur), JetBrains Mono subpixel typography,
//! Space Orange (#FF6B00) cursor caret with spring pulse, and real PTY process management.

use animus_core::event_bus::EventBus;
use animus_physics::spring::{SpringProfile, SpringSolver};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::info;

static TAB_ID_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorRgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

pub const COLOR_SPACE_ORANGE: ColorRgb = ColorRgb::new(255, 107, 0);   // #FF6B00
pub const COLOR_NEON_BLUE: ColorRgb    = ColorRgb::new(0, 122, 255);   // #007AFF
pub const COLOR_WARM_BLACK: ColorRgb   = ColorRgb::new(26, 18, 8);     // #1A1208
pub const COLOR_FOREGROUND: ColorRgb   = ColorRgb::new(242, 242, 242); // #F2F2F2
pub const COLOR_GREEN: ColorRgb        = ColorRgb::new(48, 209, 88);   // #30D158

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCell {
    pub ch: char,
    pub fg: ColorRgb,
    pub bg: ColorRgb,
    pub is_bold: bool,
    pub is_dim: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: COLOR_FOREGROUND,
            bg: COLOR_WARM_BLACK,
            is_bold: false,
            is_dim: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalTab {
    pub id: u64,
    pub title: String,
    pub working_dir: String,
    pub cols: usize,
    pub rows: usize,
    pub cursor_col: usize,
    pub cursor_row: usize,
    pub lines: Vec<String>,
    pub command_history: Vec<String>,
    #[cfg(unix)]
    #[serde(skip)]
    pub master_fd: Option<i32>,
    pub child_pid: Option<i32>,
}

impl TerminalTab {
    pub fn new(title: impl Into<String>, working_dir: impl Into<String>) -> Self {
        let id = TAB_ID_SEQ.fetch_add(1, Ordering::SeqCst);
        let mut initial_lines = Vec::new();
        initial_lines.push("vitusOS Darwin Engine v1.0.0 (x86_64-pc-vitusos-gnu)".to_string());
        initial_lines.push("Welcome to Terminow — Space Orange GPU Terminal".to_string());
        initial_lines.push("".to_string());

        let mut tab = Self {
            id,
            title: title.into(),
            working_dir: working_dir.into(),
            cols: 80,
            rows: 24,
            cursor_col: 19,
            cursor_row: 4,
            lines: initial_lines,
            command_history: Vec::new(),
            #[cfg(unix)]
            master_fd: None,
            child_pid: None,
        };

        tab.spawn_pty_process();
        tab
    }

    pub fn spawn_pty_process(&mut self) {
        #[cfg(unix)]
        {
            use nix::pty::openpty;
            use nix::unistd::{fork, ForkResult, setsid, dup2};
            use std::ffi::CString;

            if let Ok(pty) = openpty(None, None) {
                use std::os::unix::io::AsRawFd;
                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child }) => {
                        let master_raw = pty.master.as_raw_fd();
                        // Leak the master fd so it stays alive — we own it as raw i32
                        std::mem::forget(pty.master);
                        use nix::fcntl::{fcntl, FcntlArg, OFlag};
                        let _ = fcntl(master_raw, FcntlArg::F_SETFL(OFlag::O_NONBLOCK));
                        self.master_fd = Some(master_raw);
                        self.child_pid = Some(child.as_raw());
                        info!("Terminow: Spawned real PTY child PID {}", child);
                    }
                    Ok(ForkResult::Child) => {
                        let _ = setsid();
                        let slave_raw = pty.slave.as_raw_fd();
                        let _ = dup2(slave_raw, 0);
                        let _ = dup2(slave_raw, 1);
                        let _ = dup2(slave_raw, 2);
                        let shell = CString::new("/bin/bash").unwrap_or_default();
                        let args = [shell.clone()];
                        let _ = nix::unistd::execvp(&shell, &args);
                        std::process::exit(1);
                    }
                    Err(e) => {
                        tracing::warn!("Terminow: Fork failed: {}", e);
                    }
                }
            }
        }
    }

    /// Reads output bytes from the PTY master into terminal line buffers.
    pub fn read_pty_output(&mut self) {
        #[cfg(unix)]
        {
            if let Some(fd) = self.master_fd {
                use nix::unistd::read;
                let mut buf = [0u8; 4096];
                while let Ok(n) = read(fd, &mut buf[..]) {
                    if n == 0 {
                        break;
                    }
                    let s = String::from_utf8_lossy(&buf[..n]);
                    for ch in s.chars() {
                        if ch == '\n' {
                            self.new_line();
                        } else if ch == '\r' {
                            self.cursor_col = 0;
                        } else if ch == '\x08' {
                            if self.cursor_col > 0 {
                                self.cursor_col -= 1;
                                if let Some(line) = self.lines.last_mut() {
                                    line.pop();
                                }
                            }
                        } else {
                            if self.lines.is_empty() {
                                self.lines.push(String::new());
                            }
                            if let Some(line) = self.lines.last_mut() {
                                line.push(ch);
                            }
                            self.cursor_col += 1;
                        }
                    }
                }
            }
        }
    }

    pub fn write_text(&mut self, text: &str) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        let last_idx = self.lines.len() - 1;
        self.lines[last_idx].push_str(text);
        self.cursor_col += text.chars().count();

        #[cfg(unix)]
        {
            if let Some(fd) = self.master_fd {
                use nix::unistd::write;
                use std::os::fd::BorrowedFd;
                let bfd = unsafe { BorrowedFd::borrow_raw(fd) };
                let _ = write(bfd, text.as_bytes());
            }
        }
    }

    pub fn new_line(&mut self) {
        self.lines.push(String::new());
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn execute_input(&mut self, input: &str) {
        let trimmed = input.trim();
        self.command_history.push(trimmed.to_string());
        self.new_line();

        if trimmed == "clear" {
            self.lines.clear();
            return;
        }

        #[cfg(unix)]
        {
            if let Some(fd) = self.master_fd {
                use nix::unistd::write;
                use std::os::fd::BorrowedFd;
                let bfd = unsafe { BorrowedFd::borrow_raw(fd) };
                let mut cmd_bytes = input.as_bytes().to_vec();
                cmd_bytes.push(b'\n');
                let _ = write(bfd, &cmd_bytes);
                self.read_pty_output();
            }
        }
    }
}

pub struct Terminow {
    pub surface: crate::AENativeSurface,
    pub content: RwLock<animus_appkit::layout::surface::AEContent>,
    pub tabs: RwLock<Vec<TerminalTab>>,
    pub active_tab_idx: RwLock<usize>,
    pub cursor_pulse: RwLock<SpringSolver>, // SPRING_SELECTION (400, 28)
    pub font_size: RwLock<f32>,
    pub current_input: RwLock<String>,
    #[allow(dead_code)]
    bus: EventBus,
}

impl Terminow {
    pub fn new(bus: EventBus) -> Self {
        let mut surface = crate::AENativeSurface::new("terminow", "Terminow");
        let _ = surface.connect();
        
        surface.set_menu_json(r#"{"items": [{"label": "Terminow"}, {"label": "File"}, {"label": "Edit"}, {"label": "View"}]}"#);
        
        let initial_tab = TerminalTab::new("bash", "~");
        Self {
            surface,
            content: RwLock::new(animus_appkit::layout::surface::AEContent { x: 0.0, y: 0.0, width: 800.0, height: 600.0 }),
            tabs: RwLock::new(vec![initial_tab]),
            active_tab_idx: RwLock::new(0),
            cursor_pulse: RwLock::new(SpringSolver::new(1.0, SpringProfile::Selection)),
            font_size: RwLock::new(13.0),
            current_input: RwLock::new(String::new()),
            bus,
        }
    }

    pub fn new_tab(&self, title: &str) -> u64 {
        let mut tabs = self.tabs.write();
        let tab = TerminalTab::new(title, "~");
        let id = tab.id;
        tabs.push(tab);
        *self.active_tab_idx.write() = tabs.len() - 1;
        info!("Terminow: Spawned new tab #{} ('{}')", id, title);
        id
    }

    pub fn close_tab(&self, idx: usize) {
        let mut tabs = self.tabs.write();
        if tabs.len() > 1 && idx < tabs.len() {
            tabs.remove(idx);
            let mut active = self.active_tab_idx.write();
            if *active >= tabs.len() {
                *active = tabs.len() - 1;
            }
        }
    }

    pub fn input_char(&self, ch: char) {
        let mut input = self.current_input.write();
        input.push(ch);
        let mut tabs = self.tabs.write();
        let active = *self.active_tab_idx.read();
        if let Some(tab) = tabs.get_mut(active) {
            tab.write_text(&ch.to_string());
        }
    }

    pub fn submit_command(&self) {
        let mut input = self.current_input.write();
        let cmd = input.clone();
        input.clear();

        let mut tabs = self.tabs.write();
        let active = *self.active_tab_idx.read();
        if let Some(tab) = tabs.get_mut(active) {
            tab.execute_input(&cmd);
        }
    }

    pub fn update(&self, dt: f32) {
        self.cursor_pulse.write().update(dt);
        let mut tabs = self.tabs.write();
        let active = *self.active_tab_idx.read();
        if let Some(tab) = tabs.get_mut(active) {
            tab.read_pty_output();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminow_tab_management_and_execution() {
        let bus = EventBus::new();
        let term = Terminow::new(bus);

        assert_eq!(term.tabs.read().len(), 1);
        let _tab2_id = term.new_tab("compile");
        assert_eq!(term.tabs.read().len(), 2);
        assert_eq!(*term.active_tab_idx.read(), 1);

        // Input command in active tab
        term.input_char('e');
        term.input_char('c');
        term.input_char('h');
        term.input_char('o');
        term.input_char(' ');
        term.input_char('h');
        term.input_char('i');
        term.submit_command();

        {
            let tabs = term.tabs.read();
            let tab = &tabs[1];
            assert_eq!(tab.command_history.last().map(|s| s.as_str()), Some("echo hi"));
        }

        term.close_tab(1);
        assert_eq!(term.tabs.read().len(), 1);
    }
}
