#![no_std]
#![no_main]

mod str_buf;
mod steamvr;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod platform;

use core::panic::PanicInfo;
use steamvr::{SteamVrPaths, VrAudioProfile, register_autolaunch, unregister_autolaunch};
use str_buf::{FixedBuf, eq_ignore_ascii_case};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    platform::println("FATAL: VR-AutoSwapAudio panic.");
    platform::exit(1);
}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub static _fltused: i32 = 0;

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn mainCRTStartup() -> ! {
    let code = app_main();
    platform::exit(code);
}

#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    unsafe { platform::save_argc_argv(argc, argv) };
    app_main() as i32
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum RunMode {
    Auto,
    SteamVrLaunched,
    Register,
    Unregister,
    Status,
    Help,
}

fn print_help() {
    platform::println("VR-AutoSwapAudio (no_std, zero-dependency, 100% event-driven)");
    platform::println("Usage:");
    platform::println("  VR-AutoSwapAudio [OPTIONS]");
    platform::println("");
    platform::println("Options:");
    platform::println("  --status, --list       Show SteamVR HMD & active OS audio endpoints");
    platform::println("  --register             Register auto-launch .vrmanifest with SteamVR");
    platform::println("  --unregister           Remove auto-launch .vrmanifest from SteamVR");
    platform::println("  --steamvr-launched     Run event-driven VR audio session (used by SteamVR)");
    platform::println("  --output <pattern>     Optional custom VR output device substring override");
    platform::println("  --input <pattern>      Optional custom VR input device substring override");
    platform::println("  --help, -h             Show this help message");
}

fn app_main() -> u32 {
    let mut args = [
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
        FixedBuf::<128>::new(),
    ];
    let argc = platform::parse_cli_args(&mut args);

    let mut mode = RunMode::Auto;
    let mut custom_out = FixedBuf::<128>::new();
    let mut custom_in = FixedBuf::<128>::new();

    let mut i = 0usize;
    while i < argc {
        let a = args[i].as_bytes();
        if eq_ignore_ascii_case(a, b"--help") || eq_ignore_ascii_case(a, b"-h") {
            mode = RunMode::Help;
        } else if eq_ignore_ascii_case(a, b"--status") || eq_ignore_ascii_case(a, b"--list") {
            mode = RunMode::Status;
        } else if eq_ignore_ascii_case(a, b"--register") || eq_ignore_ascii_case(a, b"--install") {
            mode = RunMode::Register;
        } else if eq_ignore_ascii_case(a, b"--unregister") || eq_ignore_ascii_case(a, b"--uninstall") {
            mode = RunMode::Unregister;
        } else if eq_ignore_ascii_case(a, b"--steamvr-launched") {
            mode = RunMode::SteamVrLaunched;
        } else if eq_ignore_ascii_case(a, b"--output") && i + 1 < argc {
            i += 1;
            custom_out.set_bytes(args[i].as_bytes());
        } else if eq_ignore_ascii_case(a, b"--input") && i + 1 < argc {
            i += 1;
            custom_in.set_bytes(args[i].as_bytes());
        }
        i += 1;
    }

    if mode == RunMode::Help {
        print_help();
        return 0;
    }

    let paths = SteamVrPaths::discover();
    let profile = if let Some(ref p) = paths {
        VrAudioProfile::load(p)
    } else {
        VrAudioProfile::new()
    };

    match mode {
        RunMode::Help => 0,
        RunMode::Status => {
            platform::print_status(
                paths.as_ref(),
                &profile,
                custom_out.as_bytes(),
                custom_in.as_bytes(),
            );
            0
        }
        RunMode::Register => {
            if let Some(ref p) = paths {
                if register_autolaunch(p) {
                    platform::println("SUCCESS: Registered VR-AutoSwapAudio with SteamVR auto-launch.");
                    0
                } else {
                    platform::println("ERROR: Failed to write SteamVR manifest/config files.");
                    1
                }
            } else {
                platform::println("ERROR: Could not locate openvrpaths.vrpath.");
                1
            }
        }
        RunMode::Unregister => {
            if let Some(ref p) = paths {
                if unregister_autolaunch(p) {
                    platform::println("SUCCESS: Unregistered VR-AutoSwapAudio from SteamVR auto-launch.");
                    0
                } else {
                    platform::println("ERROR: Failed to update SteamVR appconfig.json.");
                    1
                }
            } else {
                platform::println("ERROR: Could not locate openvrpaths.vrpath.");
                1
            }
        }
        RunMode::SteamVrLaunched => {
            platform::run_event_driven_vr_session(
                paths.as_ref(),
                profile,
                custom_out.as_bytes(),
                custom_in.as_bytes(),
            );
            0
        }
        RunMode::Auto => {
            let (vrserver_pid, _) = platform::find_process_pids();
            if vrserver_pid != 0 {
                platform::println("SteamVR is currently running -> starting event-driven audio session...");
                platform::run_event_driven_vr_session(
                    paths.as_ref(),
                    profile,
                    custom_out.as_bytes(),
                    custom_in.as_bytes(),
                );
                0
            } else {
                platform::println("SteamVR is not currently running.");
                platform::ensure_desktop_audio(
                    &profile,
                    custom_out.as_bytes(),
                    custom_in.as_bytes(),
                );
                if let Some(ref p) = paths {
                    if register_autolaunch(p) {
                        platform::println(
                            "Auto-Launch registered with SteamVR! VR-AutoSwapAudio will automatically run whenever SteamVR starts.",
                        );
                    }
                }
                platform::println("");
                platform::print_status(
                    paths.as_ref(),
                    &profile,
                    custom_out.as_bytes(),
                    custom_in.as_bytes(),
                );
                0
            }
        }
    }
}
