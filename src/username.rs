use std::os::raw::{c_char, c_int, c_ulong};
use std::ptr;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

#[link(name = "secur32")]
extern "system" {
    fn GetUserNameW(lpBuffer: *mut c_char, pcbBuffer: *mut c_ulong) -> c_int;
}

pub fn get_username() -> String {
    let mut size = 0;
    let success = unsafe { GetUserNameW(ptr::null_mut(), &mut size) == 0 };
    assert!(success);

    // Step 2. Allocate memory to put the Windows (UTF-16) string.
    let mut name: Vec<u16> = Vec::with_capacity(size.try_into().unwrap_or(std::usize::MAX));
    size = name.capacity().try_into().unwrap_or(std::u32::MAX);
    let orig_size = size;
    let fail = unsafe {
        GetUserNameW(name.as_mut_ptr().cast(), &mut size) == 0
    };
    if fail {
        return String::from("unknown");
    }
    assert_eq!(orig_size, size);

    unsafe {
        name.set_len(size.try_into().unwrap_or(std::usize::MAX));
    }
    let terminator = name.pop();
    assert_eq!(terminator, Some(0u16));

    // Step 3. Convert to Rust String
    let name = OsString::from_wide(&name);
    name.into_string().unwrap()
}
