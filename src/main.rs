use clap::Parser;
use std::process;

mod platform;

#[derive(Parser, Debug)]
#[command(
    name = "im-select-rs",
    version = "0.1.0",
    about = "Switch input methods across different platforms",
    long_about = None
)]
struct Args {
    /// Input method identifier to switch to
    /// If not provided, the current input method will be displayed
    input_method: Option<String>,

    #[arg(short, long, help = "Enable verbose output for debugging")]
    verbose: bool,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        help = "Use UI Automation mode (for Microsoft Pinyin without English keyboard)"
    )]
    mspy: bool,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value = "任务栏",
        help = "Taskbar name for UI Automation"
    )]
    taskbar: String,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value = "(?:(?:托盘)?输入指示器|Input Indicator)\\s+(\\S+)",
        help = "Regex pattern to capture IME status"
    )]
    ime_pattern: String,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value = "ctrl+space",
        help = "Keys to switch IME (e.g., 'shift', 'ctrl+space')"
    )]
    switch_keys: String,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value_t = 5u32,
        help = "Verification attempts after sending switch keys"
    )]
    verify_attempts: u32,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value_t = 50u64,
        help = "Delay in ms between verification attempts"
    )]
    verify_interval_ms: u64,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value_t = 1u32,
        help = "Additional resend retries if verification fails"
    )]
    resend_retries: u32,

    #[cfg(target_os = "windows")]
    #[arg(
        long,
        default_value_t = 200u64,
        help = "Delay in ms before resending switch keys"
    )]
    resend_wait_ms: u64,
}

fn main() {
    let args = Args::parse();

    // Set verbose mode via environment variable for platform modules
    if args.verbose {
        std::env::set_var("IM_SELECT_VERBOSE", "1");
    }

    #[cfg(target_os = "windows")]
    if args.mspy {
        // 使用 UI Automation 模式
        match args.input_method {
            None => match platform::get_input_method_mspy(&args.taskbar, &args.ime_pattern) {
                Ok(im) => {
                    println!("{}", im);
                }
                Err(e) => {
                    eprintln!("Error getting input method: {}", e);
                    process::exit(1);
                }
            },
            Some(im) => {
                match platform::switch_input_method_mspy(
                    &im,
                    &args.taskbar,
                    &args.ime_pattern,
                    &args.switch_keys,
                    args.verify_attempts,
                    args.verify_interval_ms,
                    args.resend_retries,
                    args.resend_wait_ms,
                ) {
                    Ok(_) => {
                        // 切换成功，静默退出
                    }
                    Err(e) => {
                        eprintln!("Error switching input method: {}", e);
                        process::exit(1);
                    }
                }
            }
        }
        return;
    }

    // 默认模式
    match args.input_method {
        // 没有参数：获取当前输入法
        None => match platform::get_input_method() {
            Ok(im) => {
                println!("{}", im);
            }
            Err(e) => {
                eprintln!("Error getting input method: {}", e);
                process::exit(1);
            }
        },
        // 有参数：切换到指定输入法
        Some(im) => {
            match platform::switch_input_method(&im) {
                Ok(_) => {
                    // 切换成功，静默退出
                }
                Err(e) => {
                    eprintln!("Error switching input method: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
