// Taixin toolbox: Ethernet OTA (docs/wnb-ota-protocol.md), NETAT console, NETLOG viewer, for TXW8301 bridges.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod fwfile;
mod pcap;
mod proto;
mod udpproto;
mod work;

use eframe::egui;
use proto::{mac_str, Mac, ScanReport};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use work::{Msg, Out};

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Firmware,
    Netat,
    Netlog,
    About,
}

// Same links as the firmware's About page (web/src/views/about.vue).
const GH: &str = "https://github.com/";
const REPO: &str = "aliosa27/taixin-halow-bridge";
const MAIL: &str = "aliosa27@aliosa27.me";
const SOURCES: &[(&str, &str)] = &[
    ("This tool and firmware", REPO),
    ("Vendor SDK", "Taixin-Semiconductor/TXW8301_WNB"),
    ("Vendor web UI", "Taixin-Semiconductor/hgic-wnb-web"),
];
const CREDITS: &[(&str, &str)] = &[
    ("NETAT / NETLOG packet layouts", "aliosa27/taixin_tools"),
    ("OTA flash timing", "I-AM-ENGINEER/RNode_Halow_OTA_Flasher"),
];

/// Version line for the About page and issue reports.
fn version_line() -> String {
    format!(
        "Taixin toolbox {} ({}, built {}) for {} {}",
        env!("CARGO_PKG_VERSION"),
        env!("TOOLBOX_COMMIT"),
        env!("TOOLBOX_BUILD_DATE"),
        std::env::consts::OS,
        std::env::consts::ARCH
    )
}

fn about_tab(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("About");
        ui.label("Where this tool comes from and how to report problems");
        ui.add_space(8.0);

        let card = |ui: &mut egui::Ui, title: &str, body: &mut dyn FnMut(&mut egui::Ui)| {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.strong(title);
                ui.add_space(4.0);
                body(ui);
            });
            ui.add_space(6.0);
        };
        let link = |ui: &mut egui::Ui, label: &str, repo: &str, path: &str| {
            ui.label(egui::RichText::new(label).weak());
            ui.hyperlink_to(format!("{repo}{path}"), format!("{GH}{repo}{path}"));
            ui.add_space(2.0);
        };

        card(ui, "Version", &mut |ui| {
            ui.horizontal(|ui| {
                ui.monospace(version_line());
                if ui.small_button("Copy").clicked() {
                    ui.ctx().copy_text(version_line());
                }
            });
        });
        card(ui, "Source code", &mut |ui| {
            for (k, r) in SOURCES {
                link(ui, k, r, "");
            }
            ui.label(
                egui::RichText::new(
                    "Apache-2.0. The firmware is built from the Taixin SDK with patches; its web UI is derived \
                     from the vendor UI.",
                )
                .small()
                .weak(),
            );
        });
        card(ui, "Credits", &mut |ui| {
            for (k, r) in CREDITS {
                link(ui, k, r, "");
            }
        });
        card(ui, "Updates", &mut |ui| {
            link(ui, "Releases (firmware and this tool)", REPO, "/releases");
            ui.label(
                egui::RichText::new(
                    "Download the firmware .bin and install it on the Firmware (OTA) tab, or on the device's \
                     Firmware page once it runs this firmware.",
                )
                .small()
                .weak(),
            );
        });
        card(ui, "Report an issue", &mut |ui| {
            link(ui, "GitHub", REPO, "/issues");
            ui.label(egui::RichText::new("Email").weak());
            ui.hyperlink_to(MAIL, format!("mailto:{MAIL}"));
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new(
                    "Include the version line above, the device's firmware version from Scan, the log at the \
                     bottom of this window, and the UART log if the device misbehaves at boot.",
                )
                .small()
                .weak(),
            );
        });
    });
}

/// Text kept in the console and log views.
const TEXT_MAX: usize = 200_000;

/// Green for "done"/"live" labels, readable on both the light and the dark theme.
fn ok_color(ui: &egui::Ui) -> egui::Color32 {
    if ui.visuals().dark_mode {
        egui::Color32::from_rgb(0x5c, 0xd0, 0x6e)
    } else {
        egui::Color32::from_rgb(0x2e, 0xa0, 0x43)
    }
}

fn push_capped(buf: &mut String, s: &str) {
    buf.push_str(s);
    if buf.len() > TEXT_MAX {
        let cut = buf.len() - TEXT_MAX;
        let cut = (cut..buf.len()).find(|&i| buf.is_char_boundary(i)).unwrap_or(buf.len());
        buf.drain(..cut);
    }
}

struct App {
    tab: Tab,
    devices: Vec<pcap::Device>,
    selected: usize,
    show_all: bool,
    src: Mac,
    rx: Receiver<Msg>,
    out: Out,
    log: Vec<String>,

    // Firmware
    ota_busy: bool,
    ota_found: Vec<ScanReport>,
    ota_target: Option<usize>,
    fw_path: Option<PathBuf>,
    fw_data: Option<Vec<u8>>,
    fw_info: Result<fwfile::FwInfo, String>,
    reboot_after: bool,
    load_defaults: bool,
    confirm_flash: bool,
    confirm_reset: bool,
    flashing: Option<Arc<AtomicBool>>,
    progress: f32,
    flash_result: Option<Result<(), String>>,

    // NETAT
    at_busy: bool,
    at_found: Vec<(Mac, [u8; 4])>,
    at_target: Option<usize>,
    at_input: String,
    at_history: Vec<String>,
    at_hist_pos: usize,
    at_console: String,

    // NETLOG
    nl_busy: bool,
    nl_found: Vec<(Mac, [u8; 4])>,
    nl_target: Option<usize>,
    nl_stream: Option<Arc<AtomicBool>>,
    nl_text: String,
    nl_popout: bool, // NETLOG shown in its own window, so NETAT can be used beside it
}

impl App {
    fn new(ctx: &egui::Context) -> Self {
        let (tx, rx) = channel();
        let mut app = App {
            tab: Tab::Firmware,
            devices: Vec::new(),
            selected: 0,
            show_all: false,
            src: work::random6(),
            rx,
            out: Out { tx, ui: ctx.clone() },
            log: Vec::new(),
            ota_busy: false,
            ota_found: Vec::new(),
            ota_target: None,
            fw_path: None,
            fw_data: None,
            fw_info: Err(String::new()),
            reboot_after: true,
            load_defaults: false,
            confirm_flash: false,
            confirm_reset: false,
            flashing: None,
            progress: 0.0,
            flash_result: None,
            at_busy: false,
            at_found: Vec::new(),
            at_target: None,
            at_input: String::new(),
            at_history: Vec::new(),
            at_hist_pos: 0,
            at_console: String::new(),
            nl_busy: false,
            nl_found: Vec::new(),
            nl_target: None,
            nl_stream: None,
            nl_text: String::new(),
            nl_popout: false,
        };
        app.refresh();
        app
    }

    fn refresh(&mut self) {
        match pcap::devices() {
            Ok(d) => {
                self.devices = d;
                // Prefer an adapter with an IPv4 address, then any non-loopback one.
                self.selected = self
                    .devices
                    .iter()
                    .position(|d| !d.loopback && d.ipv4.is_some())
                    .or_else(|| self.devices.iter().position(|d| !d.loopback))
                    .unwrap_or(0);
                self.log.push(format!("{} adapter(s) found.", self.devices.len()));
            }
            Err(e) => self.log.push(e),
        }
    }

    fn dev(&self) -> Option<&pcap::Device> {
        self.devices.get(self.selected)
    }

    /// Adapter name and source IPv4 (0.0.0.0 when the adapter has none; the devices still answer
    /// a broadcast).
    fn dev_ip(&self) -> Option<(String, [u8; 4])> {
        self.dev().map(|d| (d.name.clone(), d.ipv4.unwrap_or([0; 4])))
    }

    fn load_fw(&mut self, path: PathBuf) {
        match std::fs::read(&path) {
            Ok(b) => {
                self.fw_info = fwfile::parse(&b);
                self.fw_data = Some(b);
            }
            Err(e) => {
                self.fw_info = Err(e.to_string());
                self.fw_data = None;
            }
        }
        self.fw_path = Some(path);
        self.flash_result = None;
        self.progress = 0.0;
    }

    fn poll(&mut self) {
        while let Ok(m) = self.rx.try_recv() {
            match m {
                Msg::Log(s) => self.log.push(s),
                Msg::OtaFound(r) => {
                    self.ota_found.push(r);
                    if self.ota_found.len() == 1 {
                        self.ota_target = Some(0);
                    }
                }
                Msg::OtaScanDone => self.ota_busy = false,
                Msg::Progress(p) => self.progress = p,
                Msg::FlashDone(r) => {
                    if let Err(e) = &r {
                        self.log.push(format!("Flash failed: {e}"));
                    }
                    self.flash_result = Some(r);
                    self.flashing = None;
                }
                Msg::AtFound(m, ip) => {
                    self.at_found.push((m, ip));
                    if self.at_found.len() == 1 {
                        self.at_target = Some(0);
                    }
                }
                Msg::AtScanDone => self.at_busy = false,
                Msg::AtText(s) => {
                    push_capped(&mut self.at_console, &s);
                    if !s.ends_with('\n') {
                        self.at_console.push('\n');
                    }
                }
                Msg::AtDone => self.at_busy = false,
                Msg::NlFound(m, ip) => {
                    self.nl_found.push((m, ip));
                    if self.nl_found.len() == 1 {
                        self.nl_target = Some(0);
                    }
                }
                Msg::NlScanDone => self.nl_busy = false,
                Msg::NlText(s) => push_capped(&mut self.nl_text, &s),
                Msg::NlStopped => self.nl_stream = None,
            }
        }
    }

    fn spawn(&self, f: impl FnOnce(Out) + Send + 'static) {
        let out = self.out.clone();
        std::thread::spawn(move || f(out));
    }

    // ---------------- Firmware tab ----------------

    fn firmware_tab(&mut self, ui: &mut egui::Ui) {
        let flashing = self.flashing.is_some();
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.ota_busy && !flashing && self.dev().is_some(), egui::Button::new("Scan")).clicked() {
                self.ota_busy = true;
                self.ota_found.clear();
                self.ota_target = None;
                let (dev, src) = (self.dev().unwrap().name.clone(), self.src);
                self.spawn(move |o| work::ota_scan(dev, src, o));
            }
            if self.ota_busy {
                ui.spinner();
            }
            ui.label("Finds bridges on this adapter, including devices on the far end of a HaLow link.");
        });
        ui.add_space(6.0);

        if self.ota_found.is_empty() {
            ui.label(if self.ota_busy { "Waiting for replies..." } else { "No devices yet. Press Scan." });
        } else {
            egui::Grid::new("ota").striped(true).num_columns(5).spacing([20.0, 6.0]).show(ui, |ui| {
                for h in ["", "MAC address", "Firmware", "Mode", "Chip ID"] {
                    ui.label(egui::RichText::new(h).strong());
                }
                ui.end_row();
                for (i, r) in self.ota_found.iter().enumerate() {
                    ui.add_enabled_ui(!flashing, |ui| ui.radio_value(&mut self.ota_target, Some(i), ""));
                    ui.monospace(mac_str(&r.mac));
                    ui.label(r.version_str());
                    ui.label(r.mode_str());
                    ui.monospace(format!("0x{:04x}", r.chipid));
                    ui.end_row();
                }
            });
        }

        ui.add_space(10.0);
        ui.separator();
        ui.label(egui::RichText::new("Firmware file").strong());
        ui.horizontal(|ui| {
            if ui.add_enabled(!flashing, egui::Button::new("Choose .bin...")).clicked() {
                if let Some(p) = rfd::FileDialog::new().add_filter("Firmware", &["bin"]).pick_file() {
                    self.load_fw(p);
                }
            }
            match &self.fw_path {
                Some(p) => ui.monospace(p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
                None => ui.label("or drop a file on this window"),
            };
        });
        if self.fw_path.is_some() {
            match &self.fw_info {
                Ok(f) => {
                    ui.label(format!(
                        "{}, built {}, chip 0x{:04x}, {:.1} KB of {:.0} KB",
                        f.version_str(),
                        f.date,
                        f.chipid,
                        f.size as f32 / 1024.0,
                        fwfile::FW_MAX as f32 / 1024.0
                    ));
                }
                Err(e) => {
                    ui.colored_label(ui.visuals().error_fg_color, e);
                }
            }
        }

        let target = self.ota_target.and_then(|i| self.ota_found.get(i)).cloned();
        let mismatch = match (&target, &self.fw_info) {
            (Some(t), Ok(f)) if t.chipid != f.chipid => {
                Some(format!("Chip ID mismatch: device 0x{:04x}, file 0x{:04x}.", t.chipid, f.chipid))
            }
            _ => None,
        };
        if let Some(m) = &mismatch {
            ui.colored_label(ui.visuals().error_fg_color, m);
        }
        // Settings saved by one major version (v1.6.x) are not laid out like the next (v2.x).
        let major_change = match (&target, &self.fw_info) {
            (Some(t), Ok(f)) => (t.version >> 24) != (f.sdk_version >> 24),
            _ => false,
        };
        if major_change && !self.load_defaults {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "Major version change: tick \"Reset settings\" so the new firmware starts from its defaults.",
            );
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let ready = target.is_some() && self.fw_info.is_ok() && mismatch.is_none() && !flashing && !self.ota_busy;
            if ui.add_enabled(ready, egui::Button::new("Flash")).clicked() {
                self.confirm_flash = true;
            }
            ui.add_enabled(!flashing, egui::Checkbox::new(&mut self.reboot_after, "Reboot when finished"));
            ui.add_enabled(
                !flashing && self.reboot_after,
                egui::Checkbox::new(&mut self.load_defaults, "Reset settings"),
            )
            .on_hover_text(
                "Reboot with the load-defaults flag: the device forgets its settings (name, keys, \
                 IP, pairing) and starts from the firmware defaults (this firmware: AP mode, \
                 192.168.123.1 or a DHCP address; boards without a factory MAC get a new one). \
                 Use it when moving between major versions.",
            );
            if let Some(cancel) = &self.flashing {
                if ui.button("Cancel").clicked() {
                    cancel.store(true, Ordering::Relaxed);
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.add_enabled(target.is_some() && !flashing, egui::Button::new("Reboot device")).clicked() {
                let (dev, src, mac) = (self.dev().unwrap().name.clone(), self.src, target.as_ref().unwrap().mac);
                self.spawn(move |o| work::reboot(dev, src, mac, 0, o));
            }
            if ui.add_enabled(target.is_some() && !flashing, egui::Button::new("Reset settings and reboot")).clicked() {
                self.confirm_reset = true;
            }
        });

        if flashing || self.flash_result.is_some() {
            ui.add(egui::ProgressBar::new(self.progress).show_percentage());
        }
        match &self.flash_result {
            Some(Ok(())) => {
                ui.colored_label(ok_color(ui), "Firmware written and verified.");
            }
            Some(Err(e)) => {
                ui.colored_label(ui.visuals().error_fg_color, e);
            }
            None => {}
        }

        if self.confirm_flash {
            let t = target.clone();
            let mut open = true;
            egui::Window::new("Flash firmware?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    if let (Some(t), Ok(f)) = (&t, &self.fw_info) {
                        ui.label(format!("Device: {}  ({}, {})", mac_str(&t.mac), t.version_str(), t.mode_str()));
                        ui.label(format!("New firmware: {}", f.version_str()));
                        ui.label("The device keeps running its current firmware until the image is complete and verified.");
                        if self.reboot_after && self.load_defaults {
                            ui.colored_label(
                                ui.visuals().warn_fg_color,
                                "Settings will be reset: the device comes back on its defaults (AP mode, 192.168.123.1 or DHCP).",
                            );
                        }
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Flash").clicked() {
                            if let (Some(t), Some(data)) = (t.clone(), self.fw_data.clone()) {
                                let cancel = Arc::new(AtomicBool::new(false));
                                self.flashing = Some(cancel.clone());
                                self.progress = 0.0;
                                self.flash_result = None;
                                let flags = if self.load_defaults { proto::REBOOT_LOADDEF } else { 0 };
                                let reboot = self.reboot_after.then_some(flags);
                                let (dev, src) = (self.dev().unwrap().name.clone(), self.src);
                                self.spawn(move |o| work::flash(dev, src, t, data, reboot, cancel, o));
                            }
                            self.confirm_flash = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_flash = false;
                        }
                    });
                });
            if !open {
                self.confirm_flash = false;
            }
        }

        if self.confirm_reset {
            let mut open = true;
            egui::Window::new("Reset settings?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    if let Some(t) = &target {
                        ui.label(format!("Device: {}  ({}, {})", mac_str(&t.mac), t.version_str(), t.mode_str()));
                    }
                    ui.label("The device forgets all settings (name, keys, IP, pairing) and reboots on its defaults:");
                    ui.label("AP mode, 192.168.123.1 or a DHCP address. Boards without a factory MAC get a new one.");
                    ui.horizontal(|ui| {
                        if ui.button("Reset and reboot").clicked() {
                            if let Some(t) = &target {
                                let (dev, src, mac) = (self.dev().unwrap().name.clone(), self.src, t.mac);
                                self.spawn(move |o| work::reboot(dev, src, mac, proto::REBOOT_LOADDEF, o));
                            }
                            self.confirm_reset = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.confirm_reset = false;
                        }
                    });
                });
            if !open {
                self.confirm_reset = false;
            }
        }
    }

    // ---------------- NETAT tab ----------------

    fn netat_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("AT commands over the vendor's NETAT broadcast (UDP 56789). Stock firmware answers; \
                  this project's firmware has NETAT off and uses the web Debug page instead.");
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.at_busy && self.dev().is_some(), egui::Button::new("Scan")).clicked() {
                self.at_busy = true;
                self.at_found.clear();
                self.at_target = None;
                let ((dev, ip), src) = (self.dev_ip().unwrap(), self.src);
                self.spawn(move |o| work::netat_scan(dev, ip, src, o));
            }
            if self.at_busy {
                ui.spinner();
            }
            let cur = self.at_target.and_then(|i| self.at_found.get(i)).map(|d| mac_str(&d.0)).unwrap_or("no device".into());
            egui::ComboBox::from_id_salt("at_dev").selected_text(cur).show_ui(ui, |ui| {
                for (i, (m, ip)) in self.at_found.iter().enumerate() {
                    ui.selectable_value(&mut self.at_target, Some(i), format!("{}  {}.{}.{}.{}", mac_str(m), ip[0], ip[1], ip[2], ip[3]));
                }
            });
        });
        ui.add_space(4.0);

        // Command row pinned to the bottom; the output fills the space above it.
        egui::TopBottomPanel::bottom("at_input")
            .frame(egui::Frame::none().inner_margin(egui::Margin { top: 6.0, ..Default::default() }))
            .show_separator_line(false)
            .show_inside(ui, |ui| self.netat_input(ui));
        egui::CentralPanel::default().frame(egui::Frame::none()).show_inside(ui, |ui| {
            egui::ScrollArea::vertical().id_salt("at_out").stick_to_bottom(true).auto_shrink(false).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut self.at_console.as_str()).code_editor().desired_width(f32::INFINITY));
            });
        });
    }

    fn netat_input(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let can_send = self.at_target.is_some() && !self.at_busy;
            let edit = ui.add_sized(
                [ui.available_width() - 130.0, 24.0],
                egui::TextEdit::singleline(&mut self.at_input).hint_text("AT+...   (Up/Down for history)").font(egui::TextStyle::Monospace),
            );
            if edit.has_focus() && !self.at_history.is_empty() {
                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && self.at_hist_pos > 0 {
                    self.at_hist_pos -= 1;
                    self.at_input = self.at_history[self.at_hist_pos].clone();
                }
                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) && self.at_hist_pos < self.at_history.len() {
                    self.at_hist_pos += 1;
                    self.at_input = self.at_history.get(self.at_hist_pos).cloned().unwrap_or_default();
                }
            }
            let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let send = ui.add_enabled(can_send, egui::Button::new("Send")).clicked() || (enter && can_send);
            if ui.button("Clear").clicked() {
                self.at_console.clear();
            }
            if send && !self.at_input.trim().is_empty() {
                let cmd = self.at_input.trim().to_string();
                push_capped(&mut self.at_console, &format!("> {cmd}\n"));
                if self.at_history.last() != Some(&cmd) {
                    self.at_history.push(cmd.clone());
                }
                self.at_hist_pos = self.at_history.len();
                self.at_input.clear();
                self.at_busy = true;
                let ((dev, ip), src) = (self.dev_ip().unwrap(), self.src);
                let dest = self.at_found[self.at_target.unwrap()].0;
                self.spawn(move |o| work::netat_cmd(dev, ip, src, dest, cmd, o));
                edit.request_focus();
            }
        });
    }

    // ---------------- NETLOG tab ----------------

    fn netlog_tab(&mut self, ui: &mut egui::Ui) {
        if self.nl_popout {
            ui.add_space(8.0);
            ui.label("NETLOG is open in its own window.");
            if ui.button("Bring it back here").clicked() {
                self.nl_popout = false;
            }
            return;
        }
        ui.label("Device log over the vendor's NETLOG broadcast (UDP 64320). Stock firmware answers; \
                  this project's firmware has NETLOG off and shows its log on the web Debug page.");
        self.netlog_view(ui);
    }

    /// NETLOG controls and log, in the tab or in the pop-out window.
    fn netlog_view(&mut self, ui: &mut egui::Ui) {
        let streaming = self.nl_stream.is_some();
        ui.horizontal(|ui| {
            if ui.add_enabled(!self.nl_busy && !streaming && self.dev().is_some(), egui::Button::new("Scan")).clicked() {
                self.nl_busy = true;
                self.nl_found.clear();
                self.nl_target = None;
                let ((dev, ip), src) = (self.dev_ip().unwrap(), self.src);
                self.spawn(move |o| work::netlog_scan(dev, ip, src, o));
            }
            if self.nl_busy {
                ui.spinner();
            }
            let cur = self.nl_target.and_then(|i| self.nl_found.get(i)).map(|d| mac_str(&d.0)).unwrap_or("no device".into());
            ui.add_enabled_ui(!streaming, |ui| {
                egui::ComboBox::from_id_salt("nl_dev").selected_text(cur).show_ui(ui, |ui| {
                    for (i, (m, ip)) in self.nl_found.iter().enumerate() {
                        ui.selectable_value(&mut self.nl_target, Some(i), format!("{}  {}.{}.{}.{}", mac_str(m), ip[0], ip[1], ip[2], ip[3]));
                    }
                });
            });
            if let Some(stop) = &self.nl_stream {
                if ui.button("Stop").clicked() {
                    stop.store(true, Ordering::Relaxed);
                }
                ui.colored_label(ok_color(ui), "live");
            } else if ui.add_enabled(self.nl_target.is_some(), egui::Button::new("Start")).clicked() {
                let stop = Arc::new(AtomicBool::new(false));
                self.nl_stream = Some(stop.clone());
                let ((dev, ip), src) = (self.dev_ip().unwrap(), self.src);
                let device = self.nl_found[self.nl_target.unwrap()].0;
                self.spawn(move |o| work::netlog_stream(dev, ip, src, device, stop, o));
            }
            if ui.button("Clear").clicked() {
                self.nl_text.clear();
            }
            if ui.add_enabled(!self.nl_text.is_empty(), egui::Button::new("Save...")).clicked() {
                if let Some(p) = rfd::FileDialog::new().set_file_name("device-log.txt").save_file() {
                    if let Err(e) = std::fs::write(&p, &self.nl_text) {
                        self.log.push(format!("Save failed: {e}"));
                    }
                }
            }
            let (label, hint) = if self.nl_popout {
                ("Dock", "Put the log back in the NETLOG tab")
            } else {
                ("Pop out", "Open the log in its own window, to watch it while using the NETAT console")
            };
            if ui.button(label).on_hover_text(hint).clicked() {
                self.nl_popout = !self.nl_popout;
            }
        });
        ui.add_space(4.0);
        egui::ScrollArea::vertical().id_salt("nl_out").stick_to_bottom(true).auto_shrink(false).show(ui, |ui| {
            ui.add(egui::TextEdit::multiline(&mut self.nl_text.as_str()).code_editor().desired_width(f32::INFINITY));
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll();

        // Firmware file dropped on the window
        if let Some(p) = ctx.input(|i| i.raw.dropped_files.first().and_then(|f| f.path.clone())) {
            if self.flashing.is_none() {
                self.tab = Tab::Firmware;
                self.load_fw(p);
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Adapter:");
                let busy = self.flashing.is_some() || self.nl_stream.is_some();
                let visible: Vec<usize> =
                    (0..self.devices.len()).filter(|&i| self.show_all || !self.devices[i].loopback).collect();
                let cur = self.dev().map(|d| d.label()).unwrap_or_default();
                ui.add_enabled_ui(!busy, |ui| {
                    egui::ComboBox::from_id_salt("adapter")
                        .width((ui.available_width() - 150.0).max(200.0))
                        .selected_text(cur)
                        .show_ui(ui, |ui| {
                            for i in visible {
                                ui.selectable_value(&mut self.selected, i, self.devices[i].label());
                            }
                        });
                    ui.checkbox(&mut self.show_all, "All");
                    if ui.button("Refresh").clicked() {
                        self.refresh();
                    }
                });
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Firmware, "Firmware (OTA)");
                ui.selectable_value(&mut self.tab, Tab::Netat, "NETAT console");
                ui.selectable_value(&mut self.tab, Tab::Netlog, "NETLOG");
                ui.selectable_value(&mut self.tab, Tab::About, "About");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Reversed for the right-to-left layout: shows Light, Dark, System.
                    let mut pref = ui.ctx().options(|o| o.theme_preference);
                    let before = pref;
                    for (p, name, hint) in [
                        (egui::ThemePreference::System, "System", "Follow the operating system's light or dark setting"),
                        (egui::ThemePreference::Dark, "Dark", "Dark theme"),
                        (egui::ThemePreference::Light, "Light", "Light theme"),
                    ] {
                        ui.selectable_value(&mut pref, p, name).on_hover_text(hint);
                    }
                    if pref != before {
                        ui.ctx().set_theme(pref);
                    }
                });
            });
            ui.add_space(2.0);
        });

        egui::TopBottomPanel::bottom("log").resizable(true).default_height(90.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for l in &self.log {
                    ui.monospace(l);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Firmware => self.firmware_tab(ui),
            Tab::Netat => self.netat_tab(ui),
            Tab::Netlog => self.netlog_tab(ui),
            Tab::About => about_tab(ui),
        });

        if self.nl_popout {
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("netlog"),
                egui::ViewportBuilder::default().with_title("NETLOG - Taixin toolbox").with_inner_size([760.0, 480.0]),
                |ctx, class| {
                    if class == egui::ViewportClass::Embedded {
                        // The platform has no second window: show it as a floating panel instead.
                        let mut open = true;
                        egui::Window::new("NETLOG").open(&mut open).default_size([700.0, 400.0]).show(ctx, |ui| self.netlog_view(ui));
                        self.nl_popout &= open;
                    } else {
                        egui::CentralPanel::default().show(ctx, |ui| self.netlog_view(ui));
                        if ctx.input(|i| i.viewport().close_requested()) {
                            self.nl_popout = false; // closing the window docks the log again
                        }
                    }
                },
            );
        }
    }
}

/// Console mode for testing without the GUI. Release builds on Windows have no console window.
fn cli(args: &[String]) -> bool {
    let cmd = args.first().map(String::as_str);
    if cmd == Some("--version") {
        println!("{}", version_line());
        return true;
    }
    if !matches!(cmd, Some("--list" | "--scan" | "--netat-scan" | "--netat")) {
        return false;
    }
    let devs = pcap::devices().unwrap_or_else(|e| {
        eprintln!("{e}");
        Vec::new()
    });
    if cmd == Some("--list") {
        devs.iter().enumerate().for_each(|(i, d)| println!("{i}: {}", d.label()));
        return true;
    }
    let Some(dev) = args.get(1).and_then(|n| n.parse::<usize>().ok()).and_then(|i| devs.get(i)) else {
        eprintln!("usage: --scan <n> | --netat-scan <n> | --netat <n> <mac> <AT command>   (n from --list)");
        return true;
    };
    let (tx, rx) = channel();
    let out = Out { tx, ui: egui::Context::default() };
    let (name, ip, src) = (dev.name.clone(), dev.ipv4.unwrap_or([0; 4]), work::random6());
    match cmd {
        Some("--scan") => work::ota_scan(name, src, out),
        Some("--netat-scan") => work::netat_scan(name, ip, src, out),
        _ => {
            let mac: Option<Mac> = args.get(2).and_then(|s| {
                let v: Vec<u8> = s.split(':').filter_map(|h| u8::from_str_radix(h, 16).ok()).collect();
                v.try_into().ok()
            });
            let (Some(mac), Some(at)) = (mac, args.get(3)) else {
                eprintln!("usage: --netat <n> <mac> <AT command>");
                return true;
            };
            work::netat_cmd(name, ip, src, mac, at.clone(), out);
        }
    }
    for m in rx.try_iter() {
        match m {
            Msg::Log(s) | Msg::AtText(s) => println!("{s}"),
            Msg::OtaFound(r) => println!("{}  {}  {}  chip 0x{:04x}", mac_str(&r.mac), r.version_str(), r.mode_str(), r.chipid),
            Msg::AtFound(m, ip) => println!("{}  {}.{}.{}.{}", mac_str(&m), ip[0], ip[1], ip[2], ip[3]),
            _ => {}
        }
    }
    true
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if cli(&args) {
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([860.0, 600.0]).with_title(format!("Taixin toolbox {}", env!("CARGO_PKG_VERSION"))),
        ..Default::default()
    };
    eframe::run_native(
        "Taixin toolbox",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::System); // light or dark, as the OS is set
            Ok(Box::new(App::new(&cc.egui_ctx)))
        }),
    )
}
