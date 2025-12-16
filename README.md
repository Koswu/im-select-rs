# im-select-rs

一个用 Rust 编写的跨平台输入法切换工具，由 [im-select](https://github.com/daipeihust/im-select) 提供参考。

## 功能特性

- 🚀 使用 Rust 编写，性能优秀
- 🔄 支持多平台：Windows、macOS、Linux
- 📦 单一可执行文件，无需额外依赖
- 🎯 简单的命令行接口
- 🏗️ 清晰的架构：共享命令行解析，平台特定的实现
- 🪟 Windows 双模式支持：
  - **默认模式**：通过 locale ID 切换（需要英文键盘）
  - **mspy 模式**：通过 UI Automation 检测和控制微软拼音等输入法（无需英文键盘）

Note: 目前仅在 Windows 上测试通过 mspy 模式切换微软拼音输入法，其他平台和输入法仍需进一步测试

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-username/im-select-rs.git
cd im-select-rs

# 构建 release 版本
cargo build --release

# 可执行文件位于 target/release/im-select-rs (或 im-select-rs.exe)
```

### 安装到系统

```bash
cargo install --path .
```

## 使用方法

### Windows

> **注意**：Windows 有两种工作模式：
> 1. **默认模式**（推荐）：适用于安装了英文键盘的系统，通过 locale ID 切换
> 2. **mspy 模式**（实验性）：适用于只使用微软拼音输入法的系统（需要 UI Automation 支持）

#### 默认模式（推荐）

**前提条件**：需要安装英文键盘
- Windows 10/11: 设置 -> 时间和语言 -> 语言 -> 添加键盘 -> 英语(美国)

#### 获取当前输入法的 locale ID
```bash
im-select-rs
# 输出示例: 2052 (简体中文)
```

#### 切换到指定输入法
```bash
im-select-rs 1033  # 切换到英文输入法
im-select-rs 2052  # 切换到简体中文输入法
```

常用的 Windows locale ID:
- `1033` - 英文 (美国)
- `2052` - 简体中文 (中国)
- `1041` - 日语
- `1042` - 韩语

#### mspy 模式（UI Automation）

此模式使用 Windows UI Automation API 来检测和控制微软拼音等输入法的状态，**无需安装英文键盘**。

**前提条件**：仅支持任务栏显示输入法指示器的系统（通常是 Windows 10/11 中文版）

**使用示例**：
```bash
# 获取当前输入法状态
im-select-rs --mspy
# 输出示例: 中文模式

# 切换到英语模式
im-select-rs --mspy 英语模式

# 切换到中文模式
im-select-rs --mspy 中文模式
```

**自定义参数**：
```bash
# 对于非简体中文系统，可能需要指定正确的任务栏名称和正则表达式
im-select-rs --mspy --taskbar "Taskbar" --ime-pattern "(?:(?:托盘)?输入指示器|Input Indicator)\s+(\S+)"
```

参数说明：
- `--taskbar` - 任务栏窗口名称（默认：任务栏）
- `--ime-pattern` - 正则表达式用于匹配输入法状态（默认：`(?:(?:托盘)?输入指示器|Input Indicator)\s+(\S+)`）
- `--verify-attempts` - 发送切换按键后的验证次数（默认：3）
- `--verify-interval-ms` - 验证尝试之间的延迟（毫秒）（默认：50）
- `--resend-retries` - 验证失败后的重试次数（默认：1）
- `--resend-wait-ms` - 重试前的等待延迟（毫秒）（默认：100）

#### 调试模式（Verbose）

当遇到"Input method indicator not found in taskbar"等错误时，可以使用 `-v` 或 `--verbose` 参数启用详细输出以便调试：

```bash
# 在 mspy 模式下启用 verbose 输出
im-select-rs --mspy --verbose

# 或使用短选项
im-select-rs --mspy -v

# 切换时也可以使用
im-select-rs --mspy -v 英语模式
```

verbose 模式会输出以下调试信息：
- 任务栏搜索参数（名称、正则表达式）
- UI Automation 初始化状态
- 找到的按钮数量
- 每个按钮的名称
- 正则表达式匹配结果
- 切换过程中的验证状态

这些信息可帮助诊断输入法切换问题，例如：
- 确认任务栏名称是否正确
- 查看实际的按钮名称以调整正则表达式
- 了解切换过程是否正常执行


### macOS

#### 获取当前输入法标识符
```bash
im-select-rs
# 输出示例: com.apple.keylayout.US
```

#### 切换到指定输入法
```bash
im-select-rs com.apple.keylayout.US              # 美式英文
im-select-rs com.apple.inputmethod.SCIM.ITABC    # 简体拼音
im-select-rs com.apple.inputmethod.TCIM.Cangjie  # 繁体仓颉
```

### Linux

Linux 平台需要使用系统特定的工具：

#### ibus
```bash
# 获取当前输入法
/usr/bin/ibus engine

# 切换输入法
/usr/bin/ibus engine xkb:us::eng
```

#### fcitx
```bash
# 获取当前输入法
fcitx-remote

# 切换输入法
fcitx-remote -s <input-method>
```

#### xkb-switch
```bash
# 获取当前输入法
xkb-switch -p

# 切换输入法
xkb-switch -s us
```

## VSCode 配置

### VSCodeVim

在 VSCode 的 `settings.json` 中配置：

#### Windows
```json
{
  "vim.autoSwitchInputMethod.enable": true,
  "vim.autoSwitchInputMethod.defaultIM": "1033",
  "vim.autoSwitchInputMethod.obtainIMCmd": "C:\\path\\to\\im-select-rs.exe",
  "vim.autoSwitchInputMethod.switchIMCmd": "C:\\path\\to\\im-select-rs.exe {im}"
}
```

### Windows mspy 模式
```json
{
  "vim.autoSwitchInputMethod.enable": true,
  "vim.autoSwitchInputMethod.defaultIM": "英",
  "vim.autoSwitchInputMethod.obtainIMCmd": "C:\\path\\to\\im-select-rs.exe --mspy",
  "vim.autoSwitchInputMethod.switchIMCmd": "C:\\path\\to\\im-select-rs.exe --mspy {im}"
}
```

#### macOS
```json
{
  "vim.autoSwitchInputMethod.enable": true,
  "vim.autoSwitchInputMethod.defaultIM": "com.apple.keylayout.US",
  "vim.autoSwitchInputMethod.obtainIMCmd": "/usr/local/bin/im-select-rs",
  "vim.autoSwitchInputMethod.switchIMCmd": "/usr/local/bin/im-select-rs {im}"
}
```

#### Linux (ibus)
```json
{
  "vim.autoSwitchInputMethod.enable": true,
  "vim.autoSwitchInputMethod.defaultIM": "xkb:us::eng",
  "vim.autoSwitchInputMethod.obtainIMCmd": "/usr/bin/ibus engine",
  "vim.autoSwitchInputMethod.switchIMCmd": "/usr/bin/ibus engine {im}"
}
```

## 开发

```bash
# 运行开发版本
cargo run

# 运行测试
cargo test

# 检查代码
cargo clippy

# 格式化代码
cargo fmt
```

## 许可证

MIT License

## 致谢

灵感来源于 [im-select](https://github.com/daipeihust/im-select) 项目。
