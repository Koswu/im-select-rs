// Windows 平台实现
#[cfg(target_os = "windows")]
mod windows_impl;

// macOS 平台实现
#[cfg(target_os = "macos")]
mod macos_impl;

// Linux 平台实现
#[cfg(target_os = "linux")]
mod linux_impl;

// 平台特定的实现
#[cfg(target_os = "windows")]
pub use windows_impl::{
    get_input_method, get_input_method_mspy, switch_input_method, switch_input_method_mspy,
};

#[cfg(target_os = "macos")]
pub use macos_impl::{get_input_method, switch_input_method};

#[cfg(target_os = "linux")]
pub use linux_impl::{get_input_method, switch_input_method};
