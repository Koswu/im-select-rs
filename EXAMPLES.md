# 示例用法

## Windows 示例

### 查看当前输入法
```powershell
PS> .\im-select-rs.exe
2052
```

输出 `2052` 表示当前使用的是简体中文输入法。

### 切换到英文输入法
```powershell
PS> .\im-select-rs.exe 1033
```

切换成功后不会有输出。

### 验证切换结果
```powershell
PS> .\im-select-rs.exe
1033
```

### 常用 Windows Locale ID 列表

| Locale ID | 语言/输入法 |
|-----------|------------|
| 1033 | 英语（美国）|
| 2052 | 简体中文（中国）|
| 1041 | 日语（日本）|
| 1042 | 韩语（韩国）|
| 1028 | 繁体中文（台湾）|
| 1031 | 德语（德国）|
| 1036 | 法语（法国）|
| 1034 | 西班牙语（西班牙）|
| 1040 | 意大利语（意大利）|
| 1049 | 俄语（俄罗斯）|

### 在脚本中使用

```powershell
# 保存当前输入法
$current_im = .\im-select-rs.exe

# 切换到英文
.\im-select-rs.exe 1033

# 做一些需要英文输入法的操作
# ...

# 恢复原输入法
.\im-select-rs.exe $current_im
```

## macOS 示例

### 查看当前输入法
```bash
$ ./im-select-rs
com.apple.keylayout.US
```

### 切换到简体拼音
```bash
$ ./im-select-rs com.apple.inputmethod.SCIM.ITABC
```

### 常用 macOS 输入法标识符

| 标识符 | 输入法 |
|-------|--------|
| com.apple.keylayout.US | 美式英文 |
| com.apple.keylayout.ABC | ABC |
| com.apple.inputmethod.SCIM.ITABC | 简体拼音 |
| com.apple.inputmethod.SCIM.Shuangpin | 简体双拼 |
| com.apple.inputmethod.TCIM.Cangjie | 繁体仓颉 |
| com.apple.inputmethod.TCIM.Zhuyin | 繁体注音 |

### 获取系统所有输入法列表（macOS）

```bash
# 可以使用以下命令列出所有可用的输入法
$ osascript -e 'tell application "System Events" to get name of every source of keyboard sources'
```

## VSCode 集成示例

### 基本配置

在 VSCode 的 `settings.json` 中添加：

```json
{
  "vim.autoSwitchInputMethod.enable": true,
  "vim.autoSwitchInputMethod.defaultIM": "1033",
  "vim.autoSwitchInputMethod.obtainIMCmd": "C:\\tools\\im-select-rs.exe",
  "vim.autoSwitchInputMethod.switchIMCmd": "C:\\tools\\im-select-rs.exe {im}"
}
```

### 工作原理

1. 进入插入模式时，VSCodeVim 会调用 `obtainIMCmd` 保存当前输入法
2. 退出插入模式时，VSCodeVim 会调用 `switchIMCmd` 切换到 `defaultIM`
3. 再次进入插入模式时，恢复之前保存的输入法

## 性能测试

```powershell
# 测试获取输入法的性能
PS> Measure-Command { .\im-select-rs.exe }

Days              : 0
Hours             : 0
Minutes           : 0
Seconds           : 0
Milliseconds      : 5
Ticks             : 52341
TotalDays         : 6.05798611111111E-08
TotalHours        : 1.45391666666667E-06
TotalMinutes      : 8.72350000000001E-05
TotalSeconds      : 0.0052341
TotalMilliseconds : 5.2341

# 非常快！约 5 毫秒
```

## 故障排除

### Windows: "Input method indicator not found in taskbar"

这个错误通常发生在 mspy 模式下，表示无法在任务栏找到输入法指示器。诊断方法：

1. **使用 verbose 模式查看详细信息**：
```powershell
PS> .\im-select-rs.exe --mspy --verbose
[VERBOSE] Starting get_input_method_mspy_impl
[VERBOSE] Taskbar name: '任务栏'
[VERBOSE] IME pattern: '(?:(?:托盘)?输入指示器|Input Indicator)\s+(\S+)'
[VERBOSE] Creating UI Automation instance...
[VERBOSE] UI Automation instance created successfully
[VERBOSE] Getting desktop element...
[VERBOSE] Desktop element obtained
[VERBOSE] Searching for taskbar with name: '任务栏'
[VERBOSE] Taskbar found successfully
[VERBOSE] Searching for buttons in taskbar...
[VERBOSE] Found 15 buttons in taskbar
[VERBOSE] Scanning 15 buttons for input method indicator...
[VERBOSE] Button 0: '用户推广的通知区域，右键单击以访问上下文菜单'
[VERBOSE] Button 1: '显示隐藏的图标'
[VERBOSE] Button 2: '中文(简体，中国) 中文模式'
[VERBOSE] Matched input method indicator: '中文模式'
中文模式
```

2. **根据 verbose 输出调整参数**：
   - 如果任务栏名称不正确，使用 `--taskbar` 参数指定正确的名称
   - 如果按钮中找到了输入法指示器但正则表达式不匹配，使用 `--ime-pattern` 调整正则表达式

3. **示例：针对英文系统**：
```powershell
PS> .\im-select-rs.exe --mspy --taskbar "Taskbar" --ime-pattern "Input Indicator\\s+(\\S+)" -v
```

### Windows: "Failed to get foreground window"

这个错误通常发生在没有活动窗口的情况下。确保：
1. 有一个活动的应用程序窗口
2. 程序有足够的权限访问窗口信息

### macOS: "Input source not found"

确保输入法标识符正确：
1. 使用 `im-select-rs` 查看当前输入法的完整标识符
2. 确保输入法已经在系统偏好设置中启用

### 切换不生效

在某些应用程序中，输入法切换可能需要：
1. 应用程序获得焦点
2. 稍等片刻让系统处理切换请求
