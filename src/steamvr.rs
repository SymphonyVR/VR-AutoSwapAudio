use core::fmt::Write;
use crate::str_buf::{
    FixedBuf, contains_ignore_ascii_case, eq_ignore_ascii_case, find_subslice,
    json_extract_first_array_string, json_extract_string_value, rfind_subslice, trim_ascii,
};
use crate::platform;

pub const APP_KEY: &str = "rust.vr.autoswapaudio";
pub const MANIFEST_FILENAME: &str = "vr-autoswapaudio.vrmanifest";

pub struct SteamVrPaths {
    pub config_dir: FixedBuf<512>,
    pub log_dir: FixedBuf<512>,
    pub runtime_dir: FixedBuf<512>,
}

impl SteamVrPaths {
    pub const fn new() -> Self {
        Self {
            config_dir: FixedBuf::new(),
            log_dir: FixedBuf::new(),
            runtime_dir: FixedBuf::new(),
        }
    }

    pub fn discover() -> Option<Self> {
        let mut vrpath_file = FixedBuf::<512>::new();
        if !platform::get_openvrpaths_file(&mut vrpath_file) {
            return None;
        }
        let mut file_buf = [0u8; 4096];
        let n = platform::read_file_bytes(vrpath_file.as_bytes(), &mut file_buf);
        if n == 0 {
            return None;
        }
        let json = &file_buf[..n];
        let mut paths = Self::new();
        paths.config_dir.len = json_extract_first_array_string(json, b"config", &mut paths.config_dir.buf);
        paths.log_dir.len = json_extract_first_array_string(json, b"log", &mut paths.log_dir.buf);
        paths.runtime_dir.len = json_extract_first_array_string(json, b"runtime", &mut paths.runtime_dir.buf);

        if paths.config_dir.is_empty() {
            None
        } else {
            Some(paths)
        }
    }
}

pub struct VrAudioProfile {
    pub hmd_manufacturer: FixedBuf<128>,
    pub hmd_model: FixedBuf<128>,
    /// Adapter/interface name extracted from SteamVR override or HMD audio (e.g. "AC Interface")
    pub vr_adapter_tag: FixedBuf<128>,
    /// USB VID/PID tag if discovered in vrmonitor.txt (e.g. "VID_2D40/PID_0016")
    pub vr_vid_pid: FixedBuf<64>,
    /// Explicit VR Playback endpoint ID (if known from settings or vrmonitor.txt)
    pub vr_playback_id: FixedBuf<160>,
    pub vr_playback_name: FixedBuf<160>,
    /// Explicit VR Recording endpoint ID (if known from settings or vrmonitor.txt)
    pub vr_record_id: FixedBuf<160>,
    pub vr_record_name: FixedBuf<160>,
    /// Desktop Playback endpoint ID (non-VR baseline)
    pub desktop_playback_id: FixedBuf<160>,
    pub desktop_playback_name: FixedBuf<160>,
    /// Desktop Recording endpoint ID (non-VR baseline)
    pub desktop_record_id: FixedBuf<160>,
    pub desktop_record_name: FixedBuf<160>,
}

impl VrAudioProfile {
    pub const fn new() -> Self {
        Self {
            hmd_manufacturer: FixedBuf::new(),
            hmd_model: FixedBuf::new(),
            vr_adapter_tag: FixedBuf::new(),
            vr_vid_pid: FixedBuf::new(),
            vr_playback_id: FixedBuf::new(),
            vr_playback_name: FixedBuf::new(),
            vr_record_id: FixedBuf::new(),
            vr_record_name: FixedBuf::new(),
            desktop_playback_id: FixedBuf::new(),
            desktop_playback_name: FixedBuf::new(),
            desktop_record_id: FixedBuf::new(),
            desktop_record_name: FixedBuf::new(),
        }
    }

    pub fn load(paths: &SteamVrPaths) -> Self {
        let mut profile = Self::new();
        profile.load_vrsettings(paths);
        profile.load_vrmonitor_log(paths);
        profile
    }

    fn load_vrsettings(&mut self, paths: &SteamVrPaths) {
        let mut settings_path = FixedBuf::<512>::new();
        let sep = platform::PATH_SEP;
        let _ = write!(settings_path, "{}{}steamvr.vrsettings", paths.config_dir.as_str(), sep);

        let mut buf = [0u8; 16384];
        let n = platform::read_file_bytes(settings_path.as_bytes(), &mut buf);
        if n == 0 {
            return;
        }
        let json = &buf[..n];

        self.hmd_manufacturer.len =
            json_extract_string_value(json, b"HMDManufacturer", &mut self.hmd_manufacturer.buf);
        self.hmd_model.len =
            json_extract_string_value(json, b"HMDModel", &mut self.hmd_model.buf);

        if !self.hmd_manufacturer.is_empty() && !self.hmd_model.is_empty() {
            let mut key = FixedBuf::<256>::new();

            // Playback override ID
            let _ = write!(
                key,
                "playbackDeviceOverride_{}_{}",
                self.hmd_manufacturer.as_str(),
                self.hmd_model.as_str()
            );
            self.vr_playback_id.len =
                json_extract_string_value(json, key.as_bytes(), &mut self.vr_playback_id.buf);

            // Playback override Name
            key.clear();
            let _ = write!(
                key,
                "playbackDeviceOverrideName_{}_{}",
                self.hmd_manufacturer.as_str(),
                self.hmd_model.as_str()
            );
            self.vr_playback_name.len =
                json_extract_string_value(json, key.as_bytes(), &mut self.vr_playback_name.buf);

            // Recording override ID
            key.clear();
            let _ = write!(
                key,
                "recordingDeviceOverride_{}_{}",
                self.hmd_manufacturer.as_str(),
                self.hmd_model.as_str()
            );
            self.vr_record_id.len =
                json_extract_string_value(json, key.as_bytes(), &mut self.vr_record_id.buf);

            // Recording override Name
            key.clear();
            let _ = write!(
                key,
                "recordingDeviceOverrideName_{}_{}",
                self.hmd_manufacturer.as_str(),
                self.hmd_model.as_str()
            );
            self.vr_record_name.len =
                json_extract_string_value(json, key.as_bytes(), &mut self.vr_record_name.buf);
        }

        // Extract parenthesized adapter tag from override name, e.g.
        // "Capture Input terminal (AC Interface)" -> "AC Interface"
        if !self.vr_record_name.is_empty() {
            extract_parenthesized_tag(self.vr_record_name.as_bytes(), &mut self.vr_adapter_tag);
        }
        if self.vr_adapter_tag.is_empty() && !self.vr_playback_name.is_empty() {
            extract_parenthesized_tag(self.vr_playback_name.as_bytes(), &mut self.vr_adapter_tag);
        }
    }

    fn load_vrmonitor_log(&mut self, paths: &SteamVrPaths) {
        if paths.log_dir.is_empty() {
            return;
        }
        let mut log_path = FixedBuf::<512>::new();
        let sep = platform::PATH_SEP;
        let _ = write!(log_path, "{}{}vrmonitor.txt", paths.log_dir.as_str(), sep);

        // Read up to the last 32 KB of vrmonitor.txt so we get the latest session's audio dump
        let mut buf = [0u8; 32768];
        let n = platform::read_file_tail_bytes(log_path.as_bytes(), &mut buf);
        if n == 0 {
            return;
        }
        let log = &buf[..n];

        // Scan lines in vrmonitor.txt:
        // Format of device lines logged by vrmonitor:
        // "\t(Comm)->Altavoces (AC Interface),0,Altavoces,AC Interface,VID_2D40/PID_0016,USB\...,{0.0.0.00000000}.{4736806b-...}"
        // "\t-->Altavoces (Realtek(R) Audio),0,Altavoces,Realtek(R) Audio,VID_0000/PID_0000,HDAUDIO\...,{0.0.0.00000000}.{d0e38a83-...}"
        // And transition lines:
        // "AUDIO: OnDefaultDeviceChanged: flow = eRender, role = eConsole, device = {0.0.0.00000000}.{4736806b-...}"
        let mut line_start = 0;
        while line_start < log.len() {
            let mut line_end = line_start;
            while line_end < log.len() && log[line_end] != b'\n' {
                line_end += 1;
            }
            let line = trim_ascii(&log[line_start..line_end]);
            line_start = line_end + 1;

            // Check if this is an enumerated endpoint line containing ",VID_" and ",{0.0."
            if let Some(guid_pos) = rfind_subslice(line, b",{0.0.") {
                let guid = &line[guid_pos + 1..];
                let is_render = guid.starts_with(b"{0.0.0.");
                let is_capture = guid.starts_with(b"{0.0.1.");

                // Find the start of the device description after "- \t" or "->"
                let desc_part = if let Some(arrow) = find_subslice(line, b"->") {
                    &line[arrow + 2..guid_pos]
                } else if let Some(tab) = find_subslice(line, b"- \t") {
                    &line[tab + 3..guid_pos]
                } else {
                    &line[..guid_pos]
                };

                // First comma separates full friendly name from ",0,Endpoint,Adapter,VID_..."
                let friendly_name = if let Some(comma) = find_subslice(desc_part, b",0,") {
                    &desc_part[..comma]
                } else {
                    desc_part
                };

                let is_vr_line = self.matches_vr_name_or_id(friendly_name, guid, desc_part);
                if is_vr_line {
                    // Extract VID_xxxx/PID_xxxx if present
                    if self.vr_vid_pid.is_empty() {
                        if let Some(vid_idx) = find_subslice(desc_part, b"VID_") {
                            let rest = &desc_part[vid_idx..];
                            if let Some(c_idx) = find_subslice(rest, b",") {
                                self.vr_vid_pid.set_bytes(&rest[..c_idx]);
                            }
                        }
                    }
                    if is_render {
                        self.vr_playback_id.set_bytes(guid);
                        self.vr_playback_name.set_bytes(friendly_name);
                    } else if is_capture {
                        self.vr_record_id.set_bytes(guid);
                        self.vr_record_name.set_bytes(friendly_name);
                    }
                } else {
                    // Non-VR device logged in vrmonitor's default audio block -> Desktop baseline!
                    if is_render {
                        self.desktop_playback_id.set_bytes(guid);
                        self.desktop_playback_name.set_bytes(friendly_name);
                    } else if is_capture {
                        self.desktop_record_id.set_bytes(guid);
                        self.desktop_record_name.set_bytes(friendly_name);
                    }
                }
            }
        }
    }

    pub fn matches_vr_name_or_id(&self, friendly_name: &[u8], id: &[u8], extra_info: &[u8]) -> bool {
        if !self.vr_playback_id.is_empty() && eq_ignore_ascii_case(id, self.vr_playback_id.as_bytes()) {
            return true;
        }
        if !self.vr_record_id.is_empty() && eq_ignore_ascii_case(id, self.vr_record_id.as_bytes()) {
            return true;
        }
        if !self.vr_adapter_tag.is_empty()
            && contains_ignore_ascii_case(friendly_name, self.vr_adapter_tag.as_bytes())
        {
            return true;
        }
        if !self.vr_vid_pid.is_empty()
            && !eq_ignore_ascii_case(self.vr_vid_pid.as_bytes(), b"VID_0000/PID_0000")
            && contains_ignore_ascii_case(extra_info, self.vr_vid_pid.as_bytes())
        {
            return true;
        }
        if !self.hmd_manufacturer.is_empty()
            && contains_ignore_ascii_case(friendly_name, self.hmd_manufacturer.as_bytes())
        {
            return true;
        }
        if !self.hmd_model.is_empty()
            && contains_ignore_ascii_case(friendly_name, self.hmd_model.as_bytes())
        {
            return true;
        }
        // Standard VR audio adapter identifiers (Pico AC Interface, Valve Index, Vive, Oculus, Virtual Desktop, Steam Streaming)
        const VR_PATTERNS: &[&[u8]] = &[
            b"AC Interface",
            b"Index HMD",
            b"Valve VR",
            b"VIVE",
            b"Oculus Virtual Audio",
            b"Quest",
            b"Virtual Desktop Audio",
            b"Steam Streaming",
            b"Pico",
            b"Bigscreen Beyond",
            b"PSVR",
        ];
        let mut i = 0;
        while i < VR_PATTERNS.len() {
            if contains_ignore_ascii_case(friendly_name, VR_PATTERNS[i]) {
                return true;
            }
            i += 1;
        }
        false
    }
}

fn extract_parenthesized_tag(name: &[u8], out: &mut FixedBuf<128>) {
    if let Some(open) = rfind_subslice(name, b"(") {
        let rest = &name[open + 1..];
        if let Some(close) = find_subslice(rest, b")") {
            let tag = trim_ascii(&rest[..close]);
            if !tag.is_empty() {
                out.set_bytes(tag);
            }
        }
    }
}

fn append_json_escaped_path(out: &mut FixedBuf<4096>, path: &[u8]) {
    let mut i = 0;
    while i < path.len() {
        if path[i] == b'\\' {
            out.push_bytes(b"\\\\");
        } else if path[i] == b'"' {
            out.push_bytes(b"\\\"");
        } else {
            out.push_byte(path[i]);
        }
        i += 1;
    }
}

/// Generates `vr-autoswapaudio.vrmanifest` next to the executable and registers it in
/// `<Steam/config>/appconfig.json` and `<Steam/config>/vrappconfig/rust.vr.autoswapaudio.vrappconfig`.
pub fn register_autolaunch(paths: &SteamVrPaths) -> bool {
    let mut exe_dir = FixedBuf::<512>::new();
    if !platform::get_exe_dir(&mut exe_dir) {
        return false;
    }

    let sep = platform::PATH_SEP;
    let mut manifest_path = FixedBuf::<512>::new();
    let _ = write!(manifest_path, "{}{}{}", exe_dir.as_str(), sep, MANIFEST_FILENAME);

    let mut manifest_json = FixedBuf::<2048>::new();
    let _ = write!(
        manifest_json,
        "{{\n    \"source\" : \"builtin\",\n    \"applications\" : [{{\n        \"app_key\" : \"{}\",\n        \"launch_type\" : \"binary\",\n        \"binary_path_windows\" : \"VR-AutoSwapAudio.exe\",\n        \"binary_path_linux\" : \"VR-AutoSwapAudio\",\n        \"arguments\" : \"--steamvr-launched\",\n        \"is_dashboard_overlay\" : true,\n        \"strings\" : {{\n            \"en_us\" : {{\n                \"name\" : \"VR-AutoSwapAudio\",\n                \"description\" : \"Event-driven OS & Discord VR audio auto-switcher\"\n            }}\n        }}\n    }}]\n}}\n",
        APP_KEY
    );

    if !platform::write_file_bytes(manifest_path.as_bytes(), manifest_json.as_bytes()) {
        return false;
    }

    // 1. Write <config_dir>/vrappconfig/rust.vr.autoswapaudio.vrappconfig
    let mut vrappconfig_path = FixedBuf::<512>::new();
    let _ = write!(
        vrappconfig_path,
        "{}{}vrappconfig{}{}.vrappconfig",
        paths.config_dir.as_str(),
        sep,
        sep,
        APP_KEY
    );
    let vrappconfig_json = b"{\n   \"autolaunch\" : true,\n   \"last_launch_time\" : \"0\"\n}\n";
    let _ = platform::write_file_bytes(vrappconfig_path.as_bytes(), vrappconfig_json);

    // 2. Ensure manifest_path is listed in <config_dir>/appconfig.json
    let mut appconfig_path = FixedBuf::<512>::new();
    let _ = write!(appconfig_path, "{}{}appconfig.json", paths.config_dir.as_str(), sep);

    let mut existing_buf = [0u8; 4096];
    let n = platform::read_file_bytes(appconfig_path.as_bytes(), &mut existing_buf);
    let existing = &existing_buf[..n];

    if contains_ignore_ascii_case(existing, MANIFEST_FILENAME.as_bytes()) {
        return true;
    }

    let mut new_appconfig = FixedBuf::<4096>::new();
    if let Some(close_bracket) = rfind_subslice(existing, b"]") {
        // Check if there is already at least one quoted string before `]`
        let before = trim_ascii(&existing[..close_bracket]);
        let has_entries = before.ends_with(b"\"");
        new_appconfig.push_bytes(before);
        if has_entries {
            new_appconfig.push_bytes(b",\n      \"");
        } else {
            new_appconfig.push_bytes(b"\n      \"");
        }
        append_json_escaped_path(&mut new_appconfig, manifest_path.as_bytes());
        new_appconfig.push_bytes(b"\"\n   ]\n}\n");
    } else {
        new_appconfig.push_bytes(b"{\n   \"manifest_paths\" : [\n      \"");
        append_json_escaped_path(&mut new_appconfig, manifest_path.as_bytes());
        new_appconfig.push_bytes(b"\"\n   ]\n}\n");
    }

    platform::write_file_bytes(appconfig_path.as_bytes(), new_appconfig.as_bytes())
}

/// Removes `vr-autoswapaudio.vrmanifest` from SteamVR's `appconfig.json` and disables autolaunch.
pub fn unregister_autolaunch(paths: &SteamVrPaths) -> bool {
    let sep = platform::PATH_SEP;
    let mut vrappconfig_path = FixedBuf::<512>::new();
    let _ = write!(
        vrappconfig_path,
        "{}{}vrappconfig{}{}.vrappconfig",
        paths.config_dir.as_str(),
        sep,
        sep,
        APP_KEY
    );
    let vrappconfig_json = b"{\n   \"autolaunch\" : false,\n   \"last_launch_time\" : \"0\"\n}\n";
    let _ = platform::write_file_bytes(vrappconfig_path.as_bytes(), vrappconfig_json);

    let mut appconfig_path = FixedBuf::<512>::new();
    let _ = write!(appconfig_path, "{}{}appconfig.json", paths.config_dir.as_str(), sep);

    let mut existing_buf = [0u8; 4096];
    let n = platform::read_file_bytes(appconfig_path.as_bytes(), &mut existing_buf);
    if n == 0 {
        return true;
    }
    let existing = &existing_buf[..n];
    if !contains_ignore_ascii_case(existing, MANIFEST_FILENAME.as_bytes()) {
        return true;
    }

    // Rebuild appconfig.json omitting the line containing MANIFEST_FILENAME
    let mut out = FixedBuf::<4096>::new();
    out.push_bytes(b"{\n   \"manifest_paths\" : [\n");
    let mut first = true;
    let mut line_start = 0;
    while line_start < existing.len() {
        let mut line_end = line_start;
        while line_end < existing.len() && existing[line_end] != b'\n' {
            line_end += 1;
        }
        let line = trim_ascii(&existing[line_start..line_end]);
        line_start = line_end + 1;

        if line.starts_with(b"\"")
            && !contains_ignore_ascii_case(line, b"manifest_paths")
            && !contains_ignore_ascii_case(line, MANIFEST_FILENAME.as_bytes())
        {
            let clean = if line.ends_with(b",") {
                &line[..line.len() - 1]
            } else {
                line
            };
            if !first {
                out.push_bytes(b",\n");
            }
            out.push_bytes(b"      ");
            out.push_bytes(clean);
            first = false;
        }
    }
    out.push_bytes(b"\n   ]\n}\n");
    platform::write_file_bytes(appconfig_path.as_bytes(), out.as_bytes())
}
