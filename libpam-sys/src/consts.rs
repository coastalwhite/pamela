macro_rules! reexport_based_on_features {
    (
     $(
        $(#[$($attrss:tt)*])*
        $name:ident
     ),* $(,)?
    ) => {
        $(
            $(#[$($attrss)*])*
            #[cfg(any(feature = "linux_pam", feature = "openpam"))]
            ///
            /// This constant has a value dependent whether the `linux_pam` or `openpam` feature
            /// was selected.
            pub const $name: core::ffi::c_int = {
                if cfg!(feature = "linux-pam") {
                    linux_pam::$name
                } else {
                    openpam::$name
                }
            };
        )*
    };
}

pub mod status_code {
    reexport_based_on_features! {
        /// Successful function return
        PAM_SUCCESS,

        /// dlopen() failure when dynamically loading a service module
        PAM_OPEN_ERR,

        /// Symbol not found
        PAM_SYMBOL_ERR,

        /// Error in service module
        PAM_SERVICE_ERR,

        /// System error
        PAM_SYSTEM_ERR,

        /// Memory buffer error
        PAM_BUF_ERR,

        /// The caller does not possess the required authority
        PAM_PERM_DENIED,

        /// Authentication failure
        PAM_AUTH_ERR,

        /// Cannot access authentication database because credentials supplied are insufficient
        PAM_CRED_INSUFFICIENT,

        /// Cannot retrieve authentication information
        PAM_AUTHINFO_UNAVAIL,

        /// The user is not known to the underlying account management module
        PAM_USER_UNKNOWN,

        /// An authentication service has maintained a retry count which has been reached.  No further
        /// retries should be attempted
        PAM_MAXTRIES,

        /// New authentication token required. This is normally returned if the machine security policies
        /// require that the password should be changed because the password is NULL or it has aged
        PAM_NEW_AUTHTOK_REQD,

        /// User account has expired
        PAM_ACCT_EXPIRED,

        /// Can not make/remove an entry for the specified session
        PAM_SESSION_ERR,

        /// Underlying authentication service can not retrieve user credentials unavailable
        PAM_CRED_UNAVAIL,

        /// User credentials expired
        PAM_CRED_EXPIRED,

        /// Failure setting user credentials
        PAM_CRED_ERR,

        /// No module specific data is present
        PAM_NO_MODULE_DATA,

        /// Conversation error
        PAM_CONV_ERR,

        /// Authentication token manipulation error
        PAM_AUTHTOK_ERR,

        /// Authentication information cannot be recovered
        PAM_AUTHTOK_RECOVERY_ERR,

        /// Authentication token lock busy
        PAM_AUTHTOK_LOCK_BUSY,

        /// Authentication token aging disabled
        PAM_AUTHTOK_DISABLE_AGING,

        /// Unable to complete operation. Try again
        PAM_TRY_AGAIN,

        /// Ignore underlying account module regardless of whether the control flag is required,
        /// optional, or sufficient
        PAM_IGNORE,

        /// General PAM failure
        PAM_ABORT,

        /// user's authentication token has expired
        PAM_AUTHTOK_EXPIRED,

        /// Module type unknown
        PAM_MODULE_UNKNOWN,

        /// Bad item passed to pam_*_item()
        PAM_BAD_ITEM,
    }

    pub mod linux_pam {
        use core::ffi::c_int;

        pub const PAM_SUCCESS: c_int = 0;
        pub const PAM_OPEN_ERR: c_int = 1;
        pub const PAM_SYMBOL_ERR: c_int = 2;
        pub const PAM_SERVICE_ERR: c_int = 3;
        pub const PAM_SYSTEM_ERR: c_int = 4;
        pub const PAM_BUF_ERR: c_int = 5;
        pub const PAM_PERM_DENIED: c_int = 6;
        pub const PAM_AUTH_ERR: c_int = 7;
        pub const PAM_CRED_INSUFFICIENT: c_int = 8;
        pub const PAM_AUTHINFO_UNAVAIL: c_int = 9;
        pub const PAM_USER_UNKNOWN: c_int = 10;
        pub const PAM_MAXTRIES: c_int = 11;
        pub const PAM_NEW_AUTHTOK_REQD: c_int = 12;
        pub const PAM_ACCT_EXPIRED: c_int = 13;
        pub const PAM_SESSION_ERR: c_int = 14;
        pub const PAM_CRED_UNAVAIL: c_int = 15;
        pub const PAM_CRED_EXPIRED: c_int = 16;
        pub const PAM_CRED_ERR: c_int = 17;
        pub const PAM_NO_MODULE_DATA: c_int = 18;
        pub const PAM_CONV_ERR: c_int = 19;
        pub const PAM_AUTHTOK_ERR: c_int = 20;
        pub const PAM_AUTHTOK_RECOVERY_ERR: c_int = 21;
        pub const PAM_AUTHTOK_LOCK_BUSY: c_int = 22;
        pub const PAM_AUTHTOK_DISABLE_AGING: c_int = 23;
        pub const PAM_TRY_AGAIN: c_int = 24;
        pub const PAM_IGNORE: c_int = 25;
        pub const PAM_ABORT: c_int = 26;
        pub const PAM_AUTHTOK_EXPIRED: c_int = 27;
        pub const PAM_MODULE_UNKNOWN: c_int = 28;

        pub const PAM_BAD_ITEM: c_int = 29;
        /// conversation function is event driven and data is not available yet
        pub const PAM_CONV_AGAIN: c_int = 30;
        /// please call this function again to complete authentication stack. Before calling again,
        /// verify that conversation is completed
        pub const PAM_INCOMPLETE: c_int = 31;
    }

    pub mod openpam {
        use core::ffi::c_int;

        pub const PAM_SUCCESS: c_int = 0;
        pub const PAM_OPEN_ERR: c_int = 1;
        pub const PAM_SYMBOL_ERR: c_int = 2;
        pub const PAM_SERVICE_ERR: c_int = 3;
        pub const PAM_SYSTEM_ERR: c_int = 4;
        pub const PAM_BUF_ERR: c_int = 5;
        pub const PAM_CONV_ERR: c_int = 6;
        pub const PAM_PERM_DENIED: c_int = 7;
        pub const PAM_MAXTRIES: c_int = 8;
        pub const PAM_AUTH_ERR: c_int = 9;
        pub const PAM_NEW_AUTHTOK_REQD: c_int = 10;
        pub const PAM_CRED_INSUFFICIENT: c_int = 11;
        pub const PAM_AUTHINFO_UNAVAIL: c_int = 12;
        pub const PAM_USER_UNKNOWN: c_int = 13;
        pub const PAM_CRED_UNAVAIL: c_int = 14;
        pub const PAM_CRED_EXPIRED: c_int = 15;
        pub const PAM_CRED_ERR: c_int = 16;
        pub const PAM_ACCT_EXPIRED: c_int = 17;
        pub const PAM_AUTHTOK_EXPIRED: c_int = 18;
        pub const PAM_SESSION_ERR: c_int = 19;
        pub const PAM_AUTHTOK_ERR: c_int = 20;
        pub const PAM_AUTHTOK_RECOVERY_ERR: c_int = 21;
        pub const PAM_AUTHTOK_LOCK_BUSY: c_int = 22;
        pub const PAM_AUTHTOK_DISABLE_AGING: c_int = 23;
        pub const PAM_NO_MODULE_DATA: c_int = 24;
        pub const PAM_IGNORE: c_int = 25;
        pub const PAM_ABORT: c_int = 26;
        pub const PAM_TRY_AGAIN: c_int = 27;
        pub const PAM_MODULE_UNKNOWN: c_int = 28;

        /// Domain unknown
        pub const PAM_DOMAIN_UNKNOWN: c_int = 29;
        /// OpenPAM extension
        pub const PAM_BAD_HANDLE: c_int = 30; 
        /// OpenPAM extension
        pub const PAM_BAD_ITEM: c_int = 31;
        /// OpenPAM extension
        pub const PAM_BAD_FEATURE: c_int = 32;
        /// OpenPAM extension
        pub const PAM_BAD_CONSTANT: c_int = 33;
        /// OpenPAM extension
        pub const PAM_NUM_ERRORS: c_int = 33;
    }
}

pub mod message_constants {
    use core::ffi::c_int;

    // Message styles
    pub const PAM_PROMPT_ECHO_OFF: c_int = 1;
    pub const PAM_PROMPT_ECHO_ON: c_int = 2;
    pub const PAM_ERROR_MSG: c_int = 3;
    pub const PAM_TEXT_INFO: c_int = 4;

    pub mod linux_pam {
        use core::ffi::c_int;
        
        // Linux-PAM specific types

        /// yes/no/maybe conditionals
        pub const PAM_RADIO_TYPE: c_int = 5;

        /// This is for server client non-human interaction.. these are NOT part of the X/Open PAM
        /// specification.
        pub const PAM_BINARY_PROMPT: c_int = 7;
    }

    pub const PAM_MAX_NUM_MSG: c_int = 32;
    pub const PAM_MAX_MSG_SIZE: c_int = 512;
    pub const PAM_MAX_RESP_SIZE: c_int = 512;
}
