# VR-AutoSwapAudio

A tiny (~33 KB), cross-platform (**Windows & Linux `x86_64`**), **`#![no_std]` + `#![no_main]`** Rust utility with **zero third-party Rust crate dependencies** that automatically switches both your **OS default audio** and **Discord voice/microphone audio** to your VR headset when SteamVR runs—and restores your pre-VR desktop audio defaults when SteamVR closes.

---

## Why This Exists

When SteamVR starts (or when a wireless/USB headset like a Pico, Quest, Index, or Vive connects):
- **Windows**: SteamVR only switches the `eConsole` and `eMultimedia` audio roles, frequently ignoring or failing to restore the **`eCommunications`** role that **Discord** uses for voice chat and microphone input—leaving Discord stuck on your desktop speakers/mic during VR, or stuck on your disconnected VR headset after quitting SteamVR.
- **Linux**: PipeWire / PulseAudio may not move already-open Discord WebRTC playback (`sink-input`) and capture (`source-output`) streams when the default sink/source changes.
- **Unpredictable Device Names**: Many VR headsets expose generic USB audio names (e.g., `Altavoces (AC Interface)` and `Capture Input terminal (AC Interface)` on Pico headsets) that don't match the headset's brand name.

`VR-AutoSwapAudio` addresses all three issues using native OS event notifications rather than background polling loops.

---

## Key Features

- **Zero Background Process When Not in VR**: Registers as a native **SteamVR Auto-Launch Application** (`.vrmanifest`). It does not run at all while SteamVR is closed; SteamVR launches it automatically on startup, and it exits as soon as SteamVR shuts down and your desktop audio is restored.
- **Event-Driven Architecture (No Polling Loops)**:
  - **Windows (`x86_64-pc-windows-msvc`)**: Uses a COM `IMMNotificationClient` vtable (`OnDefaultDeviceChanged`, `OnDeviceStateChanged`, `OnDeviceAdded`, `OnDeviceRemoved`) combined with kernel handle synchronization (`WaitForMultipleObjects(..., INFINITE)` on `vrserver.exe` and the notification event). Uses negligible idle CPU while waiting, and snapshots/restores `eConsole`, `eMultimedia`, and `eCommunications` roles independently.
  - **Linux (`x86_64-unknown-linux-gnu`, kernel 5.3+)**: Snapshots the user's original default sink/source (`pactl get-default-sink` / `get-default-source`), then uses kernel `pidfd_open` on `vrserver` multiplexed with `pactl subscribe` via `poll(..., -1)` to switch default sinks/sources and move live Discord/WebRTC streams (`pactl move-sink-input` / `pactl move-source-output`, or all streams with `--all-streams`).
- **SteamVR Configuration & Log Discovery**: Reads `openvrpaths.vrpath`, `steamvr.vrsettings`, and `vrmonitor.txt` to identify the audio endpoints (including hardware `VID/PID` and parenthesized adapter tags) that SteamVR uses for your HMD, plus heuristic fallbacks and manual `--output`/`--input` overrides.
- **`#![no_std]` & Zero Third-Party Rust Dependencies**:
  - Empty `[dependencies]` in `Cargo.toml` (relies only on native OS interfaces: `kernel32.lib` + `raw-dylib` imports from `combase.dll` and `ntdll.dll` on Windows; `libc` syscalls and `pactl` on Linux).
  - Temporary file buffers (`vrmonitor.txt`, `steamvr.vrsettings`, `appconfig.json`) are allocated via `VirtualAlloc` (`mmap` on Linux) and immediately released back to the OS (`VirtualFree(..., MEM_RELEASE)` / `munmap`) before entering the event wait.

---

## Measured Memory Footprint & Platform Compatibility

### Windows Memory Breakdown (`x86_64-pc-windows-msvc`, Windows 10/11 x64)
Measured across the 5-stage isolation benchmark (`VR-AutoSwapAudio.exe --bench-stage 1..5`) after startup settles on `WaitForSingleObject(..., INFINITE)`:

| Stage | Initialized Components | Private Working Set | Private Commit | Total Working Set (incl. shared OS DLLs) | Threads | Loaded Modules |
| :---: | :--- | :---: | :---: | :---: | :---: | :---: |
| **1** | Bare process (`mainCRTStartup` + `WaitForSingleObject`) | `~408 KB` | `~640 KB` | `~4,636 KB` | `3` | `7` |
| **2** | + COM runtime (`CoInitializeEx`) | `~532 KB` | `~788 KB` | `~5,316 KB` | `3` | `10` |
| **3** | + Core Audio (`IMMDeviceEnumerator`) | `~712 KB` | `~1,020 KB` | `~7,144 KB` | `4` | `15` |
| **4** | + Endpoint notifications (`IMMNotificationClient`) | `~776 KB` | `~1,104 KB` | `~7,524 KB` | `5` | `16` |
| **5** | Full initialization (SteamVR discovery + `RoleSnapshots` + `IPolicyConfig`) | **`~816 KB`** | **`~1,152 KB`** | **`~7,972 KB`** | `5` | `16` |

*(During an active SteamVR session with live endpoint switches, Private Commit typically settles between `1.15 MB` and `1.6 MB` depending on Windows audio driver/COM state, with `~97%` attributable to the baseline Windows process and COM/Core Audio runtime.)*

### Supported Platforms & Compatibility Status

| Category | Verified / Tested | Notes & Requirements |
| :--- | :--- | :--- |
| **Windows** | Windows 10 / 11 (`x86_64-pc-windows-msvc`) | Requires MSVC linker (`link.exe`). Uses Windows Core Audio (`MMDevApi.dll` / `combase.dll`). |
| **Linux** | `x86_64-unknown-linux-gnu` (build & static check verified) | Requires **Linux kernel 5.3+** (`pidfd_open` syscall) and `pactl` (`pipewire-pulse` or `pulseaudio-utils`). |
| **VR Headsets / Streaming** | **Pico Neo 3** (USB / Pico Link `AC Interface`) | Also includes heuristic/settings matching for Valve Index, HTC Vive, Meta Quest / Link, Virtual Desktop, Steam Link, Bigscreen Beyond, and PSVR2 (use `--output` / `--input` if your streaming driver exposes a custom device name). |

---

## Step-by-Step Setup Guide

### Step 1: Build or Download the Binary

#### Prerequisites
- **Rust toolchain** (`rustup` / `cargo`)
- **Windows**: Visual Studio C++ Build Tools (MSVC linker, standard for Rust on Windows)
- **Linux**: `gcc`/`clang` and `pactl` (`pulseaudio-utils` or `pipewire-pulse`, installed by default on most desktop distros)

#### Build Release Executable
```bash
git clone https://github.com/SymphonyVR/VR-AutoSwapAudio.git
cd VR-AutoSwapAudio
cargo build --release
```

Your compiled executable will be at:
- **Windows**: `target\release\VR-AutoSwapAudio.exe`
- **Linux**: `target/release/VR-AutoSwapAudio`

---

### Step 2: Place the Binary in a Permanent Folder (Optional)

Because SteamVR stores the absolute path to `vr-autoswapaudio.vrmanifest` inside Steam's `config/appconfig.json`, put `VR-AutoSwapAudio.exe` (or `VR-AutoSwapAudio` on Linux) in the directory where you intend to keep it (or leave it in `target\release\`).

---

### Step 3: Register Auto-Launch with SteamVR

With **SteamVR closed**, run the executable once (double-click it or run it in a terminal):

**Windows (PowerShell / CMD):**
```powershell
.\target\release\VR-AutoSwapAudio.exe
```

**Linux:**
```bash
./target/release/VR-AutoSwapAudio
```

What this single run does automatically:
1. Reads `%LOCALAPPDATA%\openvr\openvrpaths.vrpath` (Windows) or `~/.config/openvr/openvrpaths.vrpath` (Linux) to locate your Steam `config` and `logs` directories.
2. Creates `vr-autoswapaudio.vrmanifest` right next to the executable.
3. Registers `vr-autoswapaudio.vrmanifest` inside `<Steam>/config/appconfig.json` and writes `<Steam>/config/vrappconfig/rust.vr.autoswapaudio.vrappconfig` with `"autolaunch": true`.
4. Immediately checks your active OS audio devices and repairs any `[Discord/Comm]` or `[Default]` role that was left stuck on your VR headset from an earlier session.

---

### Step 4: Configure Discord Voice & Video Settings

1. Open **Discord** → **User Settings (⚙️)** → **Voice & Video**.
2. Set **Input Device** to **Default**.
3. Set **Output Device** to **Default**.

Why?
- On **Windows**, Discord's **Default** device follows the Windows **`eCommunications`** audio role. `VR-AutoSwapAudio` switches `eConsole`, `eMultimedia`, **and `eCommunications`** together in real time when your VR headset connects, and restores all three roles back to your desktop speakers and microphone when SteamVR closes.
- On **Linux**, `VR-AutoSwapAudio` switches the default PipeWire/PulseAudio sink & source *and* actively moves live playback/recording streams (`pactl move-sink-input` / `pactl move-source-output`).

---

### Step 5: Verify inside SteamVR (Optional)

1. Launch **SteamVR**.
2. Open **SteamVR Settings** → **Startup / Shutdown** → **Choose Startup Overlay Apps**.
3. Confirm **VR-AutoSwapAudio** is listed and toggled **ON**.
4. You can also run `--status` in a terminal at any time (while SteamVR is running or closed) to inspect the detected devices:
   ```powershell
   .\target\release\VR-AutoSwapAudio.exe --status
   ```

---

## Custom Device Overrides & Uninstalling

### Using a Custom Microphone, Headphones, or `--all-streams` (Linux)
By default, `VR-AutoSwapAudio` uses SteamVR's own HMD audio detection (`steamvr.vrsettings` + `vrmonitor.txt` hardware/adapter matching), and on Linux it only moves Discord / WebRTC voice streams (`discord`, `vesktop`, `webcord`, `legcord`, `armcord`, `WEBRTC VoiceEngine`).
- If you select a manual playback or recording device inside **SteamVR Settings → Audio**, `VR-AutoSwapAudio` automatically reads that override from `steamvr.vrsettings`!
- To make custom flags (`--output "<substring>"`, `--input "<substring>"`, or Linux `--all-streams`) persist when SteamVR auto-launches the utility, edit the `"arguments"` field inside `vr-autoswapaudio.vrmanifest` (located next to the binary):
  ```json
  "arguments": "--steamvr-launched --output \"Custom HMD\" --input \"ModMic\" --all-streams",
  ```
  *(Note: Re-running `--register` overwrites `vr-autoswapaudio.vrmanifest` with the default `"--steamvr-launched"` arguments, so apply any custom `.vrmanifest` edits after registering.)*

### Moving or Uninstalling
- **If you move the binary to a new folder**: Run `VR-AutoSwapAudio.exe --register` from the new folder.
- **To disable auto-launch and remove from SteamVR**:
  ```powershell
  .\target\release\VR-AutoSwapAudio.exe --unregister
  ```

---

## Troubleshooting & Diagnostics

- **Headset audio device is not detected (`[Desktop]` shown instead of `[VR-HMD]`)**:
  Run `VR-AutoSwapAudio --status` while your headset is connected and SteamVR is running. If your headset or wireless streaming driver uses an unrecognized audio endpoint name, either select it once in **SteamVR Settings → Audio** (which writes the override into `steamvr.vrsettings`) or pass `--output "<name>"` / `--input "<name>"` in `vr-autoswapaudio.vrmanifest`.When reporting unexpected device selection in an issue, please include the full output of `VR-AutoSwapAudio --status`.
- **Discord does not switch output or microphone**:
  Verify that **Discord → User Settings → Voice & Video → Input Device / Output Device** are both set to **Default** (so Discord follows the Windows `eCommunications` role or the PipeWire/PulseAudio stream move).
- **Audio stuck on the VR endpoint after a force-kill or crash**:
  Run `VR-AutoSwapAudio` once with SteamVR closed; it will immediately restore any `[Default]` or `[Discord/Comm]` role that is still pointing at a VR device back to your desktop endpoint.
- **Running built-in self-tests**:
  Run `VR-AutoSwapAudio --self-test` to verify the structural `appconfig.json` updater and Windows UTF-16 command-line parser against edge cases (escaped quotes, missing keys, malformed JSON, and buffer limits).

---

## CLI Reference

```text
Usage:
  VR-AutoSwapAudio [OPTIONS]

Options:
  --status, --list       Show SteamVR HMD & active OS audio endpoints
  --register             Register auto-launch .vrmanifest with SteamVR
  --unregister           Remove auto-launch .vrmanifest from SteamVR
  --steamvr-launched     Run event-driven VR audio session (used by SteamVR)
  --output <pattern>     Optional custom VR output device substring override
  --input <pattern>      Optional custom VR input device substring override
  --all-streams          (Linux) Move all active audio streams, not just Discord
  --self-test            Run built-in parser and configuration self-tests
  --bench-stage <1..5>   (Windows) Run read-only memory isolation stage 1..5
  --help, -h             Show this help message
```

### Example `--status` Output

```text
SteamVR Running: NO (vrserver PID: 0, vrmonitor PID: 0)
SteamVR Config : E:\Program Files (x86)\Steam\config
SteamVR Logs   : E:\Program Files (x86)\Steam\logs
Detected HMD   : Pico Pico Neo 3 (Adapter: 'AC Interface', USB: 'VID_2D40/PID_0016')
SteamVR Profile VR Output     : Altavoces (AC Interface) [{0.0.0.00000000}.{4736806b-e54c-4fea-a685-4a3633efa0ab}]
SteamVR Profile VR Input      : Capture Input terminal (AC Interface) [{0.0.1.00000000}.{b85356bd-933e-425e-b725-218d077ff131}]
SteamVR Profile Desktop Output: Altavoces (Realtek(R) Audio) [{0.0.0.00000000}.{d0e38a83-771b-4da0-80a5-fa925169ca15}]
SteamVR Profile Desktop Input : Micrófono (2- USB PnP Audio Device) [{0.0.1.00000000}.{59ca05da-3264-48de-b064-0d950a911b15}]

--- Active Windows Playback (Output) Endpoints ---
  [VR-HMD] Altavoces (AC Interface) [{0.0.0.00000000}.{4736806b-e54c-4fea-a685-4a3633efa0ab}]
  [Default] [Discord/Comm] [Desktop] Altavoces (Realtek(R) Audio) [{0.0.0.00000000}.{d0e38a83-771b-4da0-80a5-fa925169ca15}]

--- Active Windows Recording (Input) Endpoints ---
  [Default] [Discord/Comm] [Desktop] Micrófono (2- USB PnP Audio Device) [{0.0.1.00000000}.{59ca05da-3264-48de-b064-0d950a911b15}]
  [VR-HMD] Capture Input terminal (AC Interface) [{0.0.1.00000000}.{b85356bd-933e-425e-b725-218d077ff131}]
```
