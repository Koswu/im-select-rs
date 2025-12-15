use std::io;

/// 获取当前输入法
/// Linux 平台需要使用系统特定的工具，如 ibus, fcitx, xkb-switch 等
pub fn get_input_method() -> Result<String, io::Error> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Direct input method control is not supported on Linux.\n\
        Please use system-specific tools:\n\
        - For ibus: /usr/bin/ibus engine\n\
        - For fcitx: fcitx-remote\n\
        - For xkb-switch: xkb-switch -p",
    ))
}

/// 切换输入法
/// Linux 平台需要使用系统特定的工具
pub fn switch_input_method(_input_method: &str) -> Result<(), io::Error> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Direct input method control is not supported on Linux.\n\
        Please use system-specific tools:\n\
        - For ibus: /usr/bin/ibus engine <engine-name>\n\
        - For fcitx: fcitx-remote -s <input-method>\n\
        - For xkb-switch: xkb-switch -s <layout>",
    ))
}
