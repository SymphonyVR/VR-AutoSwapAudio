#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_void;
use core::fmt::Write;
use core::ptr::{null, null_mut};
use core::sync::atomic::{AtomicPtr, Ordering};

use crate::steamvr::{SteamVrPaths, VrAudioProfile};
use crate::str_buf::{
    FixedBuf, contains_ignore_ascii_case, eq_ignore_ascii_case, utf8_to_utf16_null, utf16_to_utf8,
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
    fn CreateFileW(
        lpFileName: *const u16,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: *mut c_void,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
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
    fn CommandLineToArgvW(lpCmdLine: *const u16, pNumArgs: *mut i32) -> *mut *mut u16;
    fn LocalFree(hMem: *mut c_void) -> *mut c_void;

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
    // When SteamVR changes eConsole or eMultimedia, wake up our event loop to sync eCommunications
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
    // %LOCALAPPDATA%\openvr\openvrpaths.vrpath
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

pub fn write_file_bytes(path: &[u8], data: &[u8]) -> bool {
    let mut wpath = [0u16; 520];
    utf8_to_utf16_null(path, &mut wpath);
    unsafe {
        let h = CreateFileW(
            wpath.as_ptr(),
            GENERIC_WRITE,
            FILE_SHARE_READ,
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
        CloseHandle(h);
        ok != 0 && (written as usize) == data.len()
    }
}

pub fn parse_cli_args(out_args: &mut [FixedBuf<128>; 8]) -> usize {
    unsafe {
        let cmd = GetCommandLineW();
        if cmd.is_null() {
            return 0;
        }
        let mut argc = 0i32;
        let argv = CommandLineToArgvW(cmd, &mut argc);
        if argv.is_null() {
            return 0;
        }
        let mut count = 0usize;
        let mut i = 1i32; // Skip argv[0]
        while i < argc && count < out_args.len() {
            let wptr = *argv.offset(i as isize);
            if !wptr.is_null() {
                let mut wlen = 0usize;
                while *wptr.add(wlen) != 0 && wlen < 256 {
                    wlen += 1;
                }
                let slice = core::slice::from_raw_parts(wptr, wlen);
                out_args[count].len = utf16_to_utf8(slice, &mut out_args[count].buf);
                count += 1;
            }
            i += 1;
        }
        LocalFree(argv as *mut c_void);
        count
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

unsafe fn set_default_endpoint_all_roles(device_id_utf8: &[u8]) -> bool {
    if device_id_utf8.is_empty() {
        return false;
    }
    let mut wid = [0u16; 260];
    utf8_to_utf16_null(device_id_utf8, &mut wid);

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
                break;
            }
            policy = null_mut();
            k += 1;
        }
        if policy.is_null() {
            return false;
        }

        let vtbl = &*(*policy).lpVtbl;
        // Switch all 3 Windows Audio roles:
        // - eConsole (0): Games & system audio
        // - eMultimedia (1): Media & general audio
        // - eCommunications (2): Discord voice & microphone!
        let hr0 = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_CONSOLE);
        let hr1 = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_MULTIMEDIA);
        let hr2 = (vtbl.SetDefaultEndpoint)(policy as *mut c_void, wid.as_ptr(), E_COMMUNICATIONS);
        (vtbl.Release)(policy as *mut c_void);

        hr0 == S_OK || hr1 == S_OK || hr2 == S_OK
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

/// Scans active OS endpoints for `flow` (`E_RENDER` or `E_CAPTURE`) to find:
/// 1) The active VR endpoint matching `profile` (or `custom_pattern` if provided)
/// 2) The active non-VR Desktop endpoint (for fallback restore if needed)
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
/// 1. Snapshots non-VR desktop Output and Input endpoints.
/// 2. Switches OS (`eConsole`, `eMultimedia`, `eCommunications`) + Discord to VR headset audio.
/// 3. Registers `IMMNotificationClient` and sleeps on `WaitForMultipleObjects([hVrServer, hAudioEvent], INFINITE)`
///    at 0% CPU with zero polling.
/// 4. When `vrserver.exe` exits, waits for `vrmonitor.exe` shutdown, restores Desktop Output & Input across all 3 roles, and exits.
pub fn run_event_driven_vr_session(
    paths: Option<&SteamVrPaths>,
    mut profile: VrAudioProfile,
    custom_out: &[u8],
    custom_in: &[u8],
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

        // Snapshot current OS default endpoints before switching.
        // If current OS default is NOT already a VR endpoint, prefer it as the Desktop restore target!
        let mut cur_out_id = FixedBuf::<160>::new();
        let mut cur_out_name = FixedBuf::<160>::new();
        if get_default_endpoint(enumerator, E_RENDER, E_CONSOLE, &mut cur_out_id, &mut cur_out_name) {
            if !profile.matches_vr_name_or_id(cur_out_name.as_bytes(), cur_out_id.as_bytes(), b"") {
                profile.desktop_playback_id.set_bytes(cur_out_id.as_bytes());
                profile.desktop_playback_name.set_bytes(cur_out_name.as_bytes());
            }
        }

        let mut cur_in_id = FixedBuf::<160>::new();
        let mut cur_in_name = FixedBuf::<160>::new();
        if get_default_endpoint(enumerator, E_CAPTURE, E_CONSOLE, &mut cur_in_id, &mut cur_in_name) {
            if !profile.matches_vr_name_or_id(cur_in_name.as_bytes(), cur_in_id.as_bytes(), b"") {
                profile.desktop_record_id.set_bytes(cur_in_id.as_bytes());
                profile.desktop_record_name.set_bytes(cur_in_name.as_bytes());
            }
        }

        // Create kernel auto-reset event for IMMNotificationClient callbacks
        let h_audio_event = CreateEventW(null_mut(), 0, 0, null());
        G_AUDIO_EVENT.store(h_audio_event, Ordering::Relaxed);

        let vtbl = &*(*enumerator).lpVtbl;
        let client_ptr: *mut IMMNotificationClient = &raw mut NOTIFICATION_CLIENT;
        let _ = (vtbl.RegisterEndpointNotificationCallback)(enumerator as *mut c_void, client_ptr);

        // Perform initial sync to VR audio endpoints
        sync_vr_endpoints(enumerator, paths, &mut profile, custom_out, custom_in);

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

        // If vrmonitor.exe is still shutting down, wait up to 5s on its process handle so vrmonitor's
        // exit routine doesn't overwrite our restored endpoints.
        if !h_vrmonitor.is_null() {
            let _ = WaitForSingleObject(h_vrmonitor, 5000);
            CloseHandle(h_vrmonitor);
        }
        CloseHandle(h_vrserver);

        // Restore pre-VR Desktop Output & Input across all 3 roles (eConsole, eMultimedia, eCommunications)
        if !profile.desktop_playback_id.is_empty() {
            let _ = set_default_endpoint_all_roles(profile.desktop_playback_id.as_bytes());
        }
        if !profile.desktop_record_id.is_empty() {
            let _ = set_default_endpoint_all_roles(profile.desktop_record_id.as_bytes());
        }

        (vtbl.Release)(enumerator as *mut c_void);
    }
}

pub fn ensure_desktop_audio(profile: &VrAudioProfile, custom_out: &[u8], custom_in: &[u8]) {
    unsafe {
        let enumerator = create_device_enumerator();
        if enumerator.is_null() {
            return;
        }
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
        let target_out = if !profile.desktop_playback_id.is_empty() {
            profile.desktop_playback_id.as_bytes()
        } else {
            desk_out_id.as_bytes()
        };
        if !target_out.is_empty() {
            let _ = set_default_endpoint_all_roles(target_out);
        }

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
        let target_in = if !profile.desktop_record_id.is_empty() {
            profile.desktop_record_id.as_bytes()
        } else {
            desk_in_id.as_bytes()
        };
        if !target_in.is_empty() {
            let _ = set_default_endpoint_all_roles(target_in);
        }

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
    // Re-read vrmonitor.txt if VR playback/record IDs were not yet populated at initial startup
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

        // If SteamVR switched eConsole to a non-desktop device, treat that as the active VR playback device
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
            // Check if eConsole or eCommunications is not yet set to active_vr_out_id
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

        // Now resolve and sync Recording (Input / Microphone)
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
