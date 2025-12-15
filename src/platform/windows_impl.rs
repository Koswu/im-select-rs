use regex::Regex;
use std::io;
use windows::core::BSTR;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Variant::{VARENUM, VARIANT};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, PropertyConditionFlags, TreeScope_Children,
    TreeScope_Descendants, UIA_ButtonControlTypeId, UIA_ControlTypePropertyId,
    UIA_NamePropertyId, UIA_PROPERTY_ID,
};

// VARIANT 类型常量
const VT_BSTR: VARENUM = VARENUM(8);
const VT_I4: VARENUM = VARENUM(3);
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_MENU,
    VK_SHIFT, VK_SPACE, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowThreadProcessId, PostMessageW, WM_INPUTLANGCHANGEREQUEST,
};

// HKL 类型定义（键盘布局句柄）
#[repr(transparent)]
#[derive(Clone, Copy)]
struct HKL(isize);

// 外部链接到 Windows API 函数
extern "system" {
    fn GetKeyboardLayout(idthread: u32) -> HKL;
}

// RAII 封装 COM 初始化/反初始化，减少分散的 unsafe
struct ComInit;

impl ComInit {
    fn new() -> Result<Self, io::Error> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to initialize COM: {}", e)))?;
        Ok(ComInit)
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

// 安全封装：构造包含 BSTR 的 VARIANT
fn variant_bstr(s: &str) -> VARIANT {
    let bstr = BSTR::from(s);
    let mut v = VARIANT::default();
    unsafe {
        let p_var = &mut *v.Anonymous.Anonymous;
        p_var.vt = VT_BSTR;
        std::ptr::write(
            &mut p_var.Anonymous.bstrVal as *mut _,
            std::mem::ManuallyDrop::new(bstr),
        );
    }
    v
}

// 安全封装：构造包含 I4 的 VARIANT
// 安全封装：创建 UIAutomation 实例
fn uia_create() -> Result<IUIAutomation, io::Error> {
    unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create UIAutomation: {}", e)))
}

// 安全封装：创建属性条件（Ex）
fn uia_create_property_condition_ex(
    automation: &IUIAutomation,
    prop_id: UIA_PROPERTY_ID,
    value: VARIANT,
    flags: PropertyConditionFlags,
) -> Result<windows::Win32::UI::Accessibility::IUIAutomationCondition, io::Error> {
    unsafe { automation.CreatePropertyConditionEx(prop_id, value, flags) }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create property condition: {}", e)))
}

// 安全封装：获取根元素
fn uia_get_root(automation: &IUIAutomation) -> Result<windows::Win32::UI::Accessibility::IUIAutomationElement, io::Error> {
    unsafe { automation.GetRootElement() }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to get root element: {}", e)))
}

// 安全封装：查找子元素
fn uia_find_first(
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    scope: windows::Win32::UI::Accessibility::TreeScope,
    condition: &windows::Win32::UI::Accessibility::IUIAutomationCondition,
) -> Result<windows::Win32::UI::Accessibility::IUIAutomationElement, io::Error> {
    unsafe { element.FindFirst(scope, condition) }
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to find first: {}", e)))
}

// 安全封装：查找所有元素
fn uia_find_all(
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    scope: windows::Win32::UI::Accessibility::TreeScope,
    condition: &windows::Win32::UI::Accessibility::IUIAutomationCondition,
) -> Result<windows::Win32::UI::Accessibility::IUIAutomationElementArray, io::Error> {
    unsafe { element.FindAll(scope, condition) }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to find all: {}", e)))
}

// 安全封装：数组长度
fn uia_array_len(array: &windows::Win32::UI::Accessibility::IUIAutomationElementArray) -> u32 {
    let len = unsafe { array.Length() }.unwrap_or(0);
    if len < 0 { 0 } else { len as u32 }
}

// 安全封装：获取数组元素
fn uia_array_get(
    array: &windows::Win32::UI::Accessibility::IUIAutomationElementArray,
    index: u32,
) -> Result<windows::Win32::UI::Accessibility::IUIAutomationElement, io::Error> {
    let len = unsafe { array.Length() }.unwrap_or(0);
    if index as i32 >= len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Index {} out of bounds (len = {})", index, len),
        ));
    }

    unsafe { array.GetElement(index as i32) }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to get element {}: {}", index, e)))
}

// 安全封装：获取元素当前名称
fn uia_element_current_name(element: &windows::Win32::UI::Accessibility::IUIAutomationElement) -> Result<BSTR, io::Error> {
    unsafe { element.CurrentName() }
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to get CurrentName: {}", e)))
}

fn variant_i4(i: i32) -> VARIANT {
    let mut v = VARIANT::default();
    unsafe {
        let p_var = &mut *v.Anonymous.Anonymous;
        p_var.vt = VT_I4;
        std::ptr::write(&mut p_var.Anonymous.lVal as *mut _, i);
    }
    v
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
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid key: {}", key),
            )
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
    let _com = ComInit::new()?;
    get_input_method_mspy_impl(taskbar_name, ime_pattern)
}

fn get_input_method_mspy_impl(
    taskbar_name: &str,
    ime_pattern: &str,
) -> Result<String, io::Error> {
    // 创建 UI Automation 实例（封装）
    let automation: IUIAutomation = uia_create()?;

    // 获取桌面元素（封装）
    let desktop = uia_get_root(&automation)?;

    // 查找任务栏
    let taskbar_variant = variant_bstr(taskbar_name);
    
    let taskbar_condition = uia_create_property_condition_ex(
        &automation,
        UIA_NamePropertyId,
        taskbar_variant,
        PropertyConditionFlags::default(),
    )?;

    let taskbar = uia_find_first(&desktop, TreeScope_Children, &taskbar_condition)
        .map_err(|e| io::Error::new(io::ErrorKind::NotFound, format!("Failed to find taskbar '{}': {}", taskbar_name, e)))?;

    // 查找所有按钮
    let button_variant = variant_i4(UIA_ButtonControlTypeId.0 as i32);
    
    let button_condition = uia_create_property_condition_ex(
        &automation,
        UIA_ControlTypePropertyId,
        button_variant,
        PropertyConditionFlags::default(),
    )?;

    let buttons = uia_find_all(&taskbar, TreeScope_Descendants, &button_condition)?;

    // 编译正则表达式
    let re = Regex::new(ime_pattern).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Invalid regex pattern: {}", e),
        )
    })?;

    // 遍历按钮查找输入法指示器
    let length = uia_array_len(&buttons);
    for i in 0..length {
        if let Ok(button) = uia_array_get(&buttons, i) {
            if let Ok(name_bstr) = uia_element_current_name(&button) {
                let name = name_bstr.to_string();
                if let Some(caps) = re.captures(&name) {
                    if let Some(mode) = caps.get(1) {
                        return Ok(mode.as_str().to_string());
                    }
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Input method indicator not found in taskbar",
    ))
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
    // 先获取当前模式
    let current_mode = get_input_method_mspy(taskbar_name, ime_pattern)?;

    // 如果已经是目标模式，不需要切换
    if current_mode == target_mode {
        return Ok(());
    }

    // 发送切换按键，并按配置进行验证和可选的重发
    let inputs = create_key_inputs(switch_keys)?;

    let verify = |target: &str| -> bool {
        // 初次短等待，避免立即读取旧状态
        std::thread::sleep(std::time::Duration::from_millis(100));
        for _ in 0..verify_attempts {
            std::thread::sleep(std::time::Duration::from_millis(verify_interval_ms));
            if let Ok(new_mode) = get_input_method_mspy(taskbar_name, ime_pattern) {
                if new_mode == target {
                    return true;
                }
            }
        }
        false
    };

    // 首次发送并验证
    send_virtual_inputs(&inputs)?;
    if verify(target_mode) {
        return Ok(());
    }

    // 可选重发策略
    for _ in 0..resend_retries {
        std::thread::sleep(std::time::Duration::from_millis(resend_wait_ms));
        send_virtual_inputs(&inputs)?;
        if verify(target_mode) {
            return Ok(());
        }
    }

    Err(io::Error::new(
        io::ErrorKind::Other,
        "Verification failed after sending input",
    ))
}
