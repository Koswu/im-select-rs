use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use core_graphics::base::CGError;
use std::io;
use std::ptr;

// Carbon/HIToolbox 框架中的 TIS (Text Input Source) API
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardInputSource() -> *const libc::c_void;
    fn TISCreateInputSourceList(
        properties: *const libc::c_void,
        include_all_installed: bool,
    ) -> *const libc::c_void;
    fn TISGetInputSourceProperty(
        source: *const libc::c_void,
        property_key: CFStringRef,
    ) -> *const libc::c_void;
    fn TISSelectInputSource(source: *const libc::c_void) -> CGError;

    static kTISPropertyInputSourceID: CFStringRef;
}

// Core Foundation 数组操作
extern "C" {
    fn CFArrayGetCount(array: *const libc::c_void) -> isize;
    fn CFArrayGetValueAtIndex(array: *const libc::c_void, idx: isize) -> *const libc::c_void;
    fn CFRelease(cf: *const libc::c_void);
    fn CFDictionaryCreate(
        allocator: *const libc::c_void,
        keys: *const *const libc::c_void,
        values: *const *const libc::c_void,
        num_values: isize,
        key_callbacks: *const libc::c_void,
        value_callbacks: *const libc::c_void,
    ) -> *const libc::c_void;

    static kCFTypeDictionaryKeyCallBacks: *const libc::c_void;
    static kCFTypeDictionaryValueCallBacks: *const libc::c_void;
}

/// 获取当前输入法的标识符
pub fn get_input_method() -> Result<String, io::Error> {
    unsafe {
        let current_source = TISCopyCurrentKeyboardInputSource();
        if current_source.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get current input source",
            ));
        }

        let source_id_ref = TISGetInputSourceProperty(current_source, kTISPropertyInputSourceID);

        if source_id_ref.is_null() {
            CFRelease(current_source);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to get input source ID",
            ));
        }

        let cf_string = CFString::wrap_under_get_rule(source_id_ref as CFStringRef);
        let result = cf_string.to_string();

        CFRelease(current_source);

        Ok(result)
    }
}

/// 切换到指定的输入法
pub fn switch_input_method(input_source_id: &str) -> Result<(), io::Error> {
    unsafe {
        let source_id = CFString::new(input_source_id);

        // 创建过滤字典
        let keys: [*const libc::c_void; 1] = [kTISPropertyInputSourceID as *const libc::c_void];
        let values: [*const libc::c_void; 1] =
            [source_id.as_concrete_TypeRef() as *const libc::c_void];

        let filter = CFDictionaryCreate(
            ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            kCFTypeDictionaryKeyCallBacks,
            kCFTypeDictionaryValueCallBacks,
        );

        if filter.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to create filter dictionary",
            ));
        }

        // 获取匹配的输入源列表
        let keyboards = TISCreateInputSourceList(filter, false);
        CFRelease(filter);

        if keyboards.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Input source '{}' not found", input_source_id),
            ));
        }

        let count = CFArrayGetCount(keyboards);
        if count == 0 {
            CFRelease(keyboards);
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Input source '{}' not found", input_source_id),
            ));
        }

        // 选择第一个匹配的输入源
        let selected = CFArrayGetValueAtIndex(keyboards, 0);
        let result = TISSelectInputSource(selected);

        CFRelease(keyboards);

        if result != 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to select input source (error code: {})", result),
            ))
        } else {
            Ok(())
        }
    }
}
