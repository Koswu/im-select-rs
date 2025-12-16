use std::io;
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::Foundation::{LPARAM, WPARAM}; 
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, PostMessageW, SendMessageW, WM_INPUTLANGCHANGEREQUEST // 注意：这里不要放 HWND
};

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY,
    VK_CONTROL, VK_MENU, VK_SHIFT, VK_SPACE,
};


// 简单的 verbose 日志宏
macro_rules! vlog {
    ($($arg:tt)*) => {
        if std::env::var("IM_SELECT_VERBOSE").is_ok() {
            eprintln!("[VERBOSE] {}", format_args!($($arg)*));
        }
    };
}

// HKL 类型定义（键盘布局句柄）
#[repr(transparent)]
#[derive(Clone, Copy)]
struct HKL(isize);

// 外部链接到 Windows API 函数
extern "system" {
    fn GetKeyboardLayout(idthread: u32) -> HKL;
}



// 安全封装：发送虚拟输入，集中 SendInput 的 unsafe
fn send_virtual_inputs(inputs: &[INPUT]) -> Result<(), io::Error> {
    if inputs.is_empty() {
        return Ok(());
    }

    let sent: u32 = unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to send all inputs: sent {}/{}", sent, inputs.len()),
        ));
    }

    Ok(())
}

/// 获取当前输入法的 locale ID（传统模式）
pub fn get_input_method() -> Result<String, io::Error> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get foreground window",
        ));
    }

    let thread_id = unsafe { GetWindowThreadProcessId(hwnd, None) };
    let layout = unsafe { GetKeyboardLayout(thread_id) };

    // 获取低 16 位作为 locale ID
    let locale = (layout.0 as u32) & 0x0000FFFF;

    Ok(locale.to_string())
}

/// 切换到指定的输入法 locale ID（传统模式）
pub fn switch_input_method(locale_str: &str) -> Result<(), io::Error> {
    let locale: u32 = locale_str.parse().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid locale ID: {}", locale_str),
        )
    })?;

    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0 == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to get foreground window",
        ));
    }

    let result = unsafe {
        PostMessageW(
            hwnd,
            WM_INPUTLANGCHANGEREQUEST,
            WPARAM(0),
            LPARAM(locale as isize),
        )
    };

    if result.is_ok() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to post input language change request",
        ))
    }
}

/// 解析按键字符串为虚拟键码
fn parse_key(key: &str) -> Option<VIRTUAL_KEY> {
    match key.to_lowercase().as_str() {
        "shift" => Some(VK_SHIFT),
        "ctrl" | "control" => Some(VK_CONTROL),
        "alt" => Some(VK_MENU),
        "space" => Some(VK_SPACE),
        _ => None,
    }
}

/// 从按键字符串创建 INPUT 结构（按下和释放）
fn create_key_inputs(keys_str: &str) -> Result<Vec<INPUT>, io::Error> {
    let mut inputs = Vec::new();
    let keys: Vec<&str> = keys_str.split('+').collect();

    // 按下所有键
    for key in &keys {
        let vk = parse_key(key.trim()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid key: {}", key))
        })?;

        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: Default::default(),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    // 释放所有键（逆序）
    for key in keys.iter().rev() {
        let vk = parse_key(key.trim()).unwrap();

        inputs.push(INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
    }

    Ok(inputs)
}

/// 获取当前输入法状态（UI Automation 模式，用于微软拼音等）
pub fn get_input_method_mspy(taskbar_name: &str, ime_pattern: &str) -> Result<String, io::Error> {
    get_input_method_mspy_impl(taskbar_name, ime_pattern)
}

fn get_input_method_mspy_impl(_taskbar_name: &str, _ime_pattern: &str) -> Result<String, io::Error> {
    unsafe {
        // 1. 获取当前活动窗口
        // 如果当前没有窗口（比如刚开机），可能会失败，返回错误即可
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return Err(io::Error::new(io::ErrorKind::Other, "No foreground window"));
        }

        // 2. 获取该窗口对应的默认 IME 窗口句柄
        // 这是一个隐藏的系统窗口，专门用来接收输入法控制消息
        let ime_hwnd = ImmGetDefaultIMEWnd(hwnd);
        if ime_hwnd.0 == 0 {
            // 如果拿不到 IME 窗口，通常意味着当前环境不支持 IME（比如纯英文环境/CMD核心模式）
            // 这种情况下，安全的做法是认为它是英文 (1033)
            return Ok("1033".to_string());
        }

        // 3. 发送消息询问状态
        // WM_IME_CONTROL = 0x0283
        // IMC_GETCONVERSIONMODE = 0x001
        const WM_IME_CONTROL: u32 = 0x0283;
        const IMC_GETCONVERSIONMODE: u32 = 0x0001;
        
        let conversion_mode = SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            WPARAM(IMC_GETCONVERSIONMODE as usize), 
            LPARAM(0)
        );

        // 4. 解析结果
        // SendMessageW 返回的是 LRESULT (isize)
        // IME_CMODE_NATIVE (0x0001) 位如果是 1，表示是“本地语言模式”（即中文）
        const IME_CMODE_NATIVE: u32 = 0x0001;
        let mode = conversion_mode.0 as u32;
        let is_chinese = (mode & IME_CMODE_NATIVE) != 0;

        if is_chinese {
            // 返回中文 Locale ID (微软拼音中文模式)
            Ok("中".to_string())
        } else {
            // 返回英文 Locale ID (微软拼音英文模式/纯英文)
            Ok("英".to_string())
        }
    }
}

/// 切换输入法（UI Automation 模式）
pub fn switch_input_method_mspy(
    target_mode: &str,
    taskbar_name: &str,
    ime_pattern: &str,
    switch_keys: &str,
    verify_attempts: u32,
    verify_interval_ms: u64,
    resend_retries: u32,
    resend_wait_ms: u64,
) -> Result<(), io::Error> {
    vlog!("Starting switch_input_method_mspy");
    vlog!("Target mode: '{}'", target_mode);
    vlog!("Switch keys: '{}'", switch_keys);
    vlog!(
        "Verify attempts: {}, interval: {}ms",
        verify_attempts,
        verify_interval_ms
    );
    vlog!(
        "Resend retries: {}, wait: {}ms",
        resend_retries,
        resend_wait_ms
    );

    // 先获取当前模式
    vlog!("Getting current input method mode...");
    let current_mode = get_input_method_mspy(taskbar_name, ime_pattern)?;
    vlog!("Current mode: '{}'", current_mode);

    // 如果已经是目标模式，不需要切换
    if current_mode == target_mode {
        vlog!("Already in target mode, no switch needed");
        return Ok(());
    }

    vlog!(
        "Need to switch from '{}' to '{}'",
        current_mode,
        target_mode
    );

    // 发送切换按键，并按配置进行验证和可选的重发
    let inputs = create_key_inputs(switch_keys)?;
    vlog!("Created {} key inputs", inputs.len());

    let verify = |target: &str| -> bool {
        // 初次短等待，避免立即读取旧状态
        std::thread::sleep(std::time::Duration::from_millis(100));
        for attempt in 0..verify_attempts {
            std::thread::sleep(std::time::Duration::from_millis(verify_interval_ms));
            vlog!("Verification attempt {}/{}", attempt + 1, verify_attempts);
            if let Ok(new_mode) = get_input_method_mspy(taskbar_name, ime_pattern) {
                vlog!("Current mode during verification: '{}'", new_mode);
                if new_mode == target {
                    vlog!("Verification successful!");
                    return true;
                }
            }
        }
        vlog!("Verification failed after {} attempts", verify_attempts);
        false
    };

    // 首次发送并验证
    vlog!("Sending switch keys (first attempt)...");
    send_virtual_inputs(&inputs)?;
    if verify(target_mode) {
        return Ok(());
    }

    // 可选重发策略
    for retry in 0..resend_retries {
        vlog!(
            "Resending switch keys (retry {}/{})",
            retry + 1,
            resend_retries
        );
        std::thread::sleep(std::time::Duration::from_millis(resend_wait_ms));
        send_virtual_inputs(&inputs)?;
        if verify(target_mode) {
            return Ok(());
        }
    }

    vlog!("Failed to switch input method after all retries");
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Verification failed after sending input",
    ))
}
