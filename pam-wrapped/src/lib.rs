use std::ffi::{c_int, CString, NulError};

macro_rules! match_to_return_value {
    ($return_value:expr, $return_value_type:ty { $($const_value:ident => $variant:ident),* $(,)? }) => {{
        match $return_value {
            $(
                // TODO: Make general
                v if v == ::libpam_sys::status_code::linux_pam::$const_value => <$return_value_type>::$variant,
            )*
            v => <$return_value_type>::Unexpected(v as i16),
        }
    }}
}

pub use libpam_sys::{pam_conv, pam_message, pam_response};
use libpam_sys::{pam_handle_t, PAM_DISALLOW_NULL_AUTHTOK, PAM_SILENT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PamStartReturnValue {
    Success,
    SystemError,
    ServiceError,
    BufferError,

    /// This error is not documentated to be returned, but it is in fact returned by Linux-PAM
    Abort,

    // NOTE: We chose a `u16` here because it is the smallest possible `c_int` value according the
    // docs. Furthermore, we know that all the errors are within the range of `u16`.
    Unexpected(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PamEndReturnValue {
    Success,
    SystemError,
    BufferError,
    Unexpected(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PamAuthenticateReturnValue {
    Success,
    AuthenticationError,
    InsufficientCredentials,
    AuthenticationInfoUnavailable,
    UserUnknown,
    MaxTries,
    OpenFailure,
    SymbolNotFound,
    ServiceError,
    SystemError,
    BufferError,
    ConversationError,
    PermissionDenied,
    Abort,
    Unexpected(i16),
}

pub struct PamHandle {
    raw_handle: *mut pam_handle_t,
    service_name: CString,
    user: Option<CString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PamAuthenticateFlags {
    pub silent: bool,
    pub disallow_null_auth_token: bool,
}

impl From<PamAuthenticateFlags> for c_int {
    fn from(value: PamAuthenticateFlags) -> Self {
        let int_flags = c_int::default();

        if value.silent {
            int_flags |= PAM_SILENT;
        }

        if value.disallow_null_auth_token {
            int_flags |= PAM_DISALLOW_NULL_AUTHTOK;
        }

        int_flags
    }
}

impl PamHandle {
    pub fn new() -> Self {
        Self {
            raw_handle: std::ptr::null_mut(),
            service_name: CString::default(),
            user: None,
        }
    }
}

pub unsafe fn pam_start(
    service_name: &str,
    user: Option<&str>,
    pam_conversation: &pam_conv,
    pamh: &mut PamHandle,
) -> Result<PamStartReturnValue, NulError> {
    let service_name = CString::new(service_name)?;
    let service_name_ptr = service_name.as_ptr();

    let user = if let Some(user) = user {
        Some(CString::new(user)?)
    } else {
        None
    };
    let user_ptr = user.map_or(std::ptr::null(), |user| user.as_ptr());

    pamh.service_name = service_name;
    pamh.user = user;

    let pam_conversation = pam_conversation as *const pam_conv;
    let pamh = (&mut pamh.raw_handle) as *mut *mut pam_handle_t;

    let return_value =
        unsafe { libpam_sys::pam_start(service_name_ptr, user_ptr, pam_conversation, pamh) };

    Ok(match_to_return_value! {
        return_value, PamStartReturnValue {
            PAM_SUCCESS => Success,
            PAM_SYSTEM_ERR => SystemError,
            PAM_SERVICE_ERR => ServiceError,
            PAM_BUF_ERR => BufferError,
            PAM_ABORT => Abort,
        }
    })
}

pub unsafe fn pam_end(pamh: &PamHandle, pam_status: i32) -> PamEndReturnValue {
    if option_env!("RUST_PAM_DISABLE_CHECKS").is_none() {
        debug_assert_eq!(pam_status & 0xFFFF0000, 0, "This PAM status is not cross-platform compatible. Use the 'RUST_PAM_DISABLE_CHECKS' environment variable to ignore this check.");
    }

    let pamh = pamh.raw_handle;

    // NOTE: pam_status is a variable that is passed the cleanup function for data fields set with
    // `pam_set_data`. This is an `c_int` so it is a little bit difficult to make a rust compatible
    // API for this. In this case, we chose to make it a `i32` and truncate it, when it doesn't
    // fit.
    let pam_status = pam_status as c_int;

    let return_value = unsafe { libpam_sys::pam_end(pamh, pam_status) };

    match_to_return_value! {
        return_value, PamEndReturnValue {
            PAM_SUCCESS => Success,
            PAM_SYSTEM_ERR => SystemError,
            PAM_BUF_ERR => BufferError,
        }
    }
}

pub unsafe fn pam_authenticate(
    pamh: &PamHandle,
    flags: PamAuthenticateFlags,
) -> PamAuthenticateReturnValue {
    let pamh = pamh.raw_handle;
    let flags = c_int::from(flags);

    let return_value = unsafe { libpam_sys::pam_authenticate(pamh, flags) };

    match_to_return_value! {
        return_value, PamAuthenticateReturnValue {
            PAM_SUCCESS => Success,
            PAM_AUTH_ERR => AuthenticationError,
            PAM_CRED_INSUFFICIENT => InsufficientCredentials,
            PAM_AUTHINFO_UNAVAIL => AuthenticationInfoUnavailable,
            PAM_USER_UNKNOWN => UserUnknown,
            PAM_MAXTRIES => MaxTries,
            PAM_OPEN_ERR => OpenFailure,
            PAM_SYMBOL_ERR => SymbolNotFound,
            PAM_SERVICE_ERR => ServiceError,
            PAM_SYSTEM_ERR => SystemError,
            PAM_BUF_ERR => BufferError,
            PAM_CONV_ERR => ConversationError,
            PAM_PERM_DENIED => PermissionDenied,
            PAM_ABORT => Abort,
        }
    }
}

extern "C" {
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
    pub fn pam_fail_delay(pamh: *mut pam_handle_t, musec_delay: std::ffi::c_uint) -> c_int;
}
