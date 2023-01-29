// Modeled after Linux-PAM

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnCode {
    /// Function completed successfully
    Success,

    /// Failed to dynamically load the service module
    OpenError,

    /// Symbol not found in the service module
    SymbolError,
    
    /// Error in underlying service module
    ServiceError,

    /// System Error
    SystemError,
    
    /// Memory buffer error
    BufError,

    /// Caller of the library does not have the proper authorization.
    PermissionDenied,

    /// Failure to authenticate
    AuthenticationError,

    /// Cannot access the authentication data due to insufficient credentials
    CredentialsInsufficient,

    /// Authentication service is unable to fetch authentication information
    AuthInfoUnavailable,
    
    /// User is not known to the authentication module
    UserUnknown,
    
    /// An authentication service kept a retry count and the maximum retries has been reached.
    MaximumTriesReached,

    /// A new authentication token is required. Usually, this is returned when the security
    /// policies require a new password. It is either because it was not set or because it has
    /// aged.
    NewAuthTokenRequired,

    /// User account is expired
    AccountExpired,

    /// Unable to add or remove an entry for the given session
    SessionError,
    
    /// Given authentication service is unable to fetch user credentials
    CredentialsUnavailable,

    /// User credentials have expired
    CredentialsExpired,

    /// Failed to set user credentials
    CredentialsError,

    /// No module specific data is present
    NoModuleData,

    /// Conversation Error
    ConversationError,

    /// Failure to manipulate authentication token
    AuthTokenManipulationError,

    /// Failure to recover authentication token
    AuthTokenRecoverError,
    
    /// Authentication token lock is busy
    AuthTokenLockBusy,

    /// Authentication token aging is disabled
    AuthTokenDisableAging,

    TryAgain,
    Ignore,
    Abort,

    /// User's authentication token is expired
    AuthTokenExpired,

    /// Given module is not known
    ModuleUnknown,

    /// Passed bad item to pam_*_item()
    BadItem,

    /// Data is not available yet
    ConversationAgain,

    /// Conversation is incomplete
    Incomplete,
}
