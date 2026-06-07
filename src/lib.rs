pub mod core;
pub mod fs;
pub mod platform;

// GUI module requires tauri, only compile when feature is enabled
#[cfg(feature = "gui")]
pub mod gui;
