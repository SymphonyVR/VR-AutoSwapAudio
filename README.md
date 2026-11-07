# VR-AutoSwapAudio

A tiny (~30 KB), cross-platform (**Windows & Linux**), **`#![no_std]` + `#![no_main]`** Rust utility with **zero external crate dependencies** that automatically switches both your **OS default audio** and **Discord voice/microphone audio** to your VR headset when SteamVR runs—and restores your desktop audio when SteamVR closes.

---

## Why This Exists

When SteamVR starts (or when a wireless/USB headset like a Pico, Quest, Index, or Vive connects):
- **Windows**: SteamVR only switches the `eConsole` and `eMultimedia` audio roles, frequently ignoring or failing to restore the **`eCommunications`** role that **Discord** uses for voice chat and microphone input—leaving Discord stuck on your desktop speakers/mic during VR, or stuck on your disconnected VR headset after quitting SteamVR.
- **Linux**: PipeWire / PulseAudio may not move already-open Discord WebRTC playback (`sink-input`) and capture (`source-output`) streams when the default sink/source changes.
- **Unpredictable Device Names**: Many VR headsets expose generic USB audio names (e.g., `Altavoces (AC Interface)` and `Capture Input terminal (AC Interface)` on Pico headsets) that don't match the headset's brand name.

`VR-AutoSwapAudio` solves all three problems with **zero background overhead**.

---

## Key Features

- **Zero Background Process When Not in VR**: Registers as a native **SteamVR Auto-Launch Application** (`.vrmanifest`). It does not run at all while SteamVR is closed; SteamVR launches it automatically on startup, and it exits as soon as SteamVR shuts down and your desktop audio is restored.
- **100% Event-Driven (Zero Polling)**:
  - **Windows**: Uses a custom COM `IMMNotificationClient` vtable (`OnDefaultDeviceChanged` / `OnDeviceStateChanged`) combined with kernel handle synchronization (`WaitForMultipleObjects(..., INFINITE)` on `vrserver.exe` and `vrmonitor.exe`). Consumes **0% CPU** with zero sleep/polling loops, and snapshots/restores `eConsole`, `eMultimedia`, and `eCommunications` roles independently.
  - **Linux**: Uses kernel `pidfd_open` on `vrserver` multiplexed with `pactl subscribe` via `poll(..., -1)` (infinite kernel wait) to switch default sinks/sources and move live Discord/WebRTC streams (`pactl move-sink-input` / `pactl move-source-output`, or all streams with `--all-streams`).
- **Native SteamVR Device Discovery**: Reads `openvrpaths.vrpath`, `steamvr.vrsettings`, and `vrmonitor.txt` to discover the exact audio endpoints (including hardware `VID/PID` and adapter tags) that SteamVR uses for your HMD, plus your non-VR desktop baseline devices.
- **`#![no_std]` & Zero Dependencies**:
  - Empty `[dependencies]` in `Cargo.toml`.
  - Links with `/NODEFAULTLIB` on Windows (`kernel32.lib` + `onecoreuap.lib`, without `Shell32.dll` or `ole32.dll` startup imports) and releases temporary file buffers back to the OS via `VirtualAlloc`/`VirtualFree` (`mmap`/`munmap` on Linux).
  - ~33 KB stripped release executable on Windows (~1.3 MB private commit / ~8.9 MB working set when active in SteamVR).

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

### Using a Custom Microphone or Headphones in VR
By default, `VR-AutoSwapAudio` uses SteamVR's own HMD audio detection (`steamvr.vrsettings` + `vrmonitor.txt` hardware/adapter matching).
- If you select a manual playback or recording device inside **SteamVR Settings → Audio**, `VR-AutoSwapAudio` automatically reads that override from `steamvr.vrsettings`!
- Alternatively, you can pass `--output "<substring>"` or `--input "<substring>"` in `vr-autoswapaudio.vrmanifest`'s `"arguments"` field (e.g. `"--steamvr-launched --input \"ModMic\""`).

### Moving or Uninstalling
- **If you move the binary to a new folder**: Run `VR-AutoSwapAudio.exe --register` from the new folder.
- **To disable auto-launch and remove from SteamVR**:
  ```powershell
  .\target\release\VR-AutoSwapAudio.exe --unregister
  ```

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
