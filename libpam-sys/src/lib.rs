#![allow(non_camel_case_types)]

// "linux-pam" XOR "open-pam"
#[cfg(all(feature = "linux-pam", feature = "openpam"))]
compile_error!("You cannot have both the `linux-pam` and the `openpam` feature enabled");
#[cfg(not(any(feature = "linux-pam", feature = "openpam")))]
compile_error!("You need either the `linux-pam` or the `openpam` feature enabled");

use core::ffi::{c_char, c_int, c_void};

mod consts;

pub use consts::*;

#[repr(C)]
pub struct pam_handle_t {
    _internal: [u8; 376],
}

#[repr(C)]
pub struct pam_conv {
    pub conv:
        extern "C" fn(c_int, *const *mut pam_message, *mut *mut pam_response, *mut c_void) -> c_int,
    pub app_dataptr: *mut c_void,
}

#[repr(C)]
pub struct pam_response {
    pub resp: *mut c_char,
    pub resp_retcode: c_int,
}

#[repr(C)]
pub struct pam_message {
    pub msg_style: c_int,
    pub msg: *const c_char,
}

/// The service name
pub const PAM_SERVICE: c_int = 1;
/// The user name
pub const PAM_USER: c_int = 2;
/// The tty name
pub const PAM_TTY: c_int = 3;
/// The remote host name
pub const PAM_RHOST: c_int = 4;
/// The pam_conv structure
pub const PAM_CONV: c_int = 5;
/// The authentication token (password)
pub const PAM_AUTHTOK: c_int = 6;
/// The old authentication token
pub const PAM_OLDAUTHTOK: c_int = 7;
/// The remote user name
pub const PAM_RUSER: c_int = 8;
/// The prompt for getting a username
pub const PAM_USER_PROMPT: c_int = 9;

// Linux-PAM extensions
// TODO: Extract to feature
/// app supplied function to override failure delays
pub const PAM_FAIL_DELAY: c_int = 10;
/// X display name
pub const PAM_XDISPLAY: c_int = 11;
/// X server authentication data
pub const PAM_XAUTHDATA: c_int = 12;
/// The type for pam_get_authtok
pub const PAM_AUTHTOK_TYPE: c_int = 13;

// Message styles
pub const PAM_PROMPT_ECHO_OFF: c_int = 1;
pub const PAM_PROMPT_ECHO_ON: c_int = 2;
pub const PAM_ERROR_MSG: c_int = 3;
pub const PAM_TEXT_INFO: c_int = 4;


pub const PAM_MAX_NUM_MSG: c_int = 32;
pub const PAM_MAX_MSG_SIZE: c_int = 512;
pub const PAM_MAX_RESP_SIZE: c_int = 512;

// The Linux-PAM flags
/// Authentication service should not generate any messages
pub const PAM_SILENT: c_int = 0x8000;

// Note: these flags are used by pam_authenticate{,_secondary}()

/// The authentication service should return PAM_AUTH_ERROR if the user has a null authentication
/// token
pub const PAM_DISALLOW_NULL_AUTHTOK: c_int = 0x0001;

// Note: these flags are used for pam_setcred()

/// Set user credentials for an authentication service
pub const PAM_ESTABLISH_CRED: c_int = 0x0002;

/// Delete user credentials associated with an authentication service
pub const PAM_DELETE_CRED: c_int = 0x0004;

/// Reinitialize user credentials
pub const PAM_REINITIALIZE_CRED: c_int = 0x0008;

/// Extend lifetime of user credentials
pub const PAM_REFRESH_CRED: c_int = 0x0010;

// Note: these flags are used by pam_chauthtok

/// The password service should only update those passwords that have aged. If this flag is not
/// passed, the password service should update all passwords.
pub const PAM_CHANGE_EXPIRED_AUTHTOK: c_int = 0x0020;

extern "C" {
    pub fn pam_start(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const pam_conv,
        pamh: *mut *mut pam_handle_t,
    ) -> c_int;
    pub fn pam_start_confdir(
        service_name: *const c_char,
        user: *const c_char,
        pam_conversation: *const pam_conv,
        confdir: *const c_char,
        pamh: *mut *mut pam_handle_t,
    ) -> c_int;

    pub fn pam_end(pamh: *mut pam_handle_t, pam_status: c_int) -> c_int;

    pub fn pam_authenticate(pamh: *mut pam_handle_t, flags: c_int) -> c_int;
    pub fn pam_setcred(pamh: *mut pam_handle_t, flags: c_int) -> c_int;
    pub fn pam_acct_mgmt(pamh: *mut pam_handle_t, flags: c_int) -> c_int;

    pub fn pam_open_session(pamh: *mut pam_handle_t, flags: c_int) -> c_int;
    pub fn pam_close_session(pamh: *mut pam_handle_t, flags: c_int) -> c_int;

    pub fn pam_chauthtok(pamh: *mut pam_handle_t, flags: c_int) -> c_int;

    pub fn pam_strerror(pamh: *mut pam_handle_t, errnum: c_int) -> *const c_char;

    pub fn pam_set_item(pamh: *mut pam_handle_t, item_type: c_int, item: *const c_void) -> c_int;
    pub fn pam_get_item(pamh: *const pam_handle_t, item_type: c_int, item: *const *mut c_void);

    pub fn pam_putenv(pamh: *mut pam_handle_t, name_value: *const c_char) -> c_int;
    pub fn pam_getenv(pamh: *mut pam_handle_t, name: *const c_char) -> *const c_char;
    pub fn pam_getenvlist(pamh: *mut pam_handle_t) -> *mut *const c_char;
}

#[cfg(feature = "fail-delay")]
extern "C" {
    pub fn pam_fail_delay(pamh: *mut pam_handle_t, musec_delay: core::ffi::c_uint) -> c_int;
}
