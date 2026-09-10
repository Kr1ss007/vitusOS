//! Native Linux PAM Authentication Subsystem.
//!
//! Directly interfaces with Linux Pluggable Authentication Modules (`/lib/x86_64-linux-gnu/libpam.so.0`).
//! Zero stubs, zero mocks, pure Linux PAM conversation handling.

use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub const PAM_SUCCESS: c_int = 0;
pub const PAM_PROMPT_ECHO_OFF: c_int = 1;
pub const PAM_PROMPT_ECHO_ON: c_int = 2;

#[repr(C)]
pub struct pam_message {
    pub msg_style: c_int,
    pub msg: *const c_char,
}

#[repr(C)]
pub struct pam_response {
    pub resp: *mut c_char,
    pub resp_retcode: c_int,
}

#[repr(C)]
pub struct pam_conv {
    pub conv: Option<
        unsafe extern "C" fn(
            num_msg: c_int,
            msg: *mut *const pam_message,
            resp: *mut *mut pam_response,
            appdata_ptr: *mut c_void,
        ) -> c_int,
    >,
    pub appdata_ptr: *mut c_void,
}

#[link(name = "pam")]
extern "C" {
    fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const pam_conv,
        pamh: *mut *mut c_void,
    ) -> c_int;

    fn pam_authenticate(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_acct_mgmt(pamh: *mut c_void, flags: c_int) -> c_int;

    fn pam_end(pamh: *mut c_void, pam_status: c_int) -> c_int;
}

unsafe extern "C" fn pam_conversation_fn(
    num_msg: c_int,
    msg: *mut *const pam_message,
    resp: *mut *mut pam_response,
    appdata_ptr: *mut c_void,
) -> c_int {
    if num_msg <= 0 || msg.is_null() || resp.is_null() || appdata_ptr.is_null() {
        return 19; // PAM_CONV_ERR
    }

    let password = &*(appdata_ptr as *const CString);
    let resp_arr = libc::calloc(num_msg as usize, std::mem::size_of::<pam_response>()) as *mut pam_response;
    if resp_arr.is_null() {
        return 19;
    }

    for i in 0..num_msg as isize {
        let m = *msg.offset(i);
        if !m.is_null() && ((*m).msg_style == PAM_PROMPT_ECHO_OFF || (*m).msg_style == PAM_PROMPT_ECHO_ON) {
            let r = libc::strdup(password.as_ptr());
            (*resp_arr.offset(i)).resp = r;
            (*resp_arr.offset(i)).resp_retcode = 0;
        }
    }

    *resp = resp_arr;
    PAM_SUCCESS
}

/// Authenticates a local Linux user against PAM (`/etc/pam.d/login` or `common-auth`).
pub fn authenticate_user(username: &str, password: &str) -> bool {
    let Ok(c_service) = CString::new("login") else { return false; };
    let Ok(c_user) = CString::new(username) else { return false; };
    let Ok(c_pass) = CString::new(password) else { return false; };

    let conv = pam_conv {
        conv: Some(pam_conversation_fn),
        appdata_ptr: &c_pass as *const CString as *mut c_void,
    };

    let mut pamh: *mut c_void = ptr::null_mut();
    let res = unsafe { pam_start(c_service.as_ptr(), c_user.as_ptr(), &conv, &mut pamh) };
    if res != PAM_SUCCESS || pamh.is_null() {
        return false;
    }

    let auth_res = unsafe { pam_authenticate(pamh, 0) };
    let acct_res = if auth_res == PAM_SUCCESS {
        unsafe { pam_acct_mgmt(pamh, 0) }
    } else {
        auth_res
    };

    unsafe { pam_end(pamh, acct_res) };
    auth_res == PAM_SUCCESS && acct_res == PAM_SUCCESS
}
