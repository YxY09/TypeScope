//! Windows 平台的少量辅助函数。热路径不经过这里的注册表和进程查询。

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicPtr, Ordering};

pub use typescope_proto::APP_MUTEX;
use typescope_proto::{map_len, Ring, EVENT_NAME, INIT_MUTEX, MAP_NAME};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, INVALID_HANDLE_VALUE, RECT};
use windows_sys::Win32::System::LibraryLoader::{GetModuleFileNameW, GetModuleHandleW};
use windows_sys::Win32::System::Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE};
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_QUERY_VALUE,
    KEY_SET_VALUE, REG_SZ,
};
use windows_sys::Win32::System::SystemInformation::{GetLocalTime, GetTickCount64};
use windows_sys::Win32::System::Threading::{
    CreateEventW, CreateMutexW, CreateProcessW, OpenMutexW, OpenProcess, QueryFullProcessImageNameW, ReleaseMutex, SetEvent, WaitForSingleObject,
    PROCESS_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, STARTUPINFOW, SYNCHRONIZATION_SYNCHRONIZE,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetForegroundWindow, GetWindowThreadProcessId, IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOWNORMAL,
};

use crate::dates::from_ymd;

pub fn wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
}

pub fn db_path() -> PathBuf {
    // 测试与多实例验证用：设置 TYPESCOPE_DB 可覆盖数据库位置。
    if let Ok(custom) = std::env::var("TYPESCOPE_DB") {
        if !custom.is_empty() {
            let path = PathBuf::from(custom);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            return path;
        }
    }
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned());
    let dir = PathBuf::from(base).join("TypeScope");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("typescope.db")
}

pub fn data_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned());
    let dir = PathBuf::from(base).join("TypeScope");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

pub fn exclude_path() -> PathBuf {
    data_dir().join("exclude.txt")
}

pub fn exe_path() -> PathBuf {
    let mut buf = [0u16; 1024];
    let n = unsafe { GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as u32) };
    if n == 0 {
        return PathBuf::from("typescope-agent.exe");
    }
    PathBuf::from(String::from_utf16_lossy(&buf[..n as usize]))
}

pub fn exe_dir() -> PathBuf {
    exe_path().parent().unwrap_or(Path::new(".")).to_path_buf()
}

pub fn local_stamp() -> (u32, u8, u16) {
    let mut time = unsafe { std::mem::zeroed() };
    unsafe { GetLocalTime(&mut time) };
    let day = from_ymd(time.wYear as i32, time.wMonth as u32, time.wDay as u32);
    let minute = time.wHour * 60 + time.wMinute;
    (day, time.wHour as u8, minute)
}

pub fn tick_ms() -> u64 {
    unsafe { GetTickCount64() }
}

pub fn last_error() -> u32 {
    unsafe { GetLastError() }
}

pub fn is_dark_mode() -> bool {
    unsafe {
        let mut key = std::ptr::null_mut();
        let sub = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");
        let name = wide("AppsUseLightTheme");
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) != 0 {
            return false;
        }
        let mut data = [0u8; 4];
        let mut size = data.len() as u32;
        let mut kind = REG_SZ;
        let rc = RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, data.as_mut_ptr(), &mut size);
        RegCloseKey(key);
        rc == 0 && data[0] == 0
    }
}

pub fn autostart_enabled() -> bool {
    unsafe {
        let mut key = std::ptr::null_mut();
        let sub = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        if RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) != 0 {
            return false;
        }
        let name = wide("TypeScope");
        let mut size = 0u32;
        let rc = RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), std::ptr::null_mut(), std::ptr::null_mut(), &mut size);
        RegCloseKey(key);
        rc == 0 && size > 2
    }
}

pub fn set_autostart(enable: bool) -> bool {
    unsafe {
        let mut key = std::ptr::null_mut();
        let sub = wide("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
        if RegCreateKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, std::ptr::null(), 0, KEY_SET_VALUE, std::ptr::null(), &mut key, std::ptr::null_mut()) != 0 {
            return false;
        }
        let name = wide("TypeScope");
        let rc = if enable {
            let path = exe_dir().join("typescope-agent.exe");
            let value = wide(&format!("\"{}\"", path.display()));
            let bytes = std::slice::from_raw_parts(value.as_ptr() as *const u8, value.len() * 2);
            RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, bytes.as_ptr(), bytes.len() as u32)
        } else {
            RegDeleteValueW(key, name.as_ptr())
        };
        RegCloseKey(key);
        rc == 0 || !enable
    }
}

pub fn agent_running() -> bool {
    let name = wide(typescope_proto::AGENT_MUTEX);
    let handle = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, name.as_ptr()) };
    if handle.is_null() {
        false
    } else {
        unsafe { CloseHandle(handle) };
        true
    }
}

pub fn signal_named(name: &str) {
    let wide_name = wide(name);
    let handle = unsafe { CreateEventW(std::ptr::null(), 0, 0, wide_name.as_ptr()) };
    if !handle.is_null() {
        unsafe {
            SetEvent(handle);
            CloseHandle(handle);
        }
    }
}

pub fn signal_control() {
    signal_named(typescope_proto::CONTROL_EVENT);
}

pub fn focus_existing_dashboard() -> bool {
    unsafe {
        let title = wide("TypeScope 打字统计");
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if hwnd.is_null() {
            return false;
        }
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        } else {
            ShowWindow(hwnd, SW_SHOWNORMAL);
        }
        SetForegroundWindow(hwnd);
        true
    }
}

pub fn open_dashboard() -> bool {
    if focus_existing_dashboard() {
        return true;
    }
    spawn_sibling("typescope-app.exe")
}

pub fn spawn_sibling(file_name: &str) -> bool {
    let path = exe_dir().join(file_name);
    if !path.exists() {
        return false;
    }
    let mut command = wide(&format!("\"{}\"", path.display()));
    let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
    startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
    let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
    let ok = unsafe {
        CreateProcessW(
            std::ptr::null(),
            command.as_mut_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            0,
            0,
            std::ptr::null(),
            std::ptr::null(),
            &startup,
            &mut info,
        )
    };
    if ok != 0 {
        unsafe {
            CloseHandle(info.hProcess);
            CloseHandle(info.hThread);
        }
        true
    } else {
        false
    }
}

pub struct SharedRing {
    map: HANDLE,
    view: windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS,
    pub event: HANDLE,
    pub ring: Ring,
}

impl Drop for SharedRing {
    fn drop(&mut self) {
        unsafe {
            if !self.view.Value.is_null() {
                UnmapViewOfFile(self.view);
                self.view.Value = std::ptr::null_mut();
            }
            if !self.map.is_null() {
                CloseHandle(self.map);
                self.map = std::ptr::null_mut();
            }
            if !self.event.is_null() {
                CloseHandle(self.event);
                self.event = std::ptr::null_mut();
            }
        }
    }
}

pub fn map_ring() -> Option<SharedRing> {
    unsafe {
        let map_name = wide(MAP_NAME);
        let map = CreateFileMappingW(INVALID_HANDLE_VALUE, std::ptr::null(), PAGE_READWRITE, 0, map_len() as u32, map_name.as_ptr());
        if map.is_null() {
            return None;
        }
        let view = MapViewOfFile(map, FILE_MAP_ALL_ACCESS, 0, 0, map_len());
        if view.Value.is_null() {
            CloseHandle(map);
            return None;
        }
        let event_name = wide(EVENT_NAME);
        let event = CreateEventW(std::ptr::null(), 0, 0, event_name.as_ptr());
        let ring = Ring::from_ptr(view.Value as *mut u8, map_len());
        let mutex_name = wide(INIT_MUTEX);
        let mutex = CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr());
        if !mutex.is_null() {
            WaitForSingleObject(mutex, 3000);
            ring.init_if_needed();
            ReleaseMutex(mutex);
            CloseHandle(mutex);
        } else {
            ring.init_if_needed();
        }
        Some(SharedRing { map, view, event, ring })
    }
}

#[derive(Default)]
pub struct ForegroundCache {
    hwnd: isize,
    exe: String,
}

impl ForegroundCache {
    pub fn current(&mut self) -> &str {
        let hwnd = unsafe { GetForegroundWindow() };
        let key = hwnd as isize;
        if key != 0 && key == self.hwnd {
            return &self.exe;
        }
        self.hwnd = key;
        self.exe = exe_of_hwnd(hwnd).unwrap_or_default();
        &self.exe
    }
}

fn exe_of_hwnd(hwnd: windows_sys::Win32::Foundation::HWND) -> Option<String> {
    if hwnd.is_null() {
        return None;
    }
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == 0 {
        return None;
    }
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return None;
    }
    let mut buf = [0u16; 260];
    let mut len = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut len) };
    unsafe { CloseHandle(process) };
    if ok == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&buf[..len as usize]);
    path.rsplit(['\\', '/']).next().map(|name| name.to_string())
}

pub fn read_exclude() -> Vec<String> {
    let text = std::fs::read_to_string(exclude_path()).unwrap_or_default();
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_ascii_lowercase())
        .collect()
}

pub fn ensure_exclude_file() {
    let path = exclude_path();
    if path.exists() {
        return;
    }
    let _ = std::fs::write(
        path,
        "# 每行写一个程序文件名，例如 KeePass.exe\r\n# 这个程序在前台时，TypeScope 不计按键，也不计输入法提交。\r\n# 这不能识别密码框。如果把浏览器写进来，浏览时的打字也会被排除。\r\n",
    );
}

pub fn write_report(name: &str, text: &str) {
    let path = data_dir().join(name);
    let _ = std::fs::write(path, text);
}

pub struct InstanceLock {
    handle: AtomicPtr<std::ffi::c_void>,
}

impl InstanceLock {
    pub fn acquire(name: &str) -> Option<Self> {
        let wide_name = wide(name);
        let handle = unsafe { CreateMutexW(std::ptr::null(), 1, wide_name.as_ptr()) };
        if handle.is_null() {
            return None;
        }
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            unsafe { CloseHandle(handle) };
            return None;
        }
        Some(Self { handle: AtomicPtr::new(handle) })
    }
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let handle = self.handle.swap(std::ptr::null_mut(), Ordering::Relaxed);
        if !handle.is_null() {
            unsafe {
                ReleaseMutex(handle);
                CloseHandle(handle);
            }
        }
    }
}

#[allow(dead_code)]
pub fn empty_rect() -> RECT {
    RECT { left: 0, top: 0, right: 0, bottom: 0 }
}

pub fn open_path(path: &Path) {
    let file = wide(&path.display().to_string());
    let op = wide("open");
    unsafe {
        windows_sys::Win32::UI::Shell::ShellExecuteW(std::ptr::null_mut(), op.as_ptr(), file.as_ptr(), std::ptr::null(), std::ptr::null(), SW_SHOWNORMAL);
    }
}

pub fn module_handle() -> windows_sys::Win32::Foundation::HMODULE {
    unsafe { GetModuleHandleW(std::ptr::null()) }
}

const RIME_COMMIT_LUA: &str = include_str!("../../../rime/typescope_commit.lua");

fn reg_query_sz(root: HKEY, subkey: &str, value_name: &str) -> Option<String> {
    unsafe {
        let mut key = std::ptr::null_mut();
        let sub = wide(subkey);
        if RegOpenKeyExW(root, sub.as_ptr(), 0, KEY_QUERY_VALUE, &mut key) != 0 {
            return None;
        }
        let name = wide(value_name);
        let mut buf = [0u16; 520];
        let mut size = (buf.len() * 2) as u32;
        let mut kind = REG_SZ;
        let rc = RegQueryValueExW(key, name.as_ptr(), std::ptr::null(), &mut kind, buf.as_mut_ptr() as *mut u8, &mut size);
        RegCloseKey(key);
        if rc != 0 || size < 2 {
            return None;
        }
        let len = (size as usize / 2).saturating_sub(1);
        let s = String::from_utf16_lossy(&buf[..len]).trim_end_matches('\0').trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    }
}

pub fn rime_user_dir() -> Option<PathBuf> {
    if let Some(dir) = reg_query_sz(HKEY_CURRENT_USER, "Software\\Rime\\Weasel", "RimeUserDir") {
        let p = PathBuf::from(dir);
        if p.exists() {
            return Some(p);
        }
    }
    let appdata = std::env::var_os("APPDATA")?;
    let fallback = PathBuf::from(appdata).join("Rime");
    if fallback.exists() {
        Some(fallback)
    } else {
        None
    }
}

pub fn find_weasel_deployer() -> Option<PathBuf> {
    for (root, sub) in [
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\WOW6432Node\\Rime\\Weasel"),
        (HKEY_LOCAL_MACHINE, "SOFTWARE\\Rime\\Weasel"),
        (HKEY_CURRENT_USER, "Software\\Rime\\Weasel"),
    ] {
        if let Some(weasel_root) = reg_query_sz(root, sub, "WeaselRoot") {
            let exe = PathBuf::from(weasel_root).join("WeaselDeployer.exe");
            if exe.exists() {
                return Some(exe);
            }
        }
    }
    for env_key in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(pf) = std::env::var_os(env_key) {
            let rime_dir = PathBuf::from(pf).join("Rime");
            if let Ok(entries) = std::fs::read_dir(rime_dir) {
                let mut candidates: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path().join("WeaselDeployer.exe"))
                    .filter(|p| p.exists())
                    .collect();
                candidates.sort();
                if let Some(last) = candidates.pop() {
                    return Some(last);
                }
            }
        }
    }
    None
}

pub fn install_rime() -> Result<String, String> {
    let user_dir = rime_user_dir().ok_or_else(|| "未检测到小狼毫用户目录（%APPDATA%\\Rime），请先安装小狼毫输入法".to_string())?;
    let cur_dir = exe_dir();
    let dll_path = [
        cur_dir.join("typescope_rime.dll"),
        cur_dir.join("..\\target\\release\\typescope_rime.dll"),
        cur_dir.join("target\\release\\typescope_rime.dll"),
    ]
    .into_iter()
    .find(|p| p.exists())
    .ok_or_else(|| format!("未在程序目录（{}）下找到 typescope_rime.dll", cur_dir.display()))?;

    let lua_dir = user_dir.join("lua");
    let backup_dir = user_dir.join("typescope-backup");
    std::fs::create_dir_all(&lua_dir).map_err(|e| format!("创建 lua 目录失败: {e}"))?;
    std::fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {e}"))?;

    std::fs::copy(&dll_path, lua_dir.join("typescope_rime.dll")).map_err(|e| format!("复制 typescope_rime.dll 失败: {e}"))?;
    std::fs::write(lua_dir.join("typescope_commit.lua"), RIME_COMMIT_LUA).map_err(|e| format!("写入 typescope_commit.lua 失败: {e}"))?;

    let global_configs = ["default", "weasel", "squirrel", "ibus_rime", "fcitx_rime"];
    let mut schemas: Vec<String> = Vec::new();
    let push_schema = |list: &mut Vec<String>, s: &str| {
        let clean = s.trim();
        if !clean.is_empty()
            && !global_configs.iter().any(|g| g.eq_ignore_ascii_case(clean))
            && !list.iter().any(|item| item.eq_ignore_ascii_case(clean))
        {
            list.push(clean.to_string());
        }
    };

    if let Ok(user_yaml) = std::fs::read_to_string(user_dir.join("user.yaml")) {
        for line in user_yaml.lines() {
            if let Some(rest) = line.trim().strip_prefix("previously_selected_schema:") {
                push_schema(&mut schemas, rest.trim());
            }
        }
    }

    if let Ok(entries) = std::fs::read_dir(&user_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(base) = name.strip_suffix(".custom.yaml") {
                push_schema(&mut schemas, base);
            }
        }
    }

    if schemas.is_empty() {
        if let Ok(entries) = std::fs::read_dir(&user_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(base) = name.strip_suffix(".schema.yaml") {
                    push_schema(&mut schemas, base);
                }
            }
        }
    }

    if schemas.is_empty() {
        return Err("未在小狼毫目录中识别到输入方案（*.schema.yaml / user.yaml）".to_string());
    }

    let (day, hour, minute) = local_stamp();
    let hook_line = "  \"engine/processors/@before 0\": lua_processor@*typescope_commit";
    for schema in &schemas {
        let custom_path = user_dir.join(format!("{schema}.custom.yaml"));
        if custom_path.exists() {
            let content = std::fs::read_to_string(&custom_path).unwrap_or_default();
            if !content.contains("typescope_commit") {
                let backup_path = backup_dir.join(format!("{schema}.custom.yaml.{day}-{hour:02}{minute:02}"));
                let _ = std::fs::copy(&custom_path, backup_path);
                let has_patch = content.lines().any(|l| l.trim().starts_with("patch:"));
                let mut updated = content;
                if !updated.ends_with('\n') && !updated.is_empty() {
                    updated.push_str("\r\n");
                }
                if !has_patch {
                    updated.push_str("patch:\r\n");
                }
                updated.push_str(hook_line);
                updated.push_str("\r\n");
                std::fs::write(&custom_path, updated).map_err(|e| format!("更新 {schema}.custom.yaml 失败: {e}"))?;
            }
        } else {
            let content = format!("patch:\r\n{hook_line}\r\n");
            std::fs::write(&custom_path, content).map_err(|e| format!("创建 {schema}.custom.yaml 失败: {e}"))?;
        }
    }

    let deployed = if let Some(deployer) = find_weasel_deployer() {
        let mut cmd = wide(&format!("\"{}\" /deploy", deployer.display()));
        let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
        startup.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut info: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let ok = unsafe {
            CreateProcessW(
                std::ptr::null(),
                cmd.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
                std::ptr::null(),
                std::ptr::null(),
                &startup,
                &mut info,
            )
        };
        if ok != 0 {
            unsafe {
                CloseHandle(info.hProcess);
                CloseHandle(info.hThread);
            }
            true
        } else {
            false
        }
    } else {
        false
    };

    let joined = schemas.join(", ");
    if deployed {
        Ok(format!("已自动配置方案 [{joined}] 并触发小狼毫重新部署"))
    } else {
        Ok(format!("已配置方案 [{joined}]，请右键任务栏小狼毫图标点击「重新部署」"))
    }
}
