use eframe::egui;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::core::{CrossCryptVolume, EncryptionConfig, VolumeStatus};

pub struct CrossCryptApp {
    runtime: Arc<Runtime>,
    current_view: View,
    // Create volume state
    create_device: String,
    create_label: String,
    create_password: String,
    create_confirm: String,
    create_quick: bool,
    create_status: String,
    // Mount volume state
    mount_device: String,
    mount_password: String,
    mount_point: String,
    mount_status: String,
    // General status
    status_message: String,
    show_password: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum View {
    Main,
    Create,
    Mount,
    Status,
}

impl Default for CrossCryptApp {
    fn default() -> Self {
        Self {
            runtime: Arc::new(Runtime::new().unwrap()),
            current_view: View::Main,
            create_device: String::new(),
            create_label: String::new(),
            create_password: String::new(),
            create_confirm: String::new(),
            create_quick: false,
            create_status: String::new(),
            mount_device: String::new(),
            mount_password: String::new(),
            mount_point: String::new(),
            mount_status: String::new(),
            status_message: String::new(),
            show_password: false,
        }
    }
}

impl CrossCryptApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for CrossCryptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CrossCrypt - 磁盘加密工具");
            ui.separator();

            match self.current_view {
                View::Main => self.show_main(ui),
                View::Create => self.show_create(ui),
                View::Mount => self.show_mount(ui),
                View::Status => self.show_status(ui),
            }
        });
    }
}

impl CrossCryptApp {
    fn show_main(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            
            ui.label("选择操作:");
            ui.add_space(20.0);

            if ui.button("🔒 创建加密卷").clicked() {
                self.current_view = View::Create;
                self.create_status.clear();
            }

            ui.add_space(10.0);

            if ui.button("🔓 挂载加密卷").clicked() {
                self.current_view = View::Mount;
                self.mount_status.clear();
            }

            ui.add_space(10.0);

            if ui.button("📊 查看状态").clicked() {
                self.current_view = View::Status;
            }
        });
    }

    fn show_create(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← 返回").clicked() {
                self.current_view = View::Main;
            }
            ui.heading("创建加密卷");
        });

        ui.separator();

        ui.group(|ui| {
            ui.label("设备路径:");
            ui.text_edit_singleline(&mut self.create_device)
                .hint_text("例如: E: 或 /dev/sdb");

            ui.add_space(10.0);

            ui.label("卷标 (可选):");
            ui.text_edit_singleline(&mut self.create_label);

            ui.add_space(10.0);

            ui.label("密码:");
            if self.show_password {
                ui.text_edit_singleline(&mut self.create_password);
            } else {
                ui.add(egui::TextEdit::singleline(&mut self.create_password).password(true));
            }

            ui.add_space(5.0);

            ui.label("确认密码:");
            if self.show_password {
                ui.text_edit_singleline(&mut self.create_confirm);
            } else {
                ui.add(egui::TextEdit::singleline(&mut self.create_confirm).password(true));
            }

            ui.checkbox(&mut self.show_password, "显示密码");

            ui.add_space(10.0);

            ui.checkbox(&mut self.create_quick, "快速格式化 (不加密现有数据)");
        });

        ui.add_space(20.0);

        if !self.create_status.is_empty() {
            ui.colored_label(
                if self.create_status.starts_with("✓") {
                    egui::Color32::GREEN
                } else if self.create_status.starts_with("✗") {
                    egui::Color32::RED
                } else {
                    egui::Color32::YELLOW
                },
                &self.create_status,
            );
        }

        ui.add_space(10.0);

        if ui.button("创建加密卷").clicked() {
            if self.create_password != self.create_confirm {
                self.create_status = "✗ 密码不匹配".to_string();
                return;
            }

            if self.create_password.len() < 8 {
                self.create_status = "✗ 密码至少8个字符".to_string();
                return;
            }

            if self.create_device.is_empty() {
                self.create_status = "✗ 请输入设备路径".to_string();
                return;
            }

            self.create_status = "⏳ 正在创建加密卷...".to_string();
            
            let device = self.create_device.clone();
            let password = self.create_password.clone();
            let label = if self.create_label.is_empty() { None } else { Some(self.create_label.clone()) };
            let quick = self.create_quick;
            
            let rt = self.runtime.clone();
            
            // 使用 block_on 执行异步操作
            match rt.block_on(async move {
                let config = EncryptionConfig {
                    algorithm: crate::core::CryptoAlgorithm::Aes256Xts,
                    kdf: crate::core::KdfAlgorithm::Argon2id {
                        iterations: 3,
                        memory_kb: 64 * 1024,
                        parallelism: 4,
                    },
                    sector_size: 4096,
                    label,
                };

                let mut volume = CrossCryptVolume::new(device);
                volume.create(&password, config, quick).await
            }) {
                Ok(_) => {
                    self.create_status = "✓ 加密卷创建成功!".to_string();
                    self.create_password.clear();
                    self.create_confirm.clear();
                }
                Err(e) => {
                    self.create_status = format!("✗ 错误: {}", e);
                }
            }
        }
    }

    fn show_mount(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← 返回").clicked() {
                self.current_view = View::Main;
            }
            ui.heading("挂载加密卷");
        });

        ui.separator();

        ui.group(|ui| {
            ui.label("设备路径:");
            ui.text_edit_singleline(&mut self.mount_device)
                .hint_text("例如: E:");

            ui.add_space(10.0);

            ui.label("密码:");
            if self.show_password {
                ui.text_edit_singleline(&mut self.mount_password);
            } else {
                ui.add(egui::TextEdit::singleline(&mut self.mount_password).password(true));
            }

            ui.checkbox(&mut self.show_password, "显示密码");

            ui.add_space(10.0);

            ui.label("挂载点 (可选):");
            ui.text_edit_singleline(&mut self.mount_point)
                .hint_text("Windows: 盘符如 Z:");
        });

        ui.add_space(20.0);

        if !self.mount_status.is_empty() {
            ui.colored_label(
                if self.mount_status.starts_with("✓") {
                    egui::Color32::GREEN
                } else if self.mount_status.starts_with("✗") {
                    egui::Color32::RED
                } else {
                    egui::Color32::YELLOW
                },
                &self.mount_status,
            );
        }

        ui.add_space(10.0);

        if ui.button("挂载").clicked() {
            if self.mount_device.is_empty() {
                self.mount_status = "✗ 请输入设备路径".to_string();
                return;
            }

            if self.mount_password.is_empty() {
                self.mount_status = "✗ 请输入密码".to_string();
                return;
            }

            self.mount_status = "⏳ 正在挂载...".to_string();

            let device = self.mount_device.clone();
            let password = self.mount_password.clone();
            let mountpoint = if self.mount_point.is_empty() { None } else { Some(self.mount_point.clone()) };
            
            let rt = self.runtime.clone();

            match rt.block_on(async move {
                let mut volume = CrossCryptVolume::new(device);
                volume.mount(&password, mountpoint).await
            }) {
                Ok(_) => {
                    self.mount_status = "✓ 挂载成功!".to_string();
                    self.mount_password.clear();
                }
                Err(e) => {
                    self.mount_status = format!("✗ 错误: {}", e);
                }
            }
        }

        ui.add_space(10.0);

        if ui.button("卸载").clicked() {
            if self.mount_device.is_empty() {
                self.mount_status = "✗ 请输入设备路径".to_string();
                return;
            }

            let device = self.mount_device.clone();
            let rt = self.runtime.clone();

            match rt.block_on(async move {
                CrossCryptVolume::unmount(&device, false).await
            }) {
                Ok(_) => {
                    self.mount_status = "✓ 卸载成功!".to_string();
                }
                Err(e) => {
                    self.mount_status = format!("✗ 错误: {}", e);
                }
            }
        }
    }

    fn show_status(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("← 返回").clicked() {
                self.current_view = View::Main;
            }
            ui.heading("状态查看");
        });

        ui.separator();

        ui.label("此功能正在开发中...");
        ui.label("将显示所有 CrossCrypt 加密卷的状态");
    }
}

pub fn run_gui() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 500.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CrossCrypt",
        options,
        Box::new(|cc| Box::new(CrossCryptApp::new(cc))),
    )
    .unwrap();
}
