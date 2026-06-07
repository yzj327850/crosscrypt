use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm, Password};
use crate::core::{CrossCryptVolume, EncryptionConfig, VolumeStatus};

pub fn run_gui() {
    let theme = ColorfulTheme::default();
    
    loop {
        let choices = vec![
            "🔒 创建加密卷",
            "🔓 挂载加密卷", 
            "⏏️ 卸载加密卷",
            "📊 查看状态",
            "❌ 退出",
        ];
        
        let selection = Select::with_theme(&theme)
            .with_prompt("CrossCrypt - 请选择操作")
            .items(&choices)
            .default(0)
            .interact_opt()
            .unwrap_or(None);
        
        match selection {
            Some(0) => create_volume_gui(&theme),
            Some(1) => mount_volume_gui(&theme),
            Some(2) => unmount_volume_gui(&theme),
            Some(3) => check_status_gui(&theme),
            Some(4) | None => {
                println!("感谢使用 CrossCrypt!");
                break;
            }
            _ => {}
        }
    }
}

fn create_volume_gui(theme: &ColorfulTheme) {
    println!("\n【创建加密卷】\n");
    
    // 设备路径
    let device: String = Input::with_theme(theme)
        .with_prompt("设备路径 (例如: E: 或 /dev/sdb)")
        .interact_text()
        .unwrap_or_default();
    
    if device.trim().is_empty() {
        println!("❌ 设备路径不能为空");
        return;
    }
    
    // 卷标
    let label: String = Input::with_theme(theme)
        .with_prompt("卷标 (可选，直接回车跳过)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();
    
    let label = if label.trim().is_empty() { None } else { Some(label) };
    
    // 密码
    let password = Password::with_theme(theme)
        .with_prompt("设置密码")
        .with_confirmation("确认密码", "密码不匹配")
        .interact()
        .unwrap_or_default();
    
    if password.len() < 8 {
        println!("❌ 密码至少8个字符");
        return;
    }
    
    // 快速格式化
    let quick = Confirm::with_theme(theme)
        .with_prompt("使用快速格式化? (不加密现有数据)")
        .default(false)
        .interact()
        .unwrap_or(false);
    
    // 确认
    let confirm = Confirm::with_theme(theme)
        .with_prompt(&format!(
            "⚠️  即将创建加密卷:\n   设备: {}\n   卷标: {}\n\n   警告: 这将格式化设备上的所有数据!\n\n   确认继续?",
            device,
            label.as_deref().unwrap_or("无")
        ))
        .default(false)
        .interact()
        .unwrap_or(false);
    
    if !confirm {
        println!("操作已取消");
        return;
    }
    
    // 执行创建
    println!("\n⏳ 正在创建加密卷...");
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
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
    });
    
    match result {
        Ok(_) => println!("\n✅ 加密卷创建成功!"),
        Err(e) => println!("\n❌ 创建失败: {}", e),
    }
    
    println!();
}

fn mount_volume_gui(theme: &ColorfulTheme) {
    println!("\n【挂载加密卷】\n");
    
    let device: String = Input::with_theme(theme)
        .with_prompt("设备路径 (例如: E:)")
        .interact_text()
        .unwrap_or_default();
    
    if device.trim().is_empty() {
        println!("❌ 设备路径不能为空");
        return;
    }
    
    let password = Password::with_theme(theme)
        .with_prompt("输入密码")
        .interact()
        .unwrap_or_default();
    
    if password.is_empty() {
        println!("❌ 密码不能为空");
        return;
    }
    
    let mountpoint: String = Input::with_theme(theme)
        .with_prompt("挂载点 (可选，直接回车使用默认)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();
    
    let mountpoint = if mountpoint.trim().is_empty() { None } else { Some(mountpoint) };
    
    println!("\n⏳ 正在挂载...");
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        let mut volume = CrossCryptVolume::new(device);
        volume.mount(&password, mountpoint).await
    });
    
    match result {
        Ok(_) => println!("\n✅ 挂载成功!"),
        Err(e) => println!("\n❌ 挂载失败: {}", e),
    }
    
    println!();
}

fn unmount_volume_gui(theme: &ColorfulTheme) {
    println!("\n【卸载加密卷】\n");
    
    let target: String = Input::with_theme(theme)
        .with_prompt("设备路径或挂载点")
        .interact_text()
        .unwrap_or_default();
    
    if target.trim().is_empty() {
        println!("❌ 目标不能为空");
        return;
    }
    
    let force = Confirm::with_theme(theme)
        .with_prompt("强制卸载?")
        .default(false)
        .interact()
        .unwrap_or(false);
    
    println!("\n⏳ 正在卸载...");
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        CrossCryptVolume::unmount(&target, force).await
    });
    
    match result {
        Ok(_) => println!("\n✅ 卸载成功!"),
        Err(e) => println!("\n❌ 卸载失败: {}", e),
    }
    
    println!();
}

fn check_status_gui(theme: &ColorfulTheme) {
    println!("\n【查看状态】\n");
    
    let device: String = Input::with_theme(theme)
        .with_prompt("设备路径 (可选，直接回车查看所有)")
        .allow_empty(true)
        .interact_text()
        .unwrap_or_default();
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    if device.trim().is_empty() {
        let result = rt.block_on(async {
            CrossCryptVolume::list_volumes().await
        });
        
        match result {
            Ok(volumes) => {
                if volumes.is_empty() {
                    println!("\n📭 没有找到 CrossCrypt 加密卷");
                } else {
                    println!("\n📋 CrossCrypt 加密卷:");
                    for vol in volumes {
                        println!("   • {}", vol);
                    }
                }
            }
            Err(e) => println!("\n❌ 查询失败: {}", e),
        }
    } else {
        let result = rt.block_on(async {
            let volume = CrossCryptVolume::new(device);
            volume.status().await
        });
        
        match result {
            Ok(status) => {
                let status_text = match status {
                    VolumeStatus::Encrypted => "🔒 已加密",
                    VolumeStatus::NotEncrypted => "📂 未加密",
                    VolumeStatus::EncryptionInProgress => "⏳ 加密进行中",
                };
                println!("\n{}", status_text);
            }
            Err(e) => println!("\n❌ 查询失败: {}", e),
        }
    }
    
    println!();
}
