//! 给小狼毫里的 Lua 调用。只把已提交文本拷进共享内存，然后立刻返回。
//! 不链接另一份 Lua，函数都从已经加载的 rime.dll 里取。

use std::ffi::c_void;
use std::sync::OnceLock;

use typescope_proto::{map_len, Ring, EVENT_NAME, INIT_MUTEX, MAP_NAME};
use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows_sys::Win32::System::Memory::{CreateFileMappingW, MapViewOfFile, FILE_MAP_ALL_ACCESS, PAGE_READWRITE};
use windows_sys::Win32::System::Threading::{CreateEventW, CreateMutexW, ReleaseMutex, SetEvent, WaitForSingleObject};

type LuaTolString = unsafe extern "C" fn(*mut c_void, i32, *mut usize) -> *const u8;
type LuaCreateTable = unsafe extern "C" fn(*mut c_void, i32, i32);
type LuaPushClosure = unsafe extern "C" fn(*mut c_void, unsafe extern "C" fn(*mut c_void) -> i32, i32);
type LuaSetField = unsafe extern "C" fn(*mut c_void, i32, *const u8);

struct Host {
    tolstring: LuaTolString,
    createtable: LuaCreateTable,
    pushcclosure: LuaPushClosure,
    setfield: LuaSetField,
    ring: Ring,
    event: HANDLE,
    _map: HANDLE,
    _view: *mut c_void,
}

unsafe impl Send for Host {}
unsafe impl Sync for Host {}

static HOST: OnceLock<Option<Host>> = OnceLock::new();

/// Rime Lua 模块入口函数。
///
/// # Safety
/// `state` 必须是由已加载 `rime.dll` 传入的有效 `lua_State` 指针。
#[no_mangle]
pub unsafe extern "C" fn luaopen_typescope(state: *mut c_void) -> i32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| open_module(state))).unwrap_or_default()
}

fn open_module(state: *mut c_void) -> i32 {
    let Some(host) = HOST.get_or_init(init_host).as_ref() else {
        return 0;
    };
    unsafe {
        (host.createtable)(state, 0, 1);
        (host.pushcclosure)(state, notify, 0);
        (host.setfield)(state, -2, c"notify".as_ptr().cast());
        host.ring.mark_hello();
    }
    1
}

unsafe extern "C" fn notify(state: *mut c_void) -> i32 {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(host) = HOST.get().and_then(|slot| slot.as_ref()) {
            let mut len = 0usize;
            let ptr = (host.tolstring)(state, 1, &mut len);
            if !ptr.is_null() && len > 0 {
                let bytes = std::slice::from_raw_parts(ptr, len.min(768));
                if host.ring.push(bytes) {
                    SetEvent(host.event);
                }
            }
        }
    }));
    0
}

fn init_host() -> Option<Host> {
    let module = unsafe { GetModuleHandleW(wide("rime.dll").as_ptr()) };
    if module.is_null() {
        return None;
    }
    let tolstring = load::<LuaTolString>(module, b"lua_tolstring\0")?;
    let createtable = load::<LuaCreateTable>(module, b"lua_createtable\0")?;
    let pushcclosure = load::<LuaPushClosure>(module, b"lua_pushcclosure\0")?;
    let setfield = load::<LuaSetField>(module, b"lua_setfield\0")?;
    let mapped = map_shared()?;
    Some(Host { tolstring, createtable, pushcclosure, setfield, ring: mapped.0, event: mapped.1, _map: mapped.2, _view: mapped.3 })
}

fn load<T>(module: windows_sys::Win32::Foundation::HMODULE, name: &[u8]) -> Option<T> {
    let proc = unsafe { GetProcAddress(module, name.as_ptr()) };
    let proc = proc?;
    Some(unsafe { std::mem::transmute_copy(&proc) })
}

fn map_shared() -> Option<(Ring, HANDLE, HANDLE, *mut c_void)> {
    unsafe {
        let map = CreateFileMappingW(INVALID_HANDLE_VALUE, std::ptr::null(), PAGE_READWRITE, 0, map_len() as u32, wide(MAP_NAME).as_ptr());
        if map.is_null() {
            return None;
        }
        let view = MapViewOfFile(map, FILE_MAP_ALL_ACCESS, 0, 0, map_len());
        if view.Value.is_null() {
            CloseHandle(map);
            return None;
        }
        let event = CreateEventW(std::ptr::null(), 0, 0, wide(EVENT_NAME).as_ptr());
        if event.is_null() {
            CloseHandle(map);
            return None;
        }
        let ring = Ring::from_ptr(view.Value as *mut u8, map_len());
        let mutex = CreateMutexW(std::ptr::null(), 0, wide(INIT_MUTEX).as_ptr());
        if !mutex.is_null() {
            WaitForSingleObject(mutex, 3000);
            ring.init_if_needed();
            ReleaseMutex(mutex);
            CloseHandle(mutex);
        } else {
            ring.init_if_needed();
        }
        let _ = GetLastError();
        Some((ring, event, map, view.Value))
    }
}

fn wide(text: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
}
