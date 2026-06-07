use clap::{Parser, Subcommand};
use tracing::{info, error};

mod core;
mod fs;
mod platform;

use core::{CrossCryptVolume, EncryptionConfig, VolumeStatus};

#[derive(Parser)]
#[command(name = "crosscrypt")]
#[command(about = "Cross-platform portable disk encryption")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new encrypted volume
    Create {
        /// Device path or file
        #[arg(short, long)]
        device: String,
        
        /// Volume label
        #[arg(short, long)]
        label: Option<String>,
        
        /// Quick format (empty volume)
        #[arg(long)]
        quick: bool,
        
        /// Force operation without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Mount an encrypted volume
    Mount {
        /// Device path
        #[arg(short, long)]
        device: String,
        
        /// Mount point (drive letter on Windows)
        #[arg(short, long)]
        mountpoint: Option<String>,
    },
    
    /// Unmount an encrypted volume
    Unmount {
        /// Mount point or device
        #[arg(short, long)]
        target: String,
        
        /// Force unmount
        #[arg(short, long)]
        force: bool,
    },
    
    /// Lock volume (emergency unmount)
    Lock {
        /// Device or mount point
        #[arg(short, long)]
        target: String,
    },
    
    /// Check volume status
    Status {
        /// Device path
        device: Option<String>,
    },
    
    /// Resume interrupted encryption
    Resume {
        /// Device path
        #[arg(short, long)]
        device: String,
    },
    
    /// Benchmark encryption speed
    Benchmark,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Create { device, label, quick, force } => {
            info!("Creating encrypted volume on {}", device);
            create_volume(device, label, quick, force).await?;
        }
        Commands::Mount { device, mountpoint } => {
            info!("Mounting {}", device);
            mount_volume(device, mountpoint).await?;
        }
        Commands::Unmount { target, force } => {
            info!("Unmounting {}", target);
            unmount_volume(target, force).await?;
        }
        Commands::Lock { target } => {
            info!("Locking {}", target);
            lock_volume(target).await?;
        }
        Commands::Status { device } => {
            check_status(device).await?;
        }
        Commands::Resume { device } => {
            info!("Resuming encryption on {}", device);
            resume_encryption(device).await?;
        }
        Commands::Benchmark => {
            run_benchmark().await?;
        }
    }
    
    Ok(())
}

async fn create_volume(
    device: String,
    label: Option<String>,
    quick: bool,
    force: bool,
) -> anyhow::Result<()> {
    let config = EncryptionConfig {
        algorithm: core::CryptoAlgorithm::Aes256Xts,
        kdf: core::KdfAlgorithm::Argon2id {
            iterations: 3,
            memory_kb: 64 * 1024,
            parallelism: 4,
        },
        sector_size: 4096,
        label,
    };
    
    let mut volume = CrossCryptVolume::new(device.clone());
    
    if !force {
        // Check if device has data and confirm
        let has_data = volume.check_existing_data().await?;
        if has_data && !quick {
            println!("WARNING: Device {} contains data.", device);
            println!("The data will be preserved and encrypted in-place.");
            println!("This process can be interrupted and resumed.");
            print!("Continue? [y/N] ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Aborted.");
                return Ok(());
            }
        }
    }
    
    // Get password
    let password = rpassword::prompt_password("Enter password: ")?;
    let confirm = rpassword::prompt_password("Confirm password: ")?;
    
    if password != confirm {
        anyhow::bail!("Passwords do not match");
    }
    
    if password.len() < 8 {
        anyhow::bail!("Password must be at least 8 characters");
    }
    
    println!("Creating encrypted volume...");
    volume.create(&password, config, quick).await?;
    
    println!("✓ Volume created successfully!");
    println!("You can now mount it with: crosscrypt mount -d {}", device);
    
    Ok(())
}

async fn mount_volume(device: String, mountpoint: Option<String>) -> anyhow::Result<()> {
    let mut volume = CrossCryptVolume::new(device.clone());
    
    // Check volume status
    let status = volume.status().await?;
    match status {
        VolumeStatus::Encrypted => {}
        VolumeStatus::NotEncrypted => {
            anyhow::bail!("Device is not a CrossCrypt volume");
        }
        VolumeStatus::EncryptionInProgress => {
            anyhow::bail!("Encryption in progress. Use 'resume' to continue.");
        }
    }
    
    // Get password with retry limit
    let mut attempts = 0;
    const MAX_ATTEMPTS: u32 = 3;
    
    loop {
        let password = rpassword::prompt_password("Enter password: ")?;
        
        match volume.mount(&password, mountpoint.clone()).await {
            Ok(_) => {
                println!("✓ Volume mounted successfully!");
                return Ok(());
            }
            Err(e) => {
                attempts += 1;
                if attempts >= MAX_ATTEMPTS {
                    error!("Too many failed attempts. Locking for 5 minutes.");
                    volume.lock().await?;
                    anyhow::bail!("Volume locked due to too many failed attempts");
                }
                println!("Invalid password. {} attempts remaining.", MAX_ATTEMPTS - attempts);
            }
        }
    }
}

async fn unmount_volume(target: String, force: bool) -> anyhow::Result<()> {
    CrossCryptVolume::unmount(&target, force).await?;
    println!("✓ Volume unmounted.");
    Ok(())
}

async fn lock_volume(target: String) -> anyhow::Result<()> {
    CrossCryptVolume::emergency_lock(&target).await?;
    println!("✓ Volume locked.");
    Ok(())
}

async fn check_status(device: Option<String>) -> anyhow::Result<()> {
    if let Some(dev) = device {
        let volume = CrossCryptVolume::new(dev.clone());
        let status = volume.status().await?;
        
        println!("Device: {}", dev);
        match status {
            VolumeStatus::Encrypted => println!("Status: Encrypted"),
            VolumeStatus::NotEncrypted => println!("Status: Not encrypted"),
            VolumeStatus::EncryptionInProgress => println!("Status: Encryption in progress"),
        }
    } else {
        // List all CrossCrypt volumes
        let volumes = CrossCryptVolume::list_volumes().await?;
        println!("CrossCrypt volumes:");
        for vol in volumes {
            println!("  {}", vol);
        }
    }
    Ok(())
}

async fn resume_encryption(device: String) -> anyhow::Result<()> {
    let mut volume = CrossCryptVolume::new(device);
    let password = rpassword::prompt_password("Enter password: ")?;
    
    volume.resume_encryption(&password).await?;
    println!("✓ Encryption resumed and completed!");
    Ok(())
}

async fn run_benchmark() -> anyhow::Result<()> {
    println!("Running encryption benchmark...");
    core::benchmark().await?;
    Ok(())
}
