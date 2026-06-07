# CrossCrypt Portable

This folder contains platform-specific files for portable deployment.

## Windows

- `autorun.inf` - Auto-run configuration for Windows Explorer
- Place `crosscrypt.exe` in the root of the USB drive

## macOS

- `CrossCrypt.app` - Application bundle structure
- Place `crosscrypt` binary in `CrossCrypt.app/Contents/MacOS/`
- Place `icon.icns` in `CrossCrypt.app/Contents/Resources/`

## Linux

- `crosscrypt.desktop` - Desktop entry for file managers
- Place `crosscrypt` binary in `/usr/local/bin/` or alongside the desktop file

## Usage

1. Copy the appropriate files for your platform to the USB drive
2. Run `crosscrypt` to start the encryption tool
3. Follow the prompts to create or mount encrypted volumes
