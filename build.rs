use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Platform-specific build configuration
    if target.contains("windows") {
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=kernel32");
    }
    
    // Enable AES-NI if available
    if is_x86_feature_detected!("aes") {
        println!("cargo:rustc-cfg=feature=\"aes-ni\"");
    }
}
