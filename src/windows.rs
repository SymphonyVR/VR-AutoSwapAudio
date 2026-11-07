#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_void;
use core::fmt::Write;
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicPtr, Ordering};

use crate::steamvr::{SteamVrPaths, VrAudioProfile};
use crate::str_buf::{
    FixedBuf, contains_ignore_ascii_case, eq_ignore_ascii_case, parse_windows_cmdline_utf16,
    utf8_to_utf16_null, utf16_to_utf8,
};

pub const PATH_SEP: char = '\\';

type HRESULT = i32;
type HANDLE = *mut c_void;
type DWORD = u32;
type BOOL = i32;

const S_OK: HRESULT = 0;
const E_NOINTERFACE: HRESULT = -2147467262i32; // 0x80004002
const E_POINTER: HRESULT = -2147467261i32; // 0x80004003

const COINIT_MULTITHREADED: DWORD = 0x0;
const CLSCTX_ALL: DWORD = 23;
const STGM_READ: DWORD = 0;

const E_RENDER: DWORD = 0;
const E_CAPTURE: DWORD = 1;

const E_CONSOLE: DWORD = 0;
const E_MULTIMEDIA: DWORD = 1;
const E_COMMUNICATIONS: DWORD = 2;

const DEVICE_STATE_ACTIVE: DWORD = 0x00000001;

const STD_OUTPUT_HANDLE: DWORD = -11i32 as DWORD;
const GENERIC_READ: DWORD = 0x80000000;
const GENERIC_WRITE: DWORD = 0x40000000;
const FILE_SHARE_READ: DWORD = 0x00000001;
const FILE_SHARE_WRITE: DWORD = 0x00000002;
const OPEN_EXISTING: DWORD = 3;
const CREATE_ALWAYS: DWORD = 2;
const FILE_ATTRIBUTE_NORMAL: DWORD = 0x80;
const FILE_END: DWORD = 2;
const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
const ERROR_FILE_NOT_FOUND: DWORD = 2;
const ERROR_PATH_NOT_FOUND: DWORD = 3;

const MOVEFILE_REPLACE_EXISTING: DWORD = 0x00000001;
const MOVEFILE_WRITE_THROUGH: DWORD = 0x00000008;

const MEM_COMMIT: DWORD = 0x00001000;
const MEM_RESERVE: DWORD = 0x00002000;
const MEM_RELEASE: DWORD = 0x00008000;
const PAGE_READWRITE: DWORD = 0x04;

const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
const SYNCHRONIZE: DWORD = 0x00100000;
const INFINITE: DWORD = 0xFFFFFFFF;
const WAIT_OBJECT_0: DWORD = 0x00000000;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GUID {
    pub Data1: u32,
    pub Data2: u16,
    pub Data3: u16,
    pub Data4: [u8; 8],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PROPERTYKEY {
    pub fmtid: GUID,
    pub pid: u32,
}

#[repr(C)]
pub struct PROPVARIANT {
    pub vt: u16,
    pub wReserved1: u16,
    pub wReserved2: u16,
    pub wReserved3: u16,
    pub pwszVal: *mut u16,
    pub _pad: u64,
}

#[repr(C)]
pub struct PROCESSENTRY32W {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ProcessID: DWORD,
    pub th32DefaultHeapID: usize,
    pub th32ModuleID: DWORD,
    pub cntThreads: DWORD,
    pub th32ParentProcessID: DWORD,
    pub pcPriClassBase: i32,
    pub dwFlags: DWORD,
    pub szExeFile: [u16; 260],
}

const IID_IUnknown: GUID = GUID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

const CLSID_MMDeviceEnumerator: GUID = GUID {
    Data1: 0xBCDE0395,
    Data2: 0xE52F,
    Data3: 0x467C,
    Data4: [0x8E, 0x3D, 0xC4, 0x57, 0x92, 0x91, 0x69, 0x2E],
};

const IID_IMMDeviceEnumerator: GUID = GUID {
    Data1: 0xA95664D2,
    Data2: 0x9614,
    Data3: 0x4F35,
    Data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
};

const IID_IMMNotificationClient: GUID = GUID {
    Data1: 0x7991EEC9,
    Data2: 0x7E89,
    Data3: 0x4D85,
    Data4: [0x83, 0x90, 0x6C, 0x70, 0x3C, 0xEC, 0x60, 0xC0],
};

const PKEY_Device_FriendlyName: PROPERTYKEY = PROPERTYKEY {
    fmtid: GUID {
        Data1: 0xA45C254E,
        Data2: 0xDF1C,
        Data3: 0x4EFD,
        Data4: [0x80, 0x20, 0x67, 0xD1, 0x46, 0xA8, 0x50, 0xE0],
    },
    pid: 14,
};

const CLSID_PolicyConfigClient: GUID = GUID {
    Data1: 0x870AF99C,
    Data2: 0x171D,
    Data3: 0x4F9E,
    Data4: [0xAF, 0x0D, 0xE6, 0x3D, 0xF4, 0x0C, 0x2B, 0xC9],
};

const IID_IPolicyConfig_Win10: GUID = GUID {
    Data1: 0xF8679F50,
    Data2: 0x850A,
    Data3: 0x41CF,
    Data4: [0x9C, 0x72, 0x43, 0x0F, 0x29, 0x02, 0x90, 0xC8],
};

const IID_IPolicyConfig_RS: GUID = GUID {
    Data1: 0x8F9FB2AA,
    Data2: 0x1C0B,
    Data3: 0x4D54,
    Data4: [0xB6, 0xBB, 0xB2, 0xF2, 0xA1, 0x0C, 0xE0, 0x3C],
};

const IID_IPolicyConfigVista: GUID = GUID {
    Data1: 0x568B9108,
    Data2: 0x44BF,
    Data3: 0x40B4,
    Data4: [0x90, 0x06, 0x86, 0xAF, 0xE5, 0xB5, 0xA6, 0x20],
};

unsafe extern "system" {
    pub fn ExitProcess(uExitCode: u32) -> !;
    fn GetLastError() -> DWORD;
    fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    fn WriteFile(
        hFile: HANDLE,
        lpBuffer: *const u8,
        nNumberOfBytesToWrite: DWORD,
        lpNumberOfBytesWritten: *mut DWORD,
        lpOverlapped: *mut c_void,
    ) -> BOOL;
    fn ReadFile(
        hFile: HANDLE,
        lpBuffer: *mut u8,
        nNumberOfBytesToRead: DWORD,
        lpNumberOfBytesRead: *mut DWORD,
        lpOverlapped: *mut c_void,
    ) -> BOOL;
    fn FlushFileBuffers(hFile: HANDLE) -> BOOL;
    fn GetFileSizeEx(hFile: HANDLE, lpFileSize: *mut i64) -> BOOL;
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn MoveFileExW(
        lpExistingFileName: *const u16,
        lpNewFileName: *const u16,
        dwFlags: DWORD,
    ) -> BOOL;
    fn DeleteFileW(lpFileName: *const u16) -> BOOL;
    fn SetFilePointer(
        hFile: HANDLE,
        lDistanceToMove: i32,
        lpDistanceToMoveHigh: *mut i32,
        dwMoveMethod: DWORD,
    ) -> DWORD;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn GetEnvironmentVariableW(lpName: *const u16, lpBuffer: *mut u16, nSize: DWORD) -> DWORD;
    fn GetModuleFileNameW(hModule: HANDLE, lpFilename: *mut u16, nSize: DWORD) -> DWORD;
    fn GetCommandLineW() -> *const u16;

    fn VirtualAlloc(
        lpAddress: *mut c_void,
        dwSize: usize,
        flAllocationType: DWORD,
        flProtect: DWORD,
    ) -> *mut u8;
    fn VirtualFree(lpAddress: *mut c_void, dwSize: usize, dwFreeType: DWORD) -> BOOL;

    fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> HANDLE;
    fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> BOOL;
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;

    fn CreateEventW(
        lpEventAttributes: *mut c_void,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: *const u16,
    ) -> HANDLE;
    fn SetEvent(hEvent: HANDLE) -> BOOL;
    fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    fn WaitForMultipleObjects(
        nCount: DWORD,
        lpHandles: *const HANDLE,
        bWaitAll: BOOL,
        dwMilliseconds: DWORD,
    ) -> DWORD;

    fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: DWORD) -> HRESULT;
    fn CoCreateInstance(
        rclsid: *const GUID,
        pUnkOuter: *mut c_void,
        dwClsContext: DWORD,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    fn CoTaskMemFree(pv: *mut c_void);
    fn CoFreeUnusedLibrariesEx(dwUnloadDelay: DWORD, dwReserved: DWORD);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let d = dest as *mut u8;
    let s = src as *const u8;
    let mut i = 0;
    unsafe {
        while i < n {
            *d.add(i) = *s.add(i);
            i += 1;
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void {
    let d = dest as *mut u8;
    let s = src as *const u8;
    unsafe {
        if (d as usize) < (s as usize) {
            let mut i = 0;
            while i < n {
                *d.add(i) = *s.add(i);
                i += 1;
            }
        } else if (d as usize) > (s as usize) {
            let mut i = n;
            while i > 0 {
                i -= 1;
                *d.add(i) = *s.add(i);
            }
        }
    }
    dest
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void {
    let p = s as *mut u8;
    let b = c as u8;
    let mut i = 0;
    unsafe {
        while i < n {
            *p.add(i) = b;
            i += 1;
        }
    }
    s
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32 {
    let p1 = s1 as *const u8;
    let p2 = s2 as *const u8;
    let mut i = 0;
    unsafe {
        while i < n {
            let a = *p1.add(i);
            let b = *p2.add(i);
            if a != b {
                return (a as i32) - (b as i32);
            }
            i += 1;
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlen(s: *const i8) -> usize {
    let p = s as *const u8;
    let mut i = 0;
    unsafe {
        while *p.add(i) != 0 {
            i += 1;
        }
    }
    i
}

#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".global __chkstk",
    "__chkstk:",
    "sub rsp, 16",
    "mov [rsp], r10",
    "mov [rsp + 8], r11",
    "xor r11, r11",
    "lea r10, [rsp + 24]",
    "sub r10, rax",
    "cmovb r10, r11",
    "mov r11, gs:[0x10]",
    "cmp r10, r11",
    "jae 3f",
    "and r10w, 0xF000",
    "2:",
    "lea r11, [r11 - 0x1000]",
    "test byte ptr [r11], 0",
    "cmp r10, r11",
    "jne 2b",
    "3:",
    "mov r10, [rsp]",
    "mov r11, [rsp + 8]",
    "add rsp, 16",
    "ret"
);

/// Allocates temporary scratch pages directly from the OS via `VirtualAlloc` and immediately
/// releases them back to the kernel (`MEM_RELEASE`) on return, avoiding permanently committed stack pages.
pub fn with_scratch_buffer<F: FnOnce(&mut [u8])>(size: usize, f: F) {
    if size == 0 {
        return;
    }
    unsafe {
        let ptr = VirtualAlloc(null_mut(), size, MEM_COMMIT | MEM_RESERVE, PAGE_READWRITE);
        if ptr.is_null() {
            return;
        }
        let slice = core::slice::from_raw_parts_mut(ptr, size);
        f(slice);
        let _ = VirtualFree(ptr as *mut c_void, 0, MEM_RELEASE);
    }
}

// Raw COM Vtables
#[repr(C)]
struct IMMDeviceEnumeratorVtbl {
    QueryInterface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
    EnumAudioEndpoints:
        unsafe extern "system" fn(*mut c_void, DWORD, DWORD, *mut *mut IMMDeviceCollection) -> HRESULT,
    GetDefaultAudioEndpoint:
        unsafe extern "system" fn(*mut c_void, DWORD, DWORD, *mut *mut IMMDevice) -> HRESULT,
    GetDevice: unsafe extern "system" fn(*mut c_void, *const u16, *mut *mut IMMDevice) -> HRESULT,
    RegisterEndpointNotificationCallback:
        unsafe extern "system" fn(*mut c_void, *mut IMMNotificationClient) -> HRESULT,
    UnregisterEndpointNotificationCallback:
        unsafe extern "system" fn(*mut c_void, *mut IMMNotificationClient) -> HRESULT,
}

#[repr(C)]
struct IMMDeviceEnumerator {
    lpVtbl: *const IMMDeviceEnumeratorVtbl,
}

#[repr(C)]
struct IMMDeviceCollectionVtbl {
    QueryInterface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
    GetCount: unsafe extern "system" fn(*mut c_void, *mut u32) -> HRESULT,
    Item: unsafe extern "system" fn(*mut c_void, u32, *mut *mut IMMDevice) -> HRESULT,
}

#[repr(C)]
struct IMMDeviceCollection {
    lpVtbl: *const IMMDeviceCollectionVtbl,
}

#[repr(C)]
struct IMMDeviceVtbl {
    QueryInterface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
    Activate: unsafe extern "system" fn(
        *mut c_void,
        *const GUID,
        DWORD,
        *const c_void,
        *mut *mut c_void,
    ) -> HRESULT,
    OpenPropertyStore:
        unsafe extern "system" fn(*mut c_void, DWORD, *mut *mut IPropertyStore) -> HRESULT,
    GetId: unsafe extern "system" fn(*mut c_void, *mut *mut u16) -> HRESULT,
    GetState: unsafe extern "system" fn(*mut c_void, *mut DWORD) -> HRESULT,
}

#[repr(C)]
struct IMMDevice {
    lpVtbl: *const IMMDeviceVtbl,
}

#[repr(C)]
struct IPropertyStoreVtbl {
    QueryInterface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
    GetCount: unsafe extern "system" fn(*mut c_void, *mut DWORD) -> HRESULT,
    GetAt: unsafe extern "system" fn(*mut c_void, DWORD, *mut PROPERTYKEY) -> HRESULT,
    GetValue: unsafe extern "system" fn(*mut c_void, *const PROPERTYKEY, *mut PROPVARIANT) -> HRESULT,
    SetValue: unsafe extern "system" fn(*mut c_void, *const PROPERTYKEY, *const PROPVARIANT) -> HRESULT,
    Commit: unsafe extern "system" fn(*mut c_void) -> HRESULT,
}

#[repr(C)]
struct IPropertyStore {
    lpVtbl: *const IPropertyStoreVtbl,
}

#[repr(C)]
struct IPolicyConfigVtbl {
    QueryInterface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut c_void) -> u32,
    Release: unsafe extern "system" fn(*mut c_void) -> u32,
    GetMixFormat: *const c_void,
    GetDeviceFormat: *const c_void,
    ResetDeviceFormat: *const c_void,
    SetDeviceFormat: *const c_void,
    GetProcessingPeriod: *const c_void,
    SetProcessingPeriod: *const c_void,
    GetShareMode: *const c_void,
    SetShareMode: *const c_void,
    GetPropertyValue: *const c_void,
    SetPropertyValue: *const c_void,
    SetDefaultEndpoint: unsafe extern "system" fn(*mut c_void, *const u16, DWORD) -> HRESULT,
    SetEndpointVisibility: *const c_void,
}

#[repr(C)]
struct IPolicyConfig {
    lpVtbl: *const IPolicyConfigVtbl,
}

// Static COM IMMNotificationClient for 100% event-driven audio notifications
#[repr(C)]
struct IMMNotificationClientVtbl {
    QueryInterface: unsafe extern "system" fn(*mut IMMNotificationClient, *const GUID, *mut *mut c_void) -> HRESULT,
    AddRef: unsafe extern "system" fn(*mut IMMNotificationClient) -> u32,
    Release: unsafe extern "system" fn(*mut IMMNotificationClient) -> u32,
    OnDeviceStateChanged: unsafe extern "system" fn(*mut IMMNotificationClient, *const u16, DWORD) -> HRESULT,
    OnDeviceAdded: unsafe extern "system" fn(*mut IMMNotificationClient, *const u16) -> HRESULT,
    OnDeviceRemoved: unsafe extern "system" fn(*mut IMMNotificationClient, *const u16) -> HRESULT,
    OnDefaultDeviceChanged: unsafe extern "system" fn(*mut IMMNotificationClient, DWORD, DWORD, *const u16) -> HRESULT,
    OnPropertyValueChanged: unsafe extern "system" fn(*mut IMMNotificationClient, *const u16, PROPERTYKEY) -> HRESULT,
}

#[repr(C)]
struct IMMNotificationClient {
    lpVtbl: *const IMMNotificationClientVtbl,
}

static G_AUDIO_EVENT: AtomicPtr<c_void> = AtomicPtr::new(null_mut());

unsafe extern "system" fn notif_query_interface(
    this: *mut IMMNotificationClient,
    riid: *const GUID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    let iid = unsafe { *riid };
    if iid == IID_IUnknown || iid == IID_IMMNotificationClient {
        unsafe { *ppv = this as *mut c_void };
        S_OK
    } else {
        unsafe { *ppv = null_mut() };
        E_NOINTERFACE
    }
}

unsafe extern "system" fn notif_add_ref(_this: *mut IMMNotificationClient) -> u32 {
    1
}

unsafe extern "system" fn notif_release(_this: *mut IMMNotificationClient) -> u32 {
    1
}

unsafe extern "system" fn notif_on_device_state_changed(
    _this: *mut IMMNotificationClient,
    _pwstrDeviceId: *const u16,
    _dwNewState: DWORD,
) -> HRESULT {
    let ev = G_AUDIO_EVENT.load(Ordering::Relaxed);
    if !ev.is_null() {
        unsafe { SetEvent(ev) };
    }
    S_OK
}

unsafe extern "system" fn notif_on_device_added(
    _this: *mut IMMNotificationClient,
    _pwstrDeviceId: *const u16,
) -> HRESULT {
    let ev = G_AUDIO_EVENT.load(Ordering::Relaxed);
    if !ev.is_null() {
        unsafe { SetEvent(ev) };
    }
    S_OK
}

unsafe extern "system" fn notif_on_device_removed(
    _this: *mut IMMNotificationClient,
    _pwstrDeviceId: *const u16,
) -> HRESULT {
    S_OK
}

unsafe extern "system" fn notif_on_default_device_changed(
    _this: *mut IMMNotificationClient,
    _flow: DWORD,
    role: DWORD,
    _pwstrDefaultDeviceId: *const u16,
) -> HRESULT {
    if role == E_CONSOLE || role == E_MULTIMEDIA {
        let ev = G_AUDIO_EVENT.load(Ordering::Relaxed);
        if !ev.is_null() {
            unsafe { SetEvent(ev) };
        }
    }
    S_OK
}

unsafe extern "system" fn notif_on_property_value_changed(
    _this: *mut IMMNotificationClient,
    _pwstrDeviceId: *const u16,
    _key: PROPERTYKEY,
) -> HRESULT {
    S_OK
}

static NOTIFICATION_CLIENT_VTBL: IMMNotificationClientVtbl = IMMNotificationClientVtbl {
    QueryInterface: notif_query_interface,
    AddRef: notif_add_ref,
    Release: notif_release,
    OnDeviceStateChanged: notif_on_device_state_changed,
    OnDeviceAdded: notif_on_device_added,
    OnDeviceRemoved: notif_on_device_removed,
    OnDefaultDeviceChanged: notif_on_default_device_changed,
    OnPropertyValueChanged: notif_on_property_value_changed,
};

static mut NOTIFICATION_CLIENT: IMMNotificationClient = IMMNotificationClient {
    lpVtbl: &NOTIFICATION_CLIENT_VTBL,
};

pub fn print(s: &str) {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return;
    }
    unsafe {
        let h = GetStdHandle(STD_OUTPUT_HANDLE);
        if !h.is_null() && h != INVALID_HANDLE_VALUE {
            let mut written = 0u32;
            let _ = WriteFile(h, bytes.as_ptr(), bytes.len() as DWORD, &mut written, null_mut());
        }
    }
}

pub fn println(s: &str) {
    print(s);
    print("\r\n");
}

pub fn exit(code: u32) -> ! {
    unsafe { ExitProcess(code) }
}

pub fn get_openvrpaths_file(out: &mut FixedBuf<512>) -> bool {
    let name: [u16; 13] = [
        b'L' as u16, b'O' as u16, b'C' as u16, b'A' as u16, b'L' as u16, b'A' as u16,
        b'P' as u16, b'P' as u16, b'D' as u16, b'A' as u16, b'T' as u16, b'A' as u16, 0,
    ];
    let mut wbuf = [0u16; 260];
    let len = unsafe { GetEnvironmentVariableW(name.as_ptr(), wbuf.as_mut_ptr(), 260) } as usize;
    if len == 0 || len >= 260 {
        return false;
    }
    out.len = utf16_to_utf8(&wbuf[..len], &mut out.buf);
    out.push_str("\\openvr\\openvrpaths.vrpath");
    true
}

pub fn get_exe_dir(out: &mut FixedBuf<512>) -> bool {
    let mut wbuf = [0u16; 260];
    let len = unsafe { GetModuleFileNameW(null_mut(), wbuf.as_mut_ptr(), 260) } as usize;
    if len == 0 || len >= 260 {
        return false;
    }
    let mut full = FixedBuf::<512>::new();
    full.len = utf16_to_utf8(&wbuf[..len], &mut full.buf);
    let bytes = full.as_bytes();
    let mut last_sep = None;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' || bytes[i] == b'/' {
            last_sep = Some(i);
        }
        i += 1;
    }
    if let Some(idx) = last_sep {
        out.set_bytes(&bytes[..idx]);
        true
    } else {
        false
    }
}

pub fn read_file_bytes(path: &[u8], out: &mut [u8]) -> usize {
    let mut wpath = [0u16; 520];
    utf8_to_utf16_null(path, &mut wpath);
    unsafe {
        let h = CreateFileW(
            wpath.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        );
        if h == INVALID_HANDLE_VALUE || h.is_null() {
            return 0;
        }
        let mut read_bytes = 0u32;
        let ok = ReadFile(h, out.as_mut_ptr(), out.len() as DWORD, &mut read_bytes, null_mut());
        CloseHandle(h);
        if ok != 0 {
            read_bytes as usize
        } else {
            0
        }
    }
}

/// Reads an entire file into `out`, returning:
/// - `Some(0)` if the file does not exist yet
/// - `Some(len)` if the entire file fit into `out` and was read completely
/// - `None` if the file exceeds `out.len()` or an I/O error occurred (preventing truncation)
pub fn read_file_exact_checked(path: &[u8], out: &mut [u8]) -> Option<usize> {
    let mut wpath = [0u16; 520];
    utf8_to_utf16_null(path, &mut wpath);
    unsafe {
        let h = CreateFileW(
            wpath.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        );
        if h == INVALID_HANDLE_VALUE || h.is_null() {
            let err = GetLastError();
            if err == ERROR_FILE_NOT_FOUND || err == ERROR_PATH_NOT_FOUND {
                return Some(0);
            }
            return None;
        }
        let mut file_size = 0i64;
        if GetFileSizeEx(h, &mut file_size) == 0 || file_size < 0 {
            CloseHandle(h);
            return None;
        }
        if (file_size as usize) > out.len() {
            CloseHandle(h);
            return None;
        }
        let expected = file_size as DWORD;
        let mut read_bytes = 0u32;
        let ok = ReadFile(h, out.as_mut_ptr(), expected, &mut read_bytes, null_mut());
        CloseHandle(h);
        if ok != 0 && read_bytes == expected {
            Some(read_bytes as usize)
        } else {
            None
        }
    }
}

pub fn read_file_tail_bytes(path: &[u8], out: &mut [u8]) -> usize {
    let mut wpath = [0u16; 520];
    utf8_to_utf16_null(path, &mut wpath);
    unsafe {
        let h = CreateFileW(
            wpath.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        );
        if h == INVALID_HANDLE_VALUE || h.is_null() {
            return 0;
        }
        let max_len = out.len() as i32;
        let _ = SetFilePointer(h, -max_len, null_mut(), FILE_END);
        let mut read_bytes = 0u32;
        let ok = ReadFile(h, out.as_mut_ptr(), out.len() as DWORD, &mut read_bytes, null_mut());
        CloseHandle(h);
        if ok != 0 {
            read_bytes as usize
        } else {
            0
        }
    }
}

/// Writes `data` to `<path>.tmp`, flushes buffers to disk, and atomically replaces `<path>` via `MoveFileExW`.
pub fn write_file_atomic(path: &[u8], data: &[u8]) -> bool {
    let mut tmp_path = FixedBuf::<520>::new();
    tmp_path.push_bytes(path);
    tmp_path.push_str(".tmp");

    let mut wpath = [0u16; 520];
    let mut wtmp = [0u16; 520];
    utf8_to_utf16_null(path, &mut wpath);
    utf8_to_utf16_null(tmp_path.as_bytes(), &mut wtmp);

    unsafe {
        let h = CreateFileW(
            wtmp.as_ptr(),
            GENERIC_WRITE,
            0,
            null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            null_mut(),
        );
        if h == INVALID_HANDLE_VALUE || h.is_null() {
            return false;
        }
        let mut written = 0u32;
        let ok = WriteFile(h, data.as_ptr(), data.len() as DWORD, &mut written, null_mut());
        let _ = FlushFileBuffers(h);
        CloseHandle(h);

        if ok == 0 || (written as usize) != data.len() {
            let _ = DeleteFileW(wtmp.as_ptr());
            return false;
        }

        if MoveFileExW(
            wtmp.as_ptr(),
            wpath.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        ) != 0
        {
            true
        } else {
            let _ = DeleteFileW(wtmp.as_ptr());
            false
        }
    }
}

/// Parses command-line arguments directly from `GetCommandLineW()` without loading `Shell32.dll`.
pub fn parse_cli_args(out_args: &mut [FixedBuf<128>; 8]) -> usize {
    unsafe {
        let cmd = GetCommandLineW();
        if cmd.is_null() {
            return 0;
        }
        let mut len = 0usize;
        while *cmd.add(len) != 0 && len < 4096 {
            len += 1;
        }
        let slice = core::slice::from_raw_parts(cmd, len);
        parse_windows_cmdline_utf16(slice, out_args)
    }
}

pub fn find_process_pids() -> (DWORD, DWORD) {
    let mut vrserver_pid: DWORD = 0;
    let mut vrmonitor_pid: DWORD = 0;
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE || snap.is_null() {
            return (0, 0);
        }
        let mut pe: PROCESSENTRY32W = core::mem::zeroed();
        pe.dwSize = core::mem::size_of::<PROCESSENTRY32W>() as DWORD;
        if Process32FirstW(snap, &mut pe) != 0 {
            loop {
                let mut name_buf = [0u8; 128];
                let n = utf16_to_utf8(&pe.szExeFile, &mut name_buf);
                let exe = &name_buf[..n];
                if eq_ignore_ascii_case(exe, b"vrserver.exe") {
                    vrserver_pid = pe.th32ProcessID;
                } else if eq_ignore_ascii_case(exe, b"vrmonitor.exe") {
                    vrmonitor_pid = pe.th32ProcessID;
                }
                if Process32NextW(snap, &mut pe) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
    }
    (vrserver_pid, vrmonitor_pid)
}

unsafe fn get_device_id_and_name(
    dev: *mut IMMDevice,
    id_out: &mut FixedBuf<160>,
    name_out: &mut FixedBuf<160>,
) -> bool {
    id_out.clear();
    name_out.clear();
    if dev.is_null() {
        return false;
    }
    unsafe {
        let vtbl = &*(*dev).lpVtbl;
        let mut pwstr_id: *mut u16 = null_mut();
        if (vtbl.GetId)(dev as *mut c_void, &mut pwstr_id) == S_OK && !pwstr_id.is_null() {
            let mut wlen = 0usize;
            while *pwstr_id.add(wlen) != 0 && wlen < 256 {
                wlen += 1;
            }
            let slice = core::slice::from_raw_parts(pwstr_id, wlen);
            id_out.len = utf16_to_utf8(slice, &mut id_out.buf);
            CoTaskMemFree(pwstr_id as *mut c_void);
        }

        let mut props: *mut IPropertyStore = null_mut();
        if (vtbl.OpenPropertyStore)(dev as *mut c_void, STGM_READ, &mut props) == S_OK
            && !props.is_null()
        {
            let pvtbl = &*(*props).lpVtbl;
            let mut pv: PROPVARIANT = core::mem::zeroed();
            if (pvtbl.GetValue)(props as *mut c_void, &PKEY_Device_FriendlyName, &mut pv) == S_OK {
                if pv.vt == 31 && !pv.pwszVal.is_null() {
                    let mut wlen = 0usize;
                    while *pv.pwszVal.add(wlen) != 0 && wlen < 256 {
                        wlen += 1;
                    }
                    let slice = core::slice::from_raw_parts(pv.pwszVal, wlen);
                    name_out.len = utf16_to_utf8(slice, &mut name_out.buf);
                    CoTaskMemFree(pv.pwszVal as *mut c_void);
                }
            }
            (pvtbl.Release)(props as *mut c_void);
        }
    }
    !id_out.is_empty()
}

unsafe fn get_default_endpoint(
    enumerator: *mut IMMDeviceEnumerator,
    flow: DWORD,
    role: DWORD,
    id_out: &mut FixedBuf<160>,
    name_out: &mut FixedBuf<160>,
) -> bool {
    id_out.clear();
    name_out.clear();
    unsafe {
        let vtbl = &*(*enumerator).lpVtbl;
        let mut dev: *mut IMMDevice = null_mut();
        if (vtbl.GetDefaultAudioEndpoint)(enumerator as *mut c_void, flow, role, &mut dev) == S_OK
            && !dev.is_null()
        {
            let ok = get_device_id_and_name(dev, id_out, name_out);
            let dvtbl = &*(*dev).lpVtbl;
            (dvtbl.Release)(dev as *mut c_void);
            ok
        } else {
            false
        }
    }
}

unsafe fn create_policy_config() -> *mut IPolicyConfig {
    unsafe {
        let mut policy: *mut IPolicyConfig = null_mut();
        let iids = [IID_IPolicyConfig_Win10, IID_IPolicyConfig_RS, IID_IPolicyConfigVista];
        let mut k = 0;
        while k < iids.len() {
            let hr = CoCreateInstance(
                &CLSID_PolicyConfigClient,
                null_mut(),
                CLSCTX_ALL,
                &iids[k],
                &mut policy as *mut *mut IPolicyConfig as *mut *mut c_void,
            );
            if hr == S_OK && !policy.is_null() {
                return policy;
            }
            policy = null_mut();
            k += 1;
        }
        null_mut()
    }
}

unsafe fn set_default_endpoint_all_roles(device_id_utf8: &[u8]) -> bool {
    if device_id_utf8.is_empty() {
        return false;
    }
    let mut wid = [0u16; 260];
    utf8_to_utf16_null(device_id_utf8, &mut wid);

    unsafe {
        let policy = create_policy_config();
        if policy.is_null() {
            return false;
        }
        let vtbl = &*(*policy).lpVtbl;
        let hr0 = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_CONSOLE);
        let hr1 = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_MULTIMEDIA);
        let hr2 = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_COMMUNICATIONS);
        (vtbl.Release)(policy as *mut c_void);
        hr0 == S_OK || hr1 == S_OK || hr2 == S_OK
    }
}

unsafe fn set_endpoint_per_role(
    console_id: &[u8],
    multimedia_id: &[u8],
    comm_id: &[u8],
) {
    unsafe {
        let policy = create_policy_config();
        if policy.is_null() {
            return;
        }
        let vtbl = &*(*policy).lpVtbl;
        let mut wid = [0u16; 260];
        if !console_id.is_empty() {
            utf8_to_utf16_null(console_id, &mut wid);
            let _ = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_CONSOLE);
        }
        if !multimedia_id.is_empty() {
            utf8_to_utf16_null(multimedia_id, &mut wid);
            let _ = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_MULTIMEDIA);
        }
        if !comm_id.is_empty() {
            utf8_to_utf16_null(comm_id, &mut wid);
            let _ = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_COMMUNICATIONS);
        }
        (vtbl.Release)(policy as *mut c_void);
    }
}

unsafe fn create_device_enumerator() -> *mut IMMDeviceEnumerator {
    unsafe {
        let _ = CoInitializeEx(null_mut(), COINIT_MULTITHREADED);
        let mut enumerator: *mut IMMDeviceEnumerator = null_mut();
        let hr = CoCreateInstance(
            &CLSID_MMDeviceEnumerator,
            null_mut(),
            CLSCTX_ALL,
            &IID_IMMDeviceEnumerator,
            &mut enumerator as *mut *mut IMMDeviceEnumerator as *mut *mut c_void,
        );
        if hr == S_OK {
            enumerator
        } else {
            null_mut()
        }
    }
}

/// Role-specific desktop audio snapshot so users with distinct `eConsole`, `eMultimedia`,
/// and `eCommunications` devices have each role restored to its exact pre-VR endpoint.
struct RoleSnapshots {
    render_console: FixedBuf<160>,
    render_multimedia: FixedBuf<160>,
    render_comm: FixedBuf<160>,
    capture_console: FixedBuf<160>,
    capture_multimedia: FixedBuf<160>,
    capture_comm: FixedBuf<160>,
}

impl RoleSnapshots {
    const fn new() -> Self {
        Self {
            render_console: FixedBuf::new(),
            render_multimedia: FixedBuf::new(),
            render_comm: FixedBuf::new(),
            capture_console: FixedBuf::new(),
            capture_multimedia: FixedBuf::new(),
            capture_comm: FixedBuf::new(),
        }
    }

    unsafe fn capture_initial(
        &mut self,
        enumerator: *mut IMMDeviceEnumerator,
        profile: &VrAudioProfile,
        custom_out: &[u8],
        custom_in: &[u8],
    ) {
        unsafe {
            let mut vr_out_id = FixedBuf::<160>::new();
            let mut vr_out_name = FixedBuf::<160>::new();
            let mut desk_out_id = FixedBuf::<160>::new();
            let mut desk_out_name = FixedBuf::<160>::new();
            resolve_active_endpoints(
                enumerator,
                E_RENDER,
                profile,
                custom_out,
                &mut vr_out_id,
                &mut vr_out_name,
                &mut desk_out_id,
                &mut desk_out_name,
            );
            let fallback_out = if !profile.desktop_playback_id.is_empty() {
                profile.desktop_playback_id.as_bytes()
            } else {
                desk_out_id.as_bytes()
            };

            Self::snapshot_role(
                enumerator,
                E_RENDER,
                E_CONSOLE,
                profile,
                custom_out,
                fallback_out,
                &mut self.render_console,
            );
            Self::snapshot_role(
                enumerator,
                E_RENDER,
                E_MULTIMEDIA,
                profile,
                custom_out,
                fallback_out,
                &mut self.render_multimedia,
            );
            Self::snapshot_role(
                enumerator,
                E_RENDER,
                E_COMMUNICATIONS,
                profile,
                custom_out,
                fallback_out,
                &mut self.render_comm,
            );

            let mut vr_in_id = FixedBuf::<160>::new();
            let mut vr_in_name = FixedBuf::<160>::new();
            let mut desk_in_id = FixedBuf::<160>::new();
            let mut desk_in_name = FixedBuf::<160>::new();
            resolve_active_endpoints(
                enumerator,
                E_CAPTURE,
                profile,
                custom_in,
                &mut vr_in_id,
                &mut vr_in_name,
                &mut desk_in_id,
                &mut desk_in_name,
            );
            let fallback_in = if !profile.desktop_record_id.is_empty() {
                profile.desktop_record_id.as_bytes()
            } else {
                desk_in_id.as_bytes()
            };

            Self::snapshot_role(
                enumerator,
                E_CAPTURE,
                E_CONSOLE,
                profile,
                custom_in,
                fallback_in,
                &mut self.capture_console,
            );
            Self::snapshot_role(
                enumerator,
                E_CAPTURE,
                E_MULTIMEDIA,
                profile,
                custom_in,
                fallback_in,
                &mut self.capture_multimedia,
            );
            Self::snapshot_role(
                enumerator,
                E_CAPTURE,
                E_COMMUNICATIONS,
                profile,
                custom_in,
                fallback_in,
                &mut self.capture_comm,
            );
        }
    }

    unsafe fn snapshot_role(
        enumerator: *mut IMMDeviceEnumerator,
        flow: DWORD,
        role: DWORD,
        profile: &VrAudioProfile,
        custom_pat: &[u8],
        fallback_desktop_id: &[u8],
        out: &mut FixedBuf<160>,
    ) {
        let mut cur_id = FixedBuf::<160>::new();
        let mut cur_name = FixedBuf::<160>::new();
        if unsafe { get_default_endpoint(enumerator, flow, role, &mut cur_id, &mut cur_name) } {
            let is_vr = if !custom_pat.is_empty() {
                contains_ignore_ascii_case(cur_name.as_bytes(), custom_pat)
                    || contains_ignore_ascii_case(cur_id.as_bytes(), custom_pat)
            } else {
                profile.matches_vr_name_or_id(cur_name.as_bytes(), cur_id.as_bytes(), b"")
            };
            if !is_vr {
                out.set_bytes(cur_id.as_bytes());
                return;
            }
        }
        out.set_bytes(fallback_desktop_id);
    }

    unsafe fn restore(&self) {
        unsafe {
            set_endpoint_per_role(
                self.render_console.as_bytes(),
                self.render_multimedia.as_bytes(),
                self.render_comm.as_bytes(),
            );
            set_endpoint_per_role(
                self.capture_console.as_bytes(),
                self.capture_multimedia.as_bytes(),
                self.capture_comm.as_bytes(),
            );
        }
    }

    unsafe fn sanitize_against_vr(
        &mut self,
        enumerator: *mut IMMDeviceEnumerator,
        profile: &VrAudioProfile,
        custom_out: &[u8],
        custom_in: &[u8],
    ) {
        unsafe {
            let mut vr_out_id = FixedBuf::<160>::new();
            let mut vr_out_name = FixedBuf::<160>::new();
            let mut desk_out_id = FixedBuf::<160>::new();
            let mut desk_out_name = FixedBuf::<160>::new();
            resolve_active_endpoints(
                enumerator,
                E_RENDER,
                profile,
                custom_out,
                &mut vr_out_id,
                &mut vr_out_name,
                &mut desk_out_id,
                &mut desk_out_name,
            );
            let fallback_out = if !profile.desktop_playback_id.is_empty() {
                profile.desktop_playback_id.as_bytes()
            } else {
                desk_out_id.as_bytes()
            };
            Self::fix_if_vr(&mut self.render_console, vr_out_id.as_bytes(), profile, fallback_out);
            Self::fix_if_vr(&mut self.render_multimedia, vr_out_id.as_bytes(), profile, fallback_out);
            Self::fix_if_vr(&mut self.render_comm, vr_out_id.as_bytes(), profile, fallback_out);

            let mut vr_in_id = FixedBuf::<160>::new();
            let mut vr_in_name = FixedBuf::<160>::new();
            let mut desk_in_id = FixedBuf::<160>::new();
            let mut desk_in_name = FixedBuf::<160>::new();
            resolve_active_endpoints(
                enumerator,
                E_CAPTURE,
                profile,
                custom_in,
                &mut vr_in_id,
                &mut vr_in_name,
                &mut desk_in_id,
                &mut desk_in_name,
            );
            let fallback_in = if !profile.desktop_record_id.is_empty() {
                profile.desktop_record_id.as_bytes()
            } else {
                desk_in_id.as_bytes()
            };
            Self::fix_if_vr(&mut self.capture_console, vr_in_id.as_bytes(), profile, fallback_in);
            Self::fix_if_vr(&mut self.capture_multimedia, vr_in_id.as_bytes(), profile, fallback_in);
            Self::fix_if_vr(&mut self.capture_comm, vr_in_id.as_bytes(), profile, fallback_in);
        }
    }

    fn fix_if_vr(
        slot: &mut FixedBuf<160>,
        active_vr_id: &[u8],
        profile: &VrAudioProfile,
        fallback_desktop_id: &[u8],
    ) {
        if slot.is_empty()
            || (!active_vr_id.is_empty() && eq_ignore_ascii_case(slot.as_bytes(), active_vr_id))
            || profile.matches_vr_name_or_id(b"", slot.as_bytes(), b"")
        {
            if !fallback_desktop_id.is_empty() {
                slot.set_bytes(fallback_desktop_id);
            }
        }
    }
}

unsafe fn resolve_active_endpoints(
    enumerator: *mut IMMDeviceEnumerator,
    flow: DWORD,
    profile: &VrAudioProfile,
    custom_pattern: &[u8],
    vr_id_out: &mut FixedBuf<160>,
    vr_name_out: &mut FixedBuf<160>,
    desktop_id_out: &mut FixedBuf<160>,
    desktop_name_out: &mut FixedBuf<160>,
) {
    unsafe {
        let vtbl = &*(*enumerator).lpVtbl;
        let mut coll: *mut IMMDeviceCollection = null_mut();
        if (vtbl.EnumAudioEndpoints)(
            enumerator as *mut c_void,
            flow,
            DEVICE_STATE_ACTIVE,
            &mut coll,
        ) != S_OK
            || coll.is_null()
        {
            return;
        }

        let cvtbl = &*(*coll).lpVtbl;
        let mut count = 0u32;
        let _ = (cvtbl.GetCount)(coll as *mut c_void, &mut count);

        let mut idx = 0u32;
        while idx < count {
            let mut dev: *mut IMMDevice = null_mut();
            if (cvtbl.Item)(coll as *mut c_void, idx, &mut dev) == S_OK && !dev.is_null() {
                let mut id = FixedBuf::<160>::new();
                let mut name = FixedBuf::<160>::new();
                if get_device_id_and_name(dev, &mut id, &mut name) {
                    let is_vr = if !custom_pattern.is_empty() {
                        contains_ignore_ascii_case(name.as_bytes(), custom_pattern)
                            || contains_ignore_ascii_case(id.as_bytes(), custom_pattern)
                    } else {
                        profile.matches_vr_name_or_id(name.as_bytes(), id.as_bytes(), b"")
                    };

                    if is_vr {
                        if vr_id_out.is_empty() {
                            vr_id_out.set_bytes(id.as_bytes());
                            vr_name_out.set_bytes(name.as_bytes());
                        }
                    } else if desktop_id_out.is_empty() {
                        desktop_id_out.set_bytes(id.as_bytes());
                        desktop_name_out.set_bytes(name.as_bytes());
                    }
                }
                let dvtbl = &*(*dev).lpVtbl;
                (dvtbl.Release)(dev as *mut c_void);
            }
            idx += 1;
        }
        (cvtbl.Release)(coll as *mut c_void);
    }
}

pub fn print_status(
    paths: Option<&SteamVrPaths>,
    profile: &VrAudioProfile,
    custom_out: &[u8],
    custom_in: &[u8],
) {
    let mut line = FixedBuf::<512>::new();
    let (vrserver_pid, vrmonitor_pid) = find_process_pids();
    let _ = write!(
        line,
        "SteamVR Running: {} (vrserver PID: {}, vrmonitor PID: {})",
        if vrserver_pid != 0 { "YES" } else { "NO" },
        vrserver_pid,
        vrmonitor_pid
    );
    println(line.as_str());

    if let Some(p) = paths {
        line.clear();
        let _ = write!(line, "SteamVR Config : {}", p.config_dir.as_str());
        println(line.as_str());
        line.clear();
        let _ = write!(line, "SteamVR Logs   : {}", p.log_dir.as_str());
        println(line.as_str());
    }

    if !profile.hmd_model.is_empty() {
        line.clear();
        let _ = write!(
            line,
            "Detected HMD   : {} {} (Adapter: '{}', USB: '{}')",
            profile.hmd_manufacturer.as_str(),
            profile.hmd_model.as_str(),
            profile.vr_adapter_tag.as_str(),
            profile.vr_vid_pid.as_str()
        );
        println(line.as_str());
    }

    line.clear();
    let _ = write!(
        line,
        "SteamVR Profile VR Output     : {} [{}]",
        profile.vr_playback_name.as_str(),
        profile.vr_playback_id.as_str()
    );
    println(line.as_str());

    line.clear();
    let _ = write!(
        line,
        "SteamVR Profile VR Input      : {} [{}]",
        profile.vr_record_name.as_str(),
        profile.vr_record_id.as_str()
    );
    println(line.as_str());

    line.clear();
    let _ = write!(
        line,
        "SteamVR Profile Desktop Output: {} [{}]",
        profile.desktop_playback_name.as_str(),
        profile.desktop_playback_id.as_str()
    );
    println(line.as_str());

    line.clear();
    let _ = write!(
        line,
        "SteamVR Profile Desktop Input : {} [{}]",
        profile.desktop_record_name.as_str(),
        profile.desktop_record_id.as_str()
    );
    println(line.as_str());

    unsafe {
        let enumerator = create_device_enumerator();
        if enumerator.is_null() {
            println("ERROR: Failed to initialize Windows Core Audio IMMDeviceEnumerator.");
            return;
        }

        let mut def_con_id = FixedBuf::<160>::new();
        let mut def_con_name = FixedBuf::<160>::new();
        let mut def_comm_id = FixedBuf::<160>::new();
        let mut def_comm_name = FixedBuf::<160>::new();

        println("");
        println("--- Active Windows Playback (Output) Endpoints ---");
        let _ = get_default_endpoint(enumerator, E_RENDER, E_CONSOLE, &mut def_con_id, &mut def_con_name);
        let _ = get_default_endpoint(enumerator, E_RENDER, E_COMMUNICATIONS, &mut def_comm_id, &mut def_comm_name);
        list_flow_endpoints(enumerator, E_RENDER, profile, custom_out, &def_con_id, &def_comm_id);

        println("");
        println("--- Active Windows Recording (Input) Endpoints ---");
        let _ = get_default_endpoint(enumerator, E_CAPTURE, E_CONSOLE, &mut def_con_id, &mut def_con_name);
        let _ = get_default_endpoint(enumerator, E_CAPTURE, E_COMMUNICATIONS, &mut def_comm_id, &mut def_comm_name);
        list_flow_endpoints(enumerator, E_CAPTURE, profile, custom_in, &def_con_id, &def_comm_id);

        let vtbl = &*(*enumerator).lpVtbl;
        (vtbl.Release)(enumerator as *mut c_void);
    }
}

unsafe fn list_flow_endpoints(
    enumerator: *mut IMMDeviceEnumerator,
    flow: DWORD,
    profile: &VrAudioProfile,
    custom_pat: &[u8],
    def_con_id: &FixedBuf<160>,
    def_comm_id: &FixedBuf<160>,
) {
    unsafe {
        let vtbl = &*(*enumerator).lpVtbl;
        let mut coll: *mut IMMDeviceCollection = null_mut();
        if (vtbl.EnumAudioEndpoints)(
            enumerator as *mut c_void,
            flow,
            DEVICE_STATE_ACTIVE,
            &mut coll,
        ) != S_OK
            || coll.is_null()
        {
            return;
        }
        let cvtbl = &*(*coll).lpVtbl;
        let mut count = 0u32;
        let _ = (cvtbl.GetCount)(coll as *mut c_void, &mut count);

        let mut i = 0u32;
        while i < count {
            let mut dev: *mut IMMDevice = null_mut();
            if (cvtbl.Item)(coll as *mut c_void, i, &mut dev) == S_OK && !dev.is_null() {
                let mut id = FixedBuf::<160>::new();
                let mut name = FixedBuf::<160>::new();
                if get_device_id_and_name(dev, &mut id, &mut name) {
                    let is_con = eq_ignore_ascii_case(id.as_bytes(), def_con_id.as_bytes());
                    let is_comm = eq_ignore_ascii_case(id.as_bytes(), def_comm_id.as_bytes());
                    let is_vr = if !custom_pat.is_empty() {
                        contains_ignore_ascii_case(name.as_bytes(), custom_pat)
                    } else {
                        profile.matches_vr_name_or_id(name.as_bytes(), id.as_bytes(), b"")
                    };
                    let mut line = FixedBuf::<512>::new();
                    let _ = write!(
                        line,
                        "  {}{}{} {} [{}]",
                        if is_con { "[Default] " } else { "" },
                        if is_comm { "[Discord/Comm] " } else { "" },
                        if is_vr { "[VR-HMD]" } else { "[Desktop]" },
                        name.as_str(),
                        id.as_str()
                    );
                    println(line.as_str());
                }
                let dvtbl = &*(*dev).lpVtbl;
                (dvtbl.Release)(dev as *mut c_void);
            }
            i += 1;
        }
        (cvtbl.Release)(coll as *mut c_void);
    }
}

/// Purely event-driven VR audio session:
/// 1. Snapshots each role (`eConsole`, `eMultimedia`, `eCommunications`) independently for Output and Input.
/// 2. Switches OS (`eConsole`, `eMultimedia`, `eCommunications`) + Discord to VR headset audio.
/// 3. Registers `IMMNotificationClient` and sleeps on `WaitForMultipleObjects([hVrServer, hAudioEvent], INFINITE)`
///    at 0% CPU with zero polling.
/// 4. When `vrserver.exe` exits, waits for `vrmonitor.exe` shutdown, restores each role to its snapshotted device, and exits.
pub fn run_event_driven_vr_session(
    paths: Option<&SteamVrPaths>,
    mut profile: VrAudioProfile,
    custom_out: &[u8],
    custom_in: &[u8],
    _move_all_streams: bool,
) {
    let (vrserver_pid, vrmonitor_pid) = find_process_pids();
    if vrserver_pid == 0 {
        println("SteamVR (vrserver.exe) is not running.");
        return;
    }

    unsafe {
        let h_vrserver = OpenProcess(SYNCHRONIZE, 0, vrserver_pid);
        if h_vrserver.is_null() {
            println("ERROR: Could not open SYNCHRONIZE handle to vrserver.exe.");
            return;
        }
        let h_vrmonitor = if vrmonitor_pid != 0 {
            OpenProcess(SYNCHRONIZE, 0, vrmonitor_pid)
        } else {
            null_mut()
        };

        let enumerator = create_device_enumerator();
        if enumerator.is_null() {
            CloseHandle(h_vrserver);
            if !h_vrmonitor.is_null() {
                CloseHandle(h_vrmonitor);
            }
            return;
        }

        // Snapshot each role (eConsole, eMultimedia, eCommunications) independently
        let mut snapshots = RoleSnapshots::new();
        snapshots.capture_initial(enumerator, &profile, custom_out, custom_in);

        // Create kernel auto-reset event for IMMNotificationClient callbacks
        let h_audio_event = CreateEventW(null_mut(), 0, 0, null());
        G_AUDIO_EVENT.store(h_audio_event, Ordering::Relaxed);

        let vtbl = &*(*enumerator).lpVtbl;
        let client_ptr: *mut IMMNotificationClient = &raw mut NOTIFICATION_CLIENT;
        let _ = (vtbl.RegisterEndpointNotificationCallback)(enumerator as *mut c_void, client_ptr);

        // Perform initial sync to VR audio endpoints and unload any unused transient COM DLLs
        sync_vr_endpoints(enumerator, paths, &mut profile, custom_out, custom_in);
        CoFreeUnusedLibrariesEx(0, 0);

        // Purely event-driven kernel wait (0% CPU, zero polling, infinite timeout)
        let handles = [h_vrserver, h_audio_event];
        loop {
            let wait_res = WaitForMultipleObjects(2, handles.as_ptr(), 0, INFINITE);
            if wait_res == WAIT_OBJECT_0 {
                // vrserver.exe has terminated!
                break;
            } else if wait_res == WAIT_OBJECT_0 + 1 {
                // Audio device plugged in or SteamVR changed eConsole/eMultimedia -> sync eCommunications + VR audio
                sync_vr_endpoints(enumerator, paths, &mut profile, custom_out, custom_in);
            } else {
                break;
            }
        }

        // Unregister notification client before restoring
        let _ = (vtbl.UnregisterEndpointNotificationCallback)(enumerator as *mut c_void, client_ptr);
        G_AUDIO_EVENT.store(null_mut(), Ordering::Relaxed);
        if !h_audio_event.is_null() {
            CloseHandle(h_audio_event);
        }

        // Wait up to 5s on vrmonitor.exe, AND poll for any remaining vrmonitor.exe process
        // (up to ~3s) because SteamVR's vrmonitor.exe performs its "[Audio] Audio shutdown!"
        // only on eConsole + eMultimedia right as it exits, after vrserver.exe has already closed.
        if !h_vrmonitor.is_null() {
            let _ = WaitForSingleObject(h_vrmonitor, 5000);
            CloseHandle(h_vrmonitor);
        } else {
            let mut wait_tries = 0u32;
            while wait_tries < 15 {
                let (_, vrmon_pid) = find_process_pids();
                if vrmon_pid == 0 {
                    break;
                }
                let h_mon = OpenProcess(SYNCHRONIZE, 0, vrmon_pid);
                if !h_mon.is_null() {
                    let _ = WaitForSingleObject(h_mon, 5000);
                    CloseHandle(h_mon);
                    break;
                }
                wait_tries += 1;
            }
        }
        CloseHandle(h_vrserver);

        // Refresh profile from vrmonitor.txt and re-check snapshots so any role that was
        // still on a VR device when we launched falls back to the non-VR desktop device.
        if let Some(p) = paths {
            let refreshed = VrAudioProfile::load(p);
            if profile.desktop_playback_id.is_empty() && !refreshed.desktop_playback_id.is_empty() {
                profile.desktop_playback_id.set_bytes(refreshed.desktop_playback_id.as_bytes());
                profile.desktop_playback_name.set_bytes(refreshed.desktop_playback_name.as_bytes());
            }
            if profile.desktop_record_id.is_empty() && !refreshed.desktop_record_id.is_empty() {
                profile.desktop_record_id.set_bytes(refreshed.desktop_record_id.as_bytes());
                profile.desktop_record_name.set_bytes(refreshed.desktop_record_name.as_bytes());
            }
        }
        snapshots.sanitize_against_vr(enumerator, &profile, custom_out, custom_in);

        // Restore each role (eConsole, eMultimedia, eCommunications) to its own pre-VR endpoint
        snapshots.restore();

        (vtbl.Release)(enumerator as *mut c_void);
    }
}

/// Repairs any role (`eConsole`, `eMultimedia`, `eCommunications`) that is currently stuck on a VR endpoint
/// while SteamVR is closed, without overwriting roles that are already assigned to non-VR desktop devices.
pub fn ensure_desktop_audio(
    profile: &VrAudioProfile,
    custom_out: &[u8],
    custom_in: &[u8],
    _move_all_streams: bool,
) {
    unsafe {
        let enumerator = create_device_enumerator();
        if enumerator.is_null() {
            return;
        }
        let mut snapshots = RoleSnapshots::new();
        snapshots.capture_initial(enumerator, profile, custom_out, custom_in);
        snapshots.restore();

        let vtbl = &*(*enumerator).lpVtbl;
        (vtbl.Release)(enumerator as *mut c_void);
    }
}

unsafe fn sync_vr_endpoints(
    enumerator: *mut IMMDeviceEnumerator,
    paths: Option<&SteamVrPaths>,
    profile: &mut VrAudioProfile,
    custom_out: &[u8],
    custom_in: &[u8],
) {
    if (profile.vr_playback_id.is_empty() || profile.vr_record_id.is_empty())
        && let Some(p) = paths
    {
        let refreshed = VrAudioProfile::load(p);
        if profile.vr_playback_id.is_empty() && !refreshed.vr_playback_id.is_empty() {
            profile.vr_playback_id.set_bytes(refreshed.vr_playback_id.as_bytes());
            profile.vr_playback_name.set_bytes(refreshed.vr_playback_name.as_bytes());
        }
        if profile.vr_record_id.is_empty() && !refreshed.vr_record_id.is_empty() {
            profile.vr_record_id.set_bytes(refreshed.vr_record_id.as_bytes());
            profile.vr_record_name.set_bytes(refreshed.vr_record_name.as_bytes());
        }
    }

    unsafe {
        let mut active_vr_out_id = FixedBuf::<160>::new();
        let mut active_vr_out_name = FixedBuf::<160>::new();
        let mut active_desk_out_id = FixedBuf::<160>::new();
        let mut active_desk_out_name = FixedBuf::<160>::new();

        resolve_active_endpoints(
            enumerator,
            E_RENDER,
            profile,
            custom_out,
            &mut active_vr_out_id,
            &mut active_vr_out_name,
            &mut active_desk_out_id,
            &mut active_desk_out_name,
        );

        if profile.desktop_playback_id.is_empty() && !active_desk_out_id.is_empty() {
            profile.desktop_playback_id.set_bytes(active_desk_out_id.as_bytes());
            profile.desktop_playback_name.set_bytes(active_desk_out_name.as_bytes());
        }

        if active_vr_out_id.is_empty() {
            let mut con_id = FixedBuf::<160>::new();
            let mut con_name = FixedBuf::<160>::new();
            if get_default_endpoint(enumerator, E_RENDER, E_CONSOLE, &mut con_id, &mut con_name) {
                if !profile.desktop_playback_id.is_empty()
                    && !eq_ignore_ascii_case(con_id.as_bytes(), profile.desktop_playback_id.as_bytes())
                {
                    active_vr_out_id.set_bytes(con_id.as_bytes());
                    active_vr_out_name.set_bytes(con_name.as_bytes());
                }
            }
        }

        if !active_vr_out_id.is_empty() {
            let mut comm_id = FixedBuf::<160>::new();
            let mut comm_name = FixedBuf::<160>::new();
            let _ = get_default_endpoint(
                enumerator,
                E_RENDER,
                E_COMMUNICATIONS,
                &mut comm_id,
                &mut comm_name,
            );
            let mut con_id = FixedBuf::<160>::new();
            let mut con_name = FixedBuf::<160>::new();
            let _ = get_default_endpoint(enumerator, E_RENDER, E_CONSOLE, &mut con_id, &mut con_name);

            if !eq_ignore_ascii_case(comm_id.as_bytes(), active_vr_out_id.as_bytes())
                || !eq_ignore_ascii_case(con_id.as_bytes(), active_vr_out_id.as_bytes())
            {
                let _ = set_default_endpoint_all_roles(active_vr_out_id.as_bytes());
            }
        }

        let mut active_vr_in_id = FixedBuf::<160>::new();
        let mut active_vr_in_name = FixedBuf::<160>::new();
        let mut active_desk_in_id = FixedBuf::<160>::new();
        let mut active_desk_in_name = FixedBuf::<160>::new();

        resolve_active_endpoints(
            enumerator,
            E_CAPTURE,
            profile,
            custom_in,
            &mut active_vr_in_id,
            &mut active_vr_in_name,
            &mut active_desk_in_id,
            &mut active_desk_in_name,
        );

        if profile.desktop_record_id.is_empty() && !active_desk_in_id.is_empty() {
            profile.desktop_record_id.set_bytes(active_desk_in_id.as_bytes());
            profile.desktop_record_name.set_bytes(active_desk_in_name.as_bytes());
        }

        if !active_vr_in_id.is_empty() {
            let mut comm_id = FixedBuf::<160>::new();
            let mut comm_name = FixedBuf::<160>::new();
            let _ = get_default_endpoint(
                enumerator,
                E_CAPTURE,
                E_COMMUNICATIONS,
                &mut comm_id,
                &mut comm_name,
            );
            let mut con_id = FixedBuf::<160>::new();
            let mut con_name = FixedBuf::<160>::new();
            let _ = get_default_endpoint(enumerator, E_CAPTURE, E_CONSOLE, &mut con_id, &mut con_name);

            if !eq_ignore_ascii_case(comm_id.as_bytes(), active_vr_in_id.as_bytes())
                || !eq_ignore_ascii_case(con_id.as_bytes(), active_vr_in_id.as_bytes())
            {
                let _ = set_default_endpoint_all_roles(active_vr_in_id.as_bytes());
            }
        }
    }
}
