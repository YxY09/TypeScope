//! 常驻采集进程。窗口隐藏，只在自己的消息循环里做聚合。
//! 关闭看板不会结束这个进程。完全退出时会卸下 Raw Input。

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicPtr, Ordering};
use rusqlite::Connection;
use typescope::aggregate::{Engine, Mods};
use typescope::platform::{self, ForegroundCache, InstanceLock, SharedRing};
use typescope::store;
use typescope_proto::{classify, pieces, ClassCounts, AGENT_MUTEX, CONTROL_EVENT};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, HWND, LPARAM, LRESULT, WAIT_OBJECT_0, WPARAM};
use windows_sys::Win32::System::RemoteDesktop::{WTSRegisterSessionNotification, WTSUnRegisterSessionNotification, NOTIFY_FOR_THIS_SESSION};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::System::Threading::CreateEventW;
use windows_sys::Win32::UI::Input::{
    GetCurrentInputMessageSource, GetRawInputData, RegisterRawInputDevices, IMO_INJECTED, IMO_SYSTEM, INPUT_MESSAGE_SOURCE, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RIDEV_INPUTSINK, RIDEV_REMOVE, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows_sys::Win32::Graphics::Gdi::{CreateBitmap, CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
use windows_sys::Win32::UI::Shell::{Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW, GetCursorPos,
    MsgWaitForMultipleObjects, PeekMessageW, PostMessageW, PostQuitMessage, RegisterClassExW, SetForegroundWindow, TrackPopupMenu, TranslateMessage, CS_HREDRAW,
    CS_VREDRAW, HICON, HMENU, ICONINFO, MF_STRING, MSG, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, PM_REMOVE, QS_ALLINPUT, RI_KEY_BREAK, TPM_NONOTIFY, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, WM_APP, WM_COMMAND, WM_DESTROY, WM_INPUT, WM_LBUTTONUP, WM_NULL, WM_POWERBROADCAST, WM_QUIT, WM_RBUTTONUP,
    WM_WTSSESSION_CHANGE, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP, WTS_SESSION_LOCK, WTS_SESSION_UNLOCK,
};

const WM_TRAY: u32 = WM_APP + 20;
const CMD_OPEN: usize = 1;
const CMD_PAUSE: usize = 2;
const CMD_QUIT: usize = 3;
const FLUSH_MS: u64 = 30_000;

#[derive(Clone, Copy, PartialEq)]
enum TrayState {
    Normal,
    Paused,
    Error,
}

// 托盘图标：16×16 像素直接绘制。圆角方块底色区分状态，
// 正常态是三根白色阶梯柱（输入节奏），暂停态是双竖条，异常态是感叹号。
fn draw_tray_icon(state: TrayState) -> HICON {
    const S: usize = 16;
    let base: u32 = match state {
        TrayState::Normal => 0xFF17_1717,
        TrayState::Paused => 0xFF73_7373,
        TrayState::Error => 0xFF40_4040,
    };
    let fg = 0xFFFF_FFFF;
    let mut px = vec![0u32; S * S];
    let inside = |x: usize, y: usize| -> bool {
        if !(1..15).contains(&x) || !(1..15).contains(&y) {
            return false;
        }
        let dx = if x < 4 { 4 - x } else if x >= 12 { x - 11 } else { 0 };
        let dy = if y < 4 { 4 - y } else if y >= 12 { y - 11 } else { 0 };
        dx * dx + dy * dy <= 9
    };
    for y in 0..S {
        for x in 0..S {
            if inside(x, y) {
                px[y * S + x] = base;
            }
        }
    }
    let rect = |px: &mut [u32], x0: usize, y0: usize, x1: usize, y1: usize, color: u32| {
        for y in y0..y1 {
            for x in x0..x1 {
                if inside(x, y) {
                    px[y * S + x] = color;
                }
            }
        }
    };
    match state {
        TrayState::Normal => {
            // 与窗口图标、界面顶栏图标完全统一的 TypeScope 标志：
            // 上方三根渐高灰阶柱（Scope 统计趋势）+ 下方横向空格键底座（Type 键盘）
            rect(&mut px, 4, 7, 6, 10, 0xFF8C_8C8C);
            rect(&mut px, 7, 5, 9, 10, 0xFFC8_C8C8);
            rect(&mut px, 10, 3, 12, 10, fg);
            rect(&mut px, 4, 11, 12, 13, fg);
        }
        TrayState::Paused => {
            rect(&mut px, 5, 4, 7, 9, fg);
            rect(&mut px, 9, 4, 11, 9, fg);
            rect(&mut px, 4, 11, 12, 13, 0xFFD4_D4D4);
        }
        TrayState::Error => {
            rect(&mut px, 7, 4, 9, 9, fg);
            rect(&mut px, 7, 11, 9, 13, fg);
        }
    }

    unsafe {
        let mut bi = BITMAPINFO { bmiHeader: std::mem::zeroed(), bmiColors: [std::mem::zeroed(); 1] };
        bi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bi.bmiHeader.biWidth = S as i32;
        bi.bmiHeader.biHeight = -(S as i32);
        bi.bmiHeader.biPlanes = 1;
        bi.bmiHeader.biBitCount = 32;
        bi.bmiHeader.biCompression = BI_RGB;
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let hbm_color = CreateDIBSection(std::ptr::null_mut(), &bi, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
        if hbm_color.is_null() {
            return std::ptr::null_mut();
        }
        if !bits.is_null() {
            std::ptr::copy_nonoverlapping(px.as_ptr(), bits as *mut u32, S * S);
        }
        let mask_bytes = [0u8; 32];
        let hbm_mask = CreateBitmap(S as i32, S as i32, 1, 1, mask_bytes.as_ptr() as *const core::ffi::c_void);
        let info = ICONINFO { fIcon: 1, xHotspot: 0, yHotspot: 0, hbmMask: hbm_mask, hbmColor: hbm_color };
        let icon = CreateIconIndirect(&info);
        DeleteObject(hbm_color);
        DeleteObject(hbm_mask);
        icon
    }
}

static AGENT: AtomicPtr<Agent> = AtomicPtr::new(std::ptr::null_mut());

struct Cfg {
    paused: bool,
    text_stats: bool,
    segment: bool,
    app_stats: bool,
    idle_ms: u64,
    exclude: Vec<String>,
}

#[derive(Clone)]
struct ClassRow {
    day: u32,
    hour: u8,
    counts: ClassCounts,
    commits: u32,
}

struct TextAccum {
    rows: Vec<ClassRow>,
    chars: Vec<(u32, String, u32)>,
    words: Vec<(u32, String, u32)>,
}

impl TextAccum {
    fn dirty(&self) -> bool {
        !self.rows.is_empty() || !self.chars.is_empty() || !self.words.is_empty()
    }

    fn clear(&mut self) {
        self.rows.clear();
        self.chars.clear();
        self.words.clear();
    }
}

#[derive(Clone)]
struct AppRow {
    day: u32,
    hour: u8,
    exe: String,
    physical: u32,
    active_ms: u64,
}

struct ModState {
    shift: bool,
    ctrl: bool,
    alt: bool,
    win: bool,
}

impl ModState {
    fn clear(&mut self) {
        *self = Self { shift: false, ctrl: false, alt: false, win: false };
    }

    fn bits(&self) -> Mods {
        Mods { shift: self.shift, ctrl: self.ctrl, alt: self.alt, win: self.win }
    }

    fn set(&mut self, vk: u16, down: bool) {
        match vk {
            0x10 | 0xA0 | 0xA1 => self.shift = down,
            0x11 | 0xA2 | 0xA3 => self.ctrl = down,
            0x12 | 0xA4 | 0xA5 => self.alt = down,
            0x5B | 0x5C => self.win = down,
            _ => {}
        }
    }
}

struct Agent {
    engine: Engine,
    conn: Connection,
    ring: Option<SharedRing>,
    control: HANDLE,
    hwnd: HWND,
    tray: bool,
    tray_hicon: HICON,
    tray_state: TrayState,
    last_flush_failed: bool,
    cfg: Cfg,
    text: TextAccum,
    apps: Vec<AppRow>,
    fg: ForegroundCache,
    mods: ModState,
    last_flush: u64,
    quit: bool,
    shutting_down: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--check") {
        std::process::exit(run_check());
    }
    if args.iter().any(|arg| arg == "--quit") {
        std::process::exit(run_quit());
    }
    if args.iter().any(|arg| arg == "--install-autostart") {
        let ok = platform::set_autostart(true);
        platform::write_report("autostart.txt", if ok { "已写入当前用户的开机启动。\r\n" } else { "没有写入开机启动。\r\n" });
        return;
    }
    if args.iter().any(|arg| arg == "--remove-autostart") {
        platform::set_autostart(false);
        platform::write_report("autostart.txt", "已去掉开机启动。\r\n");
        return;
    }
    let _lock = match InstanceLock::acquire(AGENT_MUTEX) {
        Some(lock) => lock,
        None => return,
    };
    if let Err(error) = run_agent() {
        platform::write_report("agent-error.txt", &format!("{error}\r\n"));
    }
}

fn run_check() -> i32 {
    let mut lines = Vec::new();
    match store::open(&platform::db_path()) {
        Ok(_) => lines.push("数据库可以打开。".to_string()),
        Err(error) => lines.push(format!("数据库打开失败：{error}")),
    }
    match platform::map_ring() {
        Some(_) => lines.push("提交用的共享内存可以创建。".to_string()),
        None => lines.push(format!("共享内存创建失败，错误号 {}。", platform::last_error())),
    }
    let registered = probe_raw_input();
    if registered {
        lines.push("Raw Input 键盘登记成功，并已立即卸下。本次检查没有把按键写入数据库。".to_string());
    } else {
        lines.push(format!("Raw Input 登记失败，错误号 {}。", platform::last_error()));
    }
    lines.push("检查过程没有修改小狼毫配置，也没有登记开机启动。".to_string());
    platform::write_report("check.txt", &lines.join("\r\n"));
    if registered { 0 } else { 1 }
}

fn probe_raw_input() -> bool {
    unsafe {
        let class_name = platform::wide("TypeScopeCheck");
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(DefWindowProcW),
            hInstance: platform::module_handle(),
            lpszClassName: class_name.as_ptr(),
            ..std::mem::zeroed()
        };
        if RegisterClassExW(&class) == 0 && platform::last_error() != 1410 {
            return false;
        }
        let title = platform::wide("TypeScopeCheck");
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            platform::module_handle(),
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return false;
        }
        let device = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        let ok = RegisterRawInputDevices(&device, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32);
        let mut remove = device;
        remove.dwFlags = RIDEV_REMOVE;
        remove.hwndTarget = std::ptr::null_mut();
        RegisterRawInputDevices(&remove, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32);
        DestroyWindow(hwnd);
        ok != 0
    }
}

fn run_quit() -> i32 {
    if !platform::agent_running() {
        platform::write_report("quit.txt", "采集进程没有在运行。\r\n");
        return 0;
    }
    if let Ok(conn) = store::open(&platform::db_path()) {
        let _ = store::meta_set(&conn, "command", "quit");
    }
    platform::signal_control();
    for _ in 0..30 {
        if !platform::agent_running() {
            platform::write_report("quit.txt", "采集进程已退出，键盘登记已卸下。\r\n");
            return 0;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    platform::write_report("quit.txt", "已发出退出请求，但进程还没有在 3 秒内结束。\r\n");
    1
}

fn run_agent() -> Result<(), String> {
    let conn = store::open(&platform::db_path()).map_err(|error| error.to_string())?;
    let _ = store::meta_set(&conn, "command", "");
    let cfg = load_cfg(&conn);
    let control_name = platform::wide(CONTROL_EVENT);
    let control = unsafe { CreateEventW(std::ptr::null(), 0, 0, control_name.as_ptr()) };
    if control.is_null() {
        return Err("控制事件创建失败".to_string());
    }
    let ring = platform::map_ring();
    let mut agent = Agent {
        engine: Engine::default(),
        conn,
        ring,
        control,
        hwnd: std::ptr::null_mut(),
        tray: false,
        tray_hicon: std::ptr::null_mut(),
        tray_state: TrayState::Normal,
        last_flush_failed: false,
        cfg,
        text: TextAccum { rows: Vec::new(), chars: Vec::new(), words: Vec::new() },
        apps: Vec::new(),
        fg: ForegroundCache::default(),
        mods: ModState { shift: false, ctrl: false, alt: false, win: false },
        last_flush: unsafe { GetTickCount64() },
        quit: false,
        shutting_down: false,
    };
    agent.engine.paused = agent.cfg.paused;
    agent.engine.idle_ms = agent.cfg.idle_ms;
    let agent = Box::into_raw(Box::new(agent));
    AGENT.store(agent, Ordering::Release);
    if !create_window(unsafe { &mut *agent }) {
        let err = platform::last_error();
        AGENT.store(std::ptr::null_mut(), Ordering::Release);
        let boxed = unsafe { Box::from_raw(agent) };
        if !boxed.control.is_null() {
            unsafe { CloseHandle(boxed.control) };
        }
        return Err(format!("采集窗口创建失败，错误号 {err}"));
    }
    loop_messages();
    shutdown();
    let _ = unsafe { Box::from_raw(agent) };
    AGENT.store(std::ptr::null_mut(), Ordering::Release);
    Ok(())
}

fn load_cfg(conn: &Connection) -> Cfg {
    let flag = |key, default| store::meta_get(conn, key).ok().flatten().map(|value| value == "1").unwrap_or(default);
    let idle_ms = store::meta_get(conn, "idle_ms")
        .ok()
        .flatten()
        .and_then(|value| value.parse().ok())
        .unwrap_or(typescope::aggregate::IDLE_MS_DEFAULT);
    Cfg {
        paused: flag("paused", false),
        text_stats: flag("text_stats", false),
        segment: flag("segment", false),
        app_stats: flag("app_stats", false),
        idle_ms,
        exclude: platform::read_exclude(),
    }
}

fn create_window(agent: &mut Agent) -> bool {
    unsafe {
        let class_name = platform::wide("TypeScopeAgent");
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(agent_proc),
            hInstance: platform::module_handle(),
            lpszClassName: class_name.as_ptr(),
            ..std::mem::zeroed()
        };
        if RegisterClassExW(&class) == 0 && platform::last_error() != 1410 {
            return false;
        }
        let title = platform::wide("TypeScope");
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            platform::module_handle(),
            std::ptr::null(),
        );
        if hwnd.is_null() {
            return false;
        }
        agent.hwnd = hwnd;
        let device = RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x06,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        if RegisterRawInputDevices(&device, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            let _ = store::meta_set(&agent.conn, "last_error", &format!("Raw Input 登记失败 {}", platform::last_error()));
        }
        WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
        agent.tray_add();
        true
    }
}

unsafe extern "system" fn agent_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let agent = AGENT.load(Ordering::Acquire);
    if agent.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    (*agent).on_message(hwnd, msg, wparam, lparam)
}

fn current() -> &'static mut Agent {
    unsafe { &mut *AGENT.load(Ordering::Acquire) }
}

fn loop_messages() {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        loop {
            let (control, ring_event, quit) = {
                let agent = current();
                let ring_event = agent.ring.as_ref().map(|ring| ring.event).unwrap_or(std::ptr::null_mut());
                (agent.control, ring_event, agent.quit)
            };
            if quit {
                break;
            }
            let handles = [control, ring_event];
            let count = if ring_event.is_null() { 1 } else { 2 };
            let wake = MsgWaitForMultipleObjects(count, handles.as_ptr(), 0, FLUSH_MS as u32, QS_ALLINPUT);
            if wake == WAIT_OBJECT_0 {
                current().apply_control();
            } else if count == 2 && wake == WAIT_OBJECT_0 + 1 {
                current().drain_commits();
            }
            while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    current().quit = true;
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            let agent = current();
            agent.flush_due();
            if agent.quit {
                break;
            }
        }
    }
}

fn shutdown() {
    let (hwnd, control) = {
        let agent = current();
        if agent.shutting_down {
            return;
        }
        agent.shutting_down = true;
        agent.engine.suspend();
        agent.flush();
        store::wal_checkpoint(&agent.conn);
        let hwnd = agent.hwnd;
        let control = agent.control;
        agent.tray_delete();
        agent.hwnd = std::ptr::null_mut();
        agent.control = std::ptr::null_mut();
        (hwnd, control)
    };
    unsafe {
        if !hwnd.is_null() {
            let device = RAWINPUTDEVICE {
                usUsagePage: 0x01,
                usUsage: 0x06,
                dwFlags: RIDEV_REMOVE,
                hwndTarget: std::ptr::null_mut(),
            };
            RegisterRawInputDevices(&device, 1, std::mem::size_of::<RAWINPUTDEVICE>() as u32);
            WTSUnRegisterSessionNotification(hwnd);
            DestroyWindow(hwnd);
        }
        if !control.is_null() {
            CloseHandle(control);
        }
    }
}

impl Agent {

    fn on_message(&mut self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_INPUT => {
                self.on_input(lparam);
                0
            }
            WM_POWERBROADCAST => {
                let event = wparam as u32;
                if event == PBT_APMSUSPEND {
                    self.engine.suspend();
                    self.mods.clear();
                } else if event == PBT_APMRESUMEAUTOMATIC {
                    self.engine.clear_downs();
                    self.mods.clear();
                }
                1
            }
            WM_WTSSESSION_CHANGE => {
                let event = wparam as u32;
                if event == WTS_SESSION_LOCK {
                    self.engine.suspend();
                    self.mods.clear();
                } else if event == WTS_SESSION_UNLOCK {
                    self.engine.clear_downs();
                    self.mods.clear();
                }
                0
            }
            WM_TRAY => {
                let mouse = lparam as u32;
                if mouse == WM_LBUTTONUP {
                    let _ = platform::open_dashboard();
                } else if mouse == WM_RBUTTONUP {
                    self.tray_menu();
                }
                0
            }
            WM_COMMAND => {
                match wparam & 0xffff {
                    CMD_OPEN => {
                        let _ = platform::open_dashboard();
                    }
                    CMD_PAUSE => {
                        if self.cfg.paused {
                            let _ = store::meta_set(&self.conn, "command", "resume");
                        } else {
                            let _ = store::meta_set(&self.conn, "command", "pause");
                        }
                        self.apply_control();
                    }
                    CMD_QUIT => self.quit = true,
                    _ => {}
                }
                0
            }
            WM_DESTROY => {
                if self.tray {
                    self.tray_delete();
                }
                unsafe { PostQuitMessage(0) };
                0
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    fn on_input(&mut self, lparam: LPARAM) {
        let sample = unsafe { read_keyboard(lparam) };
        let Some((vk, down, injected)) = sample else {
            return;
        };
        self.mods.set(vk, down);
        if self.blocked() {
            if !down {
                self.engine.observe(
                    0,
                    0,
                    typescope::aggregate::KeySample { vk, down: false, repeat: false, injected: false, at_ms: 0 },
                    Mods::default(),
                );
            }
            return;
        }
        let (day, hour, minute) = platform::local_stamp();
        let counted = self.engine.observe(
            day,
            hour,
            typescope::aggregate::KeySample { vk, down, repeat: false, injected, at_ms: platform::tick_ms() },
            self.mods.bits(),
        );
        if counted {
            self.engine.stamp_clock(day, minute);
            self.note_app(day, hour);
        }
    }

    fn blocked(&mut self) -> bool {
        if self.cfg.exclude.is_empty() {
            return false;
        }
        let exe = self.fg.current().to_ascii_lowercase();
        self.cfg.exclude.iter().any(|item| item == &exe)
    }

    fn note_app(&mut self, day: u32, hour: u8) {
        if !self.cfg.app_stats {
            return;
        }
        let exe = self.fg.current().to_ascii_lowercase();
        if exe.is_empty() || self.cfg.exclude.iter().any(|item| item == &exe) {
            return;
        }
        let gap = self.engine.last_counted_gap;
        if let Some(row) = self.apps.iter_mut().find(|row| row.day == day && row.hour == hour && row.exe == exe) {
            row.physical += 1;
            row.active_ms += gap;
            return;
        }
        if self.apps.len() >= 64 {
            return;
        }
        self.apps.push(AppRow { day, hour, exe, physical: 1, active_ms: gap });
    }

    fn drain_commits(&mut self) {
        let Some(ring) = self.ring.take() else {
            return;
        };
        let mut buf = [0u8; 768];
        while let Some(size) = ring.ring.pop(&mut buf) {
            if self.blocked() {
                continue;
            }
            let Ok(text) = std::str::from_utf8(&buf[..size]) else {
                continue;
            };
            if text.is_empty() {
                continue;
            }
            let (day, hour, _) = platform::local_stamp();
            let counts = classify(text);
            if let Some(row) = self.text.rows.iter_mut().find(|row| row.day == day && row.hour == hour) {
                add_counts(&mut row.counts, counts);
                row.commits += 1;
            } else {
                self.text.rows.push(ClassRow { day, hour, counts, commits: 1 });
            }
            if self.cfg.text_stats && !looks_like_secret(text) {
                let parts = pieces(text, self.cfg.segment);
                for ch in parts.chars {
                    bump(&mut self.text.chars, day, ch, 4000);
                }
                if self.cfg.segment {
                    for word in parts.words {
                        bump(&mut self.text.words, day, word, 2000);
                    }
                }
            }
        }
        let hello = ring.ring.hello();
        let accepted = ring.ring.accepted();
        let dropped = ring.ring.dropped();
        self.ring = Some(ring);
        let _ = store::meta_set(&self.conn, "rime_hello", if hello { "1" } else { "0" });
        let _ = store::meta_set(&self.conn, "rime_accepted", &accepted.to_string());
        let _ = store::meta_set(&self.conn, "rime_dropped", &dropped.to_string());
    }

    fn apply_control(&mut self) {
        let command = store::meta_get(&self.conn, "command").ok().flatten().unwrap_or_default();
        let _ = store::meta_set(&self.conn, "command", "");
        self.cfg = load_cfg(&self.conn);
        self.engine.paused = self.cfg.paused;
        self.engine.idle_ms = self.cfg.idle_ms;
        self.tray_tip();
        match command.as_str() {
            "pause" => {
                self.cfg.paused = true;
                self.engine.paused = true;
                let _ = store::meta_set(&self.conn, "paused", "1");
            }
            "resume" => {
                self.cfg.paused = false;
                self.engine.paused = false;
                self.engine.suspend();
                let _ = store::meta_set(&self.conn, "paused", "0");
            }
            "clear" => {
                self.engine = Engine::default();
                self.engine.paused = self.cfg.paused;
                self.engine.idle_ms = self.cfg.idle_ms;
                self.text.clear();
                self.apps.clear();
                let _ = store::clear_all(&self.conn);
            }
            "quit" => self.quit = true,
            _ => {}
        }
    }

    fn flush_due(&mut self) {
        let now = unsafe { GetTickCount64() };
        if now.saturating_sub(self.last_flush) < FLUSH_MS {
            return;
        }
        self.last_flush = now;
        self.flush();
    }

    fn flush(&mut self) {
        let engine_dirty = self.engine.has_pending();
        let text_dirty = self.text.dirty();
        let app_dirty = !self.apps.is_empty();
        if !engine_dirty && !text_dirty && !app_dirty {
            return;
        }
        let hours = self.engine.take_hours();
        let keys = self.engine.take_keys();
        let intervals = self.engine.take_intervals();
        let shortcuts = self.engine.take_shortcuts();
        let peaks = self.engine.copy_peaks();
        let text_rows = self.text.rows.clone();
        let chars = self.text.chars.clone();
        let words = self.text.words.clone();
        let apps = self.apps.clone();
        let result = (|| {
            let tx = self.conn.transaction().map_err(|error| error.to_string())?;
            store::flush_hours(&tx, &hours).map_err(|error| error.to_string())?;
            store::flush_keys(&tx, &keys).map_err(|error| error.to_string())?;
            store::flush_intervals(&tx, &intervals).map_err(|error| error.to_string())?;
            store::flush_shortcuts(&tx, &shortcuts).map_err(|error| error.to_string())?;
            for (day, row) in &peaks {
                if row.sessions == 0 && row.best_10 == 0 && row.best_30 == 0 && row.best_60 == 0 && row.first_minute < 0 {
                    continue;
                }
                store::flush_peaks(&tx, *day, *row).map_err(|error| error.to_string())?;
            }
            for row in &text_rows {
                store::flush_classes(&tx, row.day, row.hour, row.counts, row.commits)
                    .map_err(|error| error.to_string())?;
            }
            store::flush_freq_batch(&tx, "char_freq", &chars).map_err(|error| error.to_string())?;
            store::flush_freq_batch(&tx, "word_freq", &words).map_err(|error| error.to_string())?;
            for row in &apps {
                store::flush_app(&tx, row.day, row.hour, &row.exe, row.physical, row.active_ms).map_err(|error| error.to_string())?;
            }
            tx.commit().map_err(|error| error.to_string())
        })();
        if result.is_ok() {
            let (today, _, _) = platform::local_stamp();
            self.engine.mark_clean();
            self.engine.clear_flushed_sessions();
            self.engine.prune_flushed_peaks(today);
            self.text.clear();
            self.apps.clear();
            if self.last_flush_failed {
                self.last_flush_failed = false;
                self.tray_tip();
            }
        } else {
            self.engine.untake_hours(hours);
            self.engine.untake_keys(keys);
            self.engine.untake_intervals(intervals);
            self.engine.untake_shortcuts(shortcuts);
            if let Err(error) = result {
                let _ = store::meta_set(&self.conn, "last_error", &error);
            }
            if !self.last_flush_failed {
                self.last_flush_failed = true;
                self.tray_tip();
            }
        }
    }

    fn tray_state(&self) -> TrayState {
        if self.last_flush_failed {
            TrayState::Error
        } else if self.cfg.paused {
            TrayState::Paused
        } else {
            TrayState::Normal
        }
    }

    fn tray_tip_text(&self) -> &'static str {
        match self.tray_state() {
            TrayState::Normal => "TypeScope 正在统计按键",
            TrayState::Paused => "TypeScope 已暂停（右键可继续）",
            TrayState::Error => "TypeScope 写入数据库失败，会自动重试",
        }
    }

    fn tray_add(&mut self) {
        if self.hwnd.is_null() {
            return;
        }
        let icon = draw_tray_icon(self.tray_state());
        let mut data = tray_data(self.hwnd, icon, self.tray_tip_text());
        self.tray = unsafe { Shell_NotifyIconW(NIM_ADD, &data) } != 0;
        if !self.tray {
            data.uFlags = NIF_MESSAGE | NIF_TIP;
            self.tray = unsafe { Shell_NotifyIconW(NIM_ADD, &data) } != 0;
        }
        if self.tray {
            self.tray_hicon = icon;
            self.tray_state = self.tray_state();
        } else {
            unsafe { DestroyIcon(icon) };
        }
    }

    fn tray_tip(&mut self) {
        if !self.tray {
            return;
        }
        let state = self.tray_state();
        if state == self.tray_state {
            return;
        }
        let icon = draw_tray_icon(state);
        let data = tray_data(self.hwnd, icon, self.tray_tip_text());
        let ok = unsafe { Shell_NotifyIconW(NIM_MODIFY, &data) } != 0;
        if ok {
            if !self.tray_hicon.is_null() {
                unsafe { DestroyIcon(self.tray_hicon) };
            }
            self.tray_hicon = icon;
            self.tray_state = state;
        } else {
            unsafe { DestroyIcon(icon) };
        }
    }

    fn tray_delete(&mut self) {
        if self.hwnd.is_null() {
            return;
        }
        let data = tray_data(self.hwnd, std::ptr::null_mut(), "");
        unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
        if !self.tray_hicon.is_null() {
            unsafe { DestroyIcon(self.tray_hicon) };
            self.tray_hicon = std::ptr::null_mut();
        }
        self.tray = false;
    }

    fn tray_menu(&mut self) {
        unsafe {
            let menu = CreatePopupMenu();
            if menu.is_null() {
                return;
            }
            let open = platform::wide("打开看板");
            let pause = platform::wide(if self.cfg.paused { "继续统计" } else { "暂停统计" });
            let quit = platform::wide("完全退出");
            append_menu(menu, open.as_ptr(), CMD_OPEN);
            append_menu(menu, pause.as_ptr(), CMD_PAUSE);
            append_menu(menu, quit.as_ptr(), CMD_QUIT);
            let mut point = std::mem::zeroed();
            GetCursorPos(&mut point);
            SetForegroundWindow(self.hwnd);
            let picked = TrackPopupMenu(menu, TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY, point.x, point.y, 0, self.hwnd, std::ptr::null());
            PostMessageW(self.hwnd, WM_NULL, 0, 0);
            DestroyMenu(menu);
            if picked == CMD_OPEN as i32 {
                let _ = platform::open_dashboard();
            } else if picked == CMD_PAUSE as i32 {
                let command = if self.cfg.paused { "resume" } else { "pause" };
                let _ = store::meta_set(&self.conn, "command", command);
                self.apply_control();
            } else if picked == CMD_QUIT as i32 {
                self.quit = true;
            }
        }
    }
}

fn append_menu(menu: HMENU, text: *const u16, id: usize) {
    unsafe { windows_sys::Win32::UI::WindowsAndMessaging::AppendMenuW(menu, MF_STRING, id, text) };
}

fn add_counts(dst: &mut ClassCounts, src: ClassCounts) {
    dst.han += src.han;
    dst.latin += src.latin;
    dst.digit += src.digit;
    dst.punct += src.punct;
    dst.space += src.space;
    dst.other += src.other;
}

fn bump(list: &mut Vec<(u32, String, u32)>, day: u32, item: String, cap: usize) {
    if item.is_empty() {
        return;
    }
    if let Some((_, _, count)) = list.iter_mut().find(|(row_day, row_item, _)| *row_day == day && row_item == &item) {
        *count += 1;
        return;
    }
    let used = list.iter().filter(|(row_day, _, _)| *row_day == day).count();
    if used < cap {
        list.push((day, item, 1));
    }
}

fn looks_like_secret(text: &str) -> bool {
    let count = text.chars().count();
    count >= 8 && text.chars().all(|ch| ch.is_ascii_alphanumeric())
}

unsafe fn read_keyboard(lparam: LPARAM) -> Option<(u16, bool, bool)> {
    let handle = lparam as windows_sys::Win32::UI::Input::HRAWINPUT;
    let mut size = std::mem::size_of::<RAWINPUT>() as u32;
    let mut bytes = [0u8; 128];
    let got = GetRawInputData(handle, RID_INPUT, bytes.as_mut_ptr() as *mut _, &mut size, std::mem::size_of::<RAWINPUTHEADER>() as u32);
    if got == u32::MAX || got == 0 {
        return None;
    }
    let input = &*(bytes.as_ptr() as *const RAWINPUT);
    if input.header.dwType != RIM_TYPEKEYBOARD {
        return None;
    }
    let keyboard = input.data.keyboard;
    if keyboard.VKey == 0 || keyboard.VKey == 0xFF {
        return None;
    }
    let down = keyboard.Flags & (RI_KEY_BREAK as u16) == 0;
    let mut injected = input.header.hDevice.is_null();
    let mut source: INPUT_MESSAGE_SOURCE = std::mem::zeroed();
    if GetCurrentInputMessageSource(&mut source) != 0 && (source.originId == IMO_INJECTED || source.originId == IMO_SYSTEM) {
        injected = true;
    }
    Some((keyboard.VKey, down, injected))
}

fn tray_data(hwnd: HWND, icon: HICON, tip: &str) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = 1;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = icon;
    let text = platform::wide(tip);
    let n = text.len().min(127);
    data.szTip[..n].copy_from_slice(&text[..n]);
    data
}
