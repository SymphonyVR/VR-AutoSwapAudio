#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::c_void;
use core::fmt::Write;
use core::ptr::null;

use crate::steamvr::{SteamVrPaths, VrAudioProfile};
use crate::str_buf::{
    FixedBuf, contains_ignore_ascii_case, eq_ignore_ascii_case, find_subslice, trim_ascii,
};

pub const PATH_SEP: char = '/';

type c_int = i32;
type c_uint = u32;
type c_long = i64;
type size_t = usize;
type ssize_t = isize;
type pid_t = i32;
type off_t = i64;

const O_RDONLY: c_int = 0;
const O_WRONLY: c_int = 1;
const O_CREAT: c_int = 0o100;
const O_TRUNC: c_int = 0o1000;
const SEEK_END: c_int = 2;

const POLLIN: i16 = 0x0001;
const POLLERR: i16 = 0x0008;
const POLLHUP: i16 = 0x0010;
const SIGTERM: c_int = 15;

#[cfg(target_arch = "x86_64")]
const SYS_PIDFD_OPEN: c_long = 434;
#[cfg(target_arch = "aarch64")]
const SYS_PIDFD_OPEN: c_long = 434;

#[repr(C)]
struct pollfd {
    fd: c_int,
    events: i16,
    revents: i16,
}

#[repr(C)]
struct dirent64 {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
    d_name: [u8; 256],
}

unsafe extern "C" {
    fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t;
    fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t;
    fn open(pathname: *const u8, flags: c_int, mode: c_uint) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn lseek(fd: c_int, offset: off_t, whence: c_int) -> off_t;
    fn _exit(status: c_int) -> !;
    fn getenv(name: *const u8) -> *const u8;
    fn readlink(pathname: *const u8, buf: *mut u8, bufsiz: size_t) -> ssize_t;
    fn opendir(name: *const u8) -> *mut c_void;
    fn readdir64(dirp: *mut c_void) -> *mut dirent64;
    fn closedir(dirp: *mut c_void) -> c_int;
    fn pipe(pipefd: *mut c_int) -> c_int;
    fn fork() -> pid_t;
    fn dup2(oldfd: c_int, newfd: c_int) -> c_int;
    fn execvp(file: *const u8, argv: *const *const u8) -> c_int;
    fn waitpid(pid: pid_t, wstatus: *mut c_int, options: c_int) -> pid_t;
    fn kill(pid: pid_t, sig: c_int) -> c_int;
    fn poll(fds: *mut pollfd, nfds: u64, timeout: c_int) -> c_int;
    fn syscall(num: c_long, ...) -> c_long;
}

static mut G_ARGC: c_int = 0;
static mut G_ARGV: *const *const u8 = null();

pub unsafe fn save_argc_argv(argc: c_int, argv: *const *const u8) {
    unsafe {
        G_ARGC = argc;
        G_ARGV = argv;
    }
}

pub fn print(s: &str) {
    let bytes = s.as_bytes();
    if !bytes.is_empty() {
        unsafe {
            let _ = write(1, bytes.as_ptr() as *const c_void, bytes.len());
        }
    }
}

pub fn println(s: &str) {
    print(s);
    print("\n");
}

pub fn exit(code: u32) -> ! {
    unsafe { _exit(code as c_int) }
}

unsafe fn cstr_len(ptr: *const u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let mut len = 0usize;
    unsafe {
        while *ptr.add(len) != 0 && len < 4096 {
            len += 1;
        }
    }
    len
}

fn to_cstr<const N: usize>(bytes: &[u8]) -> [u8; N] {
    let mut out = [0u8; N];
    let limit = if bytes.len() + 1 < N { bytes.len() } else { N.saturating_sub(1) };
    let mut i = 0;
    while i < limit {
        out[i] = bytes[i];
        i += 1;
    }
    out[limit] = 0;
    out
}

pub fn get_openvrpaths_file(out: &mut FixedBuf<512>) -> bool {
    unsafe {
        let xdg = getenv(b"XDG_CONFIG_HOME\0".as_ptr());
        if !xdg.is_null() {
            let len = cstr_len(xdg);
            if len > 0 {
                out.set_bytes(core::slice::from_raw_parts(xdg, len));
                out.push_str("/openvr/openvrpaths.vrpath");
                return true;
            }
        }
        let home = getenv(b"HOME\0".as_ptr());
        if !home.is_null() {
            let len = cstr_len(home);
            if len > 0 {
                out.set_bytes(core::slice::from_raw_parts(home, len));
                out.push_str("/.config/openvr/openvrpaths.vrpath");
                return true;
            }
        }
    }
    false
}

pub fn get_exe_dir(out: &mut FixedBuf<512>) -> bool {
    let mut buf = [0u8; 512];
    let n = unsafe { readlink(b"/proc/self/exe\0".as_ptr(), buf.as_mut_ptr(), buf.len()) };
    if n <= 0 {
        return false;
    }
    let slice = &buf[..n as usize];
    let mut last_sep = None;
    let mut i = 0;
    while i < slice.len() {
        if slice[i] == b'/' {
            last_sep = Some(i);
        }
        i += 1;
    }
    if let Some(idx) = last_sep {
        out.set_bytes(&slice[..idx]);
        true
    } else {
        false
    }
}

pub fn read_file_bytes(path: &[u8], out: &mut [u8]) -> usize {
    let cpath = to_cstr::<520>(path);
    unsafe {
        let fd = open(cpath.as_ptr(), O_RDONLY, 0);
        if fd < 0 {
            return 0;
        }
        let n = read(fd, out.as_mut_ptr() as *mut c_void, out.len());
        let _ = close(fd);
        if n > 0 { n as usize } else { 0 }
    }
}

pub fn read_file_tail_bytes(path: &[u8], out: &mut [u8]) -> usize {
    let cpath = to_cstr::<520>(path);
    unsafe {
        let fd = open(cpath.as_ptr(), O_RDONLY, 0);
        if fd < 0 {
            return 0;
        }
        let _ = lseek(fd, -(out.len() as off_t), SEEK_END);
        let n = read(fd, out.as_mut_ptr() as *mut c_void, out.len());
        let _ = close(fd);
        if n > 0 { n as usize } else { 0 }
    }
}

pub fn write_file_bytes(path: &[u8], data: &[u8]) -> bool {
    let cpath = to_cstr::<520>(path);
    unsafe {
        let fd = open(cpath.as_ptr(), O_WRONLY | O_CREAT | O_TRUNC, 0o644);
        if fd < 0 {
            return false;
        }
        let n = write(fd, data.as_ptr() as *const c_void, data.len());
        let _ = close(fd);
        n >= 0 && (n as usize) == data.len()
    }
}

pub fn parse_cli_args(out_args: &mut [FixedBuf<128>; 8]) -> usize {
    unsafe {
        if G_ARGV.is_null() || G_ARGC <= 1 {
            return 0;
        }
        let mut count = 0usize;
        let mut i = 1;
        while i < G_ARGC && count < out_args.len() {
            let ptr = *G_ARGV.offset(i as isize);
            if !ptr.is_null() {
                let len = cstr_len(ptr);
                out_args[count].set_bytes(core::slice::from_raw_parts(ptr, len));
                count += 1;
            }
            i += 1;
        }
        count
    }
}

pub fn find_process_pids() -> (u32, u32) {
    let mut vrserver_pid = 0u32;
    let mut vrmonitor_pid = 0u32;
    unsafe {
        let dir = opendir(b"/proc\0".as_ptr());
        if dir.is_null() {
            return (0, 0);
        }
        loop {
            let ent = readdir64(dir);
            if ent.is_null() {
                break;
            }
            let name_ptr = (*ent).d_name.as_ptr();
            let nlen = cstr_len(name_ptr);
            let name = core::slice::from_raw_parts(name_ptr, nlen);
            if name.is_empty() || !name[0].is_ascii_digit() {
                continue;
            }
            let mut pid = 0u32;
            let mut valid = true;
            let mut k = 0;
            while k < name.len() {
                if !name[k].is_ascii_digit() {
                    valid = false;
                    break;
                }
                pid = pid.saturating_mul(10).saturating_add((name[k] - b'0') as u32);
                k += 1;
            }
            if !valid || pid == 0 {
                continue;
            }

            let mut comm_path = FixedBuf::<64>::new();
            let _ = write!(comm_path, "/proc/{}/comm", pid);
            let mut comm_buf = [0u8; 64];
            let cn = read_file_bytes(comm_path.as_bytes(), &mut comm_buf);
            if cn > 0 {
                let comm = trim_ascii(&comm_buf[..cn]);
                if eq_ignore_ascii_case(comm, b"vrserver") {
                    vrserver_pid = pid;
                } else if eq_ignore_ascii_case(comm, b"vrmonitor") {
                    vrmonitor_pid = pid;
                }
            }
        }
        let _ = closedir(dir);
    }
    (vrserver_pid, vrmonitor_pid)
}

fn run_pactl_capture(args: &[&[u8]], out: &mut [u8]) -> usize {
    unsafe {
        let mut pfd = [0i32; 2];
        if pipe(pfd.as_mut_ptr()) != 0 {
            return 0;
        }
        let pid = fork();
        if pid < 0 {
            let _ = close(pfd[0]);
            let _ = close(pfd[1]);
            return 0;
        }
        if pid == 0 {
            let _ = close(pfd[0]);
            let _ = dup2(pfd[1], 1);
            let devnull = open(b"/dev/null\0".as_ptr(), O_WRONLY, 0);
            if devnull >= 0 {
                let _ = dup2(devnull, 2);
                let _ = close(devnull);
            }
            let _ = close(pfd[1]);

            let mut c_args = [[0u8; 160]; 6];
            let mut ptrs: [*const u8; 7] = [null(); 7];
            c_args[0] = to_cstr::<160>(b"pactl");
            ptrs[0] = c_args[0].as_ptr();
            let mut i = 0;
            while i < args.len() && i + 1 < 6 {
                c_args[i + 1] = to_cstr::<160>(args[i]);
                ptrs[i + 1] = c_args[i + 1].as_ptr();
                i += 1;
            }
            let _ = execvp(c_args[0].as_ptr(), ptrs.as_ptr());
            _exit(127);
        }

        let _ = close(pfd[1]);
        let mut total = 0usize;
        while total < out.len() {
            let n = read(
                pfd[0],
                out[total..].as_mut_ptr() as *mut c_void,
                out.len() - total,
            );
            if n <= 0 {
                break;
            }
            total += n as usize;
        }
        let _ = close(pfd[0]);
        let mut status = 0;
        let _ = waitpid(pid, &mut status, 0);
        total
    }
}

fn move_all_pactl_streams(list_type: &[u8], move_cmd: &[u8], target_device: &[u8]) {
    if target_device.is_empty() {
        return;
    }
    let mut buf = [0u8; 4096];
    let n = run_pactl_capture(&[b"list", b"short", list_type], &mut buf);
    if n == 0 {
        return;
    }
    let data = &buf[..n];
    let mut line_start = 0;
    while line_start < data.len() {
        let mut line_end = line_start;
        while line_end < data.len() && data[line_end] != b'\n' {
            line_end += 1;
        }
        let line = trim_ascii(&data[line_start..line_end]);
        line_start = line_end + 1;
        if line.is_empty() {
            continue;
        }
        let mut id_end = 0;
        while id_end < line.len() && line[id_end].is_ascii_digit() {
            id_end += 1;
        }
        if id_end > 0 {
            let stream_id = &line[..id_end];
            let mut dummy = [0u8; 64];
            let _ = run_pactl_capture(&[move_cmd, stream_id, target_device], &mut dummy);
        }
    }
}

fn switch_linux_audio(sink: &[u8], source: &[u8]) {
    let mut dummy = [0u8; 64];
    if !sink.is_empty() {
        let _ = run_pactl_capture(&[b"set-default-sink", sink], &mut dummy);
        move_all_pactl_streams(b"sink-inputs", b"move-sink-input", sink);
    }
    if !source.is_empty() {
        let _ = run_pactl_capture(&[b"set-default-source", source], &mut dummy);
        move_all_pactl_streams(b"source-outputs", b"move-source-output", source);
    }
}

fn find_pactl_devices(
    kind: &[u8],
    profile: &VrAudioProfile,
    custom_pat: &[u8],
    vr_name_out: &mut FixedBuf<160>,
    vr_desc_out: &mut FixedBuf<160>,
    desk_name_out: &mut FixedBuf<160>,
    desk_desc_out: &mut FixedBuf<160>,
) {
    let mut buf = [0u8; 16384];
    let n = run_pactl_capture(&[b"list", kind], &mut buf);
    if n == 0 {
        return;
    }
    let text = &buf[..n];
    let mut cur_name = FixedBuf::<160>::new();
    let mut cur_desc = FixedBuf::<160>::new();

    let mut line_start = 0;
    while line_start < text.len() {
        let mut line_end = line_start;
        while line_end < text.len() && text[line_end] != b'\n' {
            line_end += 1;
        }
        let line = trim_ascii(&text[line_start..line_end]);
        line_start = line_end + 1;

        if line.starts_with(b"Name:") {
            cur_name.set_bytes(trim_ascii(&line[5..]));
        } else if line.starts_with(b"Description:") {
            cur_desc.set_bytes(trim_ascii(&line[12..]));
            if !cur_name.is_empty() && !cur_desc.is_empty() {
                // Ignore monitor sources when scanning microphone sources
                if eq_ignore_ascii_case(kind, b"sources")
                    && contains_ignore_ascii_case(cur_name.as_bytes(), b".monitor")
                {
                    continue;
                }
                let is_vr = if !custom_pat.is_empty() {
                    contains_ignore_ascii_case(cur_desc.as_bytes(), custom_pat)
                        || contains_ignore_ascii_case(cur_name.as_bytes(), custom_pat)
                } else {
                    profile.matches_vr_name_or_id(cur_desc.as_bytes(), cur_name.as_bytes(), b"")
                };
                if is_vr {
                    if vr_name_out.is_empty() {
                        vr_name_out.set_bytes(cur_name.as_bytes());
                        vr_desc_out.set_bytes(cur_desc.as_bytes());
                    }
                } else if desk_name_out.is_empty() {
                    desk_name_out.set_bytes(cur_name.as_bytes());
                    desk_desc_out.set_bytes(cur_desc.as_bytes());
                }
            }
        }
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
    }

    let mut vr_sink = FixedBuf::<160>::new();
    let mut vr_sink_desc = FixedBuf::<160>::new();
    let mut desk_sink = FixedBuf::<160>::new();
    let mut desk_sink_desc = FixedBuf::<160>::new();
    find_pactl_devices(
        b"sinks",
        profile,
        custom_out,
        &mut vr_sink,
        &mut vr_sink_desc,
        &mut desk_sink,
        &mut desk_sink_desc,
    );

    let mut vr_src = FixedBuf::<160>::new();
    let mut vr_src_desc = FixedBuf::<160>::new();
    let mut desk_src = FixedBuf::<160>::new();
    let mut desk_src_desc = FixedBuf::<160>::new();
    find_pactl_devices(
        b"sources",
        profile,
        custom_in,
        &mut vr_src,
        &mut vr_src_desc,
        &mut desk_src,
        &mut desk_src_desc,
    );

    line.clear();
    let _ = write!(line, "VR Sink        : {} [{}]", vr_sink_desc.as_str(), vr_sink.as_str());
    println(line.as_str());
    line.clear();
    let _ = write!(line, "VR Source      : {} [{}]", vr_src_desc.as_str(), vr_src.as_str());
    println(line.as_str());
    line.clear();
    let _ = write!(line, "Desktop Sink   : {} [{}]", desk_sink_desc.as_str(), desk_sink.as_str());
    println(line.as_str());
    line.clear();
    let _ = write!(line, "Desktop Source : {} [{}]", desk_src_desc.as_str(), desk_src.as_str());
    println(line.as_str());
}

/// 100% Event-Driven Linux VR Audio Session using `pidfd_open(vrserver_pid)` + `pactl subscribe` + `poll(-1)`.
pub fn run_event_driven_vr_session(
    _paths: Option<&SteamVrPaths>,
    profile: VrAudioProfile,
    custom_out: &[u8],
    custom_in: &[u8],
) {
    let (vrserver_pid, _vrmonitor_pid) = find_process_pids();
    if vrserver_pid == 0 {
        println("SteamVR (vrserver) is not running.");
        return;
    }

    unsafe {
        let pidfd = syscall(SYS_PIDFD_OPEN, vrserver_pid as c_long, 0 as c_long) as c_int;
        if pidfd < 0 {
            println("ERROR: pidfd_open failed for vrserver.");
            return;
        }

        let mut vr_sink = FixedBuf::<160>::new();
        let mut vr_sink_desc = FixedBuf::<160>::new();
        let mut desk_sink = FixedBuf::<160>::new();
        let mut desk_sink_desc = FixedBuf::<160>::new();
        find_pactl_devices(
            b"sinks",
            &profile,
            custom_out,
            &mut vr_sink,
            &mut vr_sink_desc,
            &mut desk_sink,
            &mut desk_sink_desc,
        );

        let mut vr_src = FixedBuf::<160>::new();
        let mut vr_src_desc = FixedBuf::<160>::new();
        let mut desk_src = FixedBuf::<160>::new();
        let mut desk_src_desc = FixedBuf::<160>::new();
        find_pactl_devices(
            b"sources",
            &profile,
            custom_in,
            &mut vr_src,
            &mut vr_src_desc,
            &mut desk_src,
            &mut desk_src_desc,
        );

        // Initial switch to VR audio + move Discord streams
        switch_linux_audio(vr_sink.as_bytes(), vr_src.as_bytes());

        // Spawn `pactl subscribe` to receive real-time PipeWire/PulseAudio events without polling
        let mut pfd = [-1i32; 2];
        let mut sub_pid: pid_t = -1;
        if pipe(pfd.as_mut_ptr()) == 0 {
            sub_pid = fork();
            if sub_pid == 0 {
                let _ = close(pfd[0]);
                let _ = dup2(pfd[1], 1);
                let _ = close(pfd[1]);
                let c0 = b"pactl\0";
                let c1 = b"subscribe\0";
                let argv: [*const u8; 3] = [c0.as_ptr(), c1.as_ptr(), null()];
                let _ = execvp(c0.as_ptr(), argv.as_ptr());
                _exit(127);
            }
            let _ = close(pfd[1]);
        }

        let mut fds = [
            pollfd {
                fd: pidfd,
                events: POLLIN,
                revents: 0,
            },
            pollfd {
                fd: pfd[0],
                events: POLLIN,
                revents: 0,
            },
        ];
        let nfds = if pfd[0] >= 0 { 2 } else { 1 };

        let mut ev_buf = [0u8; 1024];
        loop {
            let rc = poll(fds.as_mut_ptr(), nfds, -1); // Infinite wait (-1), 0% CPU
            if rc <= 0 {
                break;
            }
            if (fds[0].revents & (POLLIN | POLLHUP | POLLERR)) != 0 {
                // vrserver has exited!
                break;
            }
            if nfds == 2 && (fds[1].revents & POLLIN) != 0 {
                let n = read(pfd[0], ev_buf.as_mut_ptr() as *mut c_void, ev_buf.len());
                if n > 0 {
                    let ev = &ev_buf[..n as usize];
                    // If a new sink/source or new Discord stream ('new' on sink-input / source-output) appeared
                    if find_subslice(ev, b"'new'").is_some() {
                        if vr_sink.is_empty() || vr_src.is_empty() {
                            find_pactl_devices(
                                b"sinks",
                                &profile,
                                custom_out,
                                &mut vr_sink,
                                &mut vr_sink_desc,
                                &mut desk_sink,
                                &mut desk_sink_desc,
                            );
                            find_pactl_devices(
                                b"sources",
                                &profile,
                                custom_in,
                                &mut vr_src,
                                &mut vr_src_desc,
                                &mut desk_src,
                                &mut desk_src_desc,
                            );
                        }
                        switch_linux_audio(vr_sink.as_bytes(), vr_src.as_bytes());
                    }
                } else {
                    break;
                }
            }
        }

        if sub_pid > 0 {
            let _ = kill(sub_pid, SIGTERM);
            let mut st = 0;
            let _ = waitpid(sub_pid, &mut st, 0);
        }
        if pfd[0] >= 0 {
            let _ = close(pfd[0]);
        }
        let _ = close(pidfd);

        // Restore desktop default sink/source and move Discord streams back
        switch_linux_audio(desk_sink.as_bytes(), desk_src.as_bytes());
    }
}

pub fn ensure_desktop_audio(profile: &VrAudioProfile, custom_out: &[u8], custom_in: &[u8]) {
    let mut vr_sink = FixedBuf::<160>::new();
    let mut vr_sink_desc = FixedBuf::<160>::new();
    let mut desk_sink = FixedBuf::<160>::new();
    let mut desk_sink_desc = FixedBuf::<160>::new();
    find_pactl_devices(
        b"sinks",
        profile,
        custom_out,
        &mut vr_sink,
        &mut vr_sink_desc,
        &mut desk_sink,
        &mut desk_sink_desc,
    );

    let mut vr_src = FixedBuf::<160>::new();
    let mut vr_src_desc = FixedBuf::<160>::new();
    let mut desk_src = FixedBuf::<160>::new();
    let mut desk_src_desc = FixedBuf::<160>::new();
    find_pactl_devices(
        b"sources",
        profile,
        custom_in,
        &mut vr_src,
        &mut vr_src_desc,
        &mut desk_src,
        &mut desk_src_desc,
    );

    switch_linux_audio(desk_sink.as_bytes(), desk_src.as_bytes());
}

