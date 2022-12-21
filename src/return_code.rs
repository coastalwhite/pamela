// Modeled after Linux-PAM

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ReturnCode {
    /// Function completed successfully
    Success = 0,

    /// Failed to dynamically load the service module
    OpenError = 1,

    /// Symbol not found in the service module
    SymbolError = 2,
    
    /// Error in underlying service module
    ServiceError = 3,

    /// System Error
    SystemError = 4,
    
    /// Memory buffer error
    BufError = 5,

    /// Caller of the library does not have the proper authorization.
    PermissionDenied = 6,

    /// Failure to authenticate
    AuthenticationError = 7,

    /// Cannot access the authentication data due to insufficient credentials
    CredentialsInsufficient = 8,

    /// Authentication service is unable to fetch authentication information
    AuthInfoUnavailable = 9,
    
    /// User is not known to the authentication module
    UserUnknown = 10,
    
    /// An authentication service kept a retry count and the maximum retries has been reached.
    MaximumTriesReached = 11,

    /// A new authentication token is required. Usually, this is returned when the security
    /// policies require a new password. It is either because it was not set or because it has
    /// aged.
    NewAuthTokenRequired = 12,

    /// User account is expired
    AccountExpired = 13,

    /// Unable to add or remove an entry for the given session
    SessionError = 14,
    
    /// Given authentication service is unable to fetch user credentials
    CredentialsUnavailable = 15,

    /// User credentials have expired
    CredentialsExpired = 16,

    /// Failed to set user credentials
    CredentialsError = 17,

    /// No module specific data is present
    NoModuleData = 18,

    /// Conversation Error
    ConversationError = 19,

    /// Failure to manipulate authentication token
    AuthTokenManipulationError = 20,

    /// Failure to recover authentication token
    AuthTokenRecoverError = 21,
    
    /// Authentication token lock is busy
    AuthTokenLockBusy = 22,

    /// Authentication token aging is disabled
    AuthTokenDisableAging = 23,

    TryAgain = 24,
    Ignore = 25,
    Abort = 26,

    /// User's authentication token is expired
    AuthTokenExpired = 27,

    /// Given module is not known
    ModuleUnknown = 28,

    /// Passed bad item to pam_*_item()
    BadItem = 29,

    /// Data is not available yet
    ConversationAgain = 30,

    /// Conversation is incomplete
    Incomplete = 31,
}
