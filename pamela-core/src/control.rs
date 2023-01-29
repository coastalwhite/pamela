use std::fmt::Display;
use std::str::FromStr;

use crate::return_code::ReturnCode;

#[derive(Debug)]
pub enum Control {
    Required,
    Requisite,
    Sufficient,
    Optional,
    Include,
    Substack,
    Selection(Selection),
}

#[derive(Debug)]
pub struct Selection(Vec<SelectionItem>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionItem {
    value: Value,
    action: Action,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    ReturnCode(ReturnCode),
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Ignore,
    Bad,
    Die,
    Ok,
    Done,
    JumpOver(u32),
    Reset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlParseError {
    UnknownPreset(String),
    UnknownValue(String),
    UnknownAction(String),
    ExpectedEquals,
    UnexpectedEnd,
    UnclosedSelection,
    EmptyString,
    ZeroJump,
    ExpectedDigit,
    JumpOverflow,
}

impl FromStr for Control {
    type Err = ControlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "required" => Control::Required,
            "requisite" => Control::Requisite,
            "sufficient" => Control::Sufficient,
            "optional" => Control::Optional,
            "include" => Control::Include,
            "substack" => Control::Substack,
            s => Control::Selection(Selection::from_str(s)?),
        })
    }
}

impl FromStr for Selection {
    type Err = ControlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('[') {
            return Err(ControlParseError::UnknownPreset(s.to_string()));
        }

        if !s.ends_with(']') {
            return Err(ControlParseError::UnclosedSelection);
        }

        // TODO: This needs to be general whitespace
        let items = s[1..s.len() - 1]
            .split(' ')
            .map(|item_str| SelectionItem::from_str(item_str))
            .collect::<Result<Vec<SelectionItem>, ControlParseError>>()?;

        Ok(Self(items))
    }
}

impl FromStr for SelectionItem {
    type Err = ControlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(equals_position) = s.find('=') else {
            return Err(ControlParseError::ExpectedEquals);
        };

        let value = &s[..equals_position];
        let action = &s[equals_position + 1..];

        let Ok(value) = Value::from_str(value) else {
            return Err(ControlParseError::UnknownValue(value.to_string()));
        };
        let Ok(action) = Action::from_str(action) else {
            return Err(ControlParseError::UnknownAction(action.to_string()));
        };

        Ok(Self { value, action })
    }
}

impl FromStr for Value {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use crate::return_code::ReturnCode::*;
        use Value::*;

        Ok(match s {
            "default" => Default,
            "success" => ReturnCode(Success),
            "open_err" => ReturnCode(OpenError),
            "symbol_err" => ReturnCode(SymbolError),
            "service_err" => ReturnCode(ServiceError),
            "system_err" => ReturnCode(SystemError),
            "buf_err" => ReturnCode(BufError),
            "perm_denied" => ReturnCode(PermissionDenied),
            "auth_err" => ReturnCode(AuthenticationError),
            "cred_insufficient" => ReturnCode(CredentialsInsufficient),
            "authinfo_unavail" => ReturnCode(AuthInfoUnavailable),
            "user_unknown" => ReturnCode(UserUnknown),
            "maxtries" => ReturnCode(MaximumTriesReached),
            "new_authtok_reqd" => ReturnCode(NewAuthTokenRequired),
            "acct_expired" => ReturnCode(AccountExpired),
            "session_err" => ReturnCode(SessionError),
            "cred_unavail" => ReturnCode(CredentialsUnavailable),
            "cred_expired" => ReturnCode(CredentialsExpired),
            "cred_err" => ReturnCode(CredentialsError),
            "no_module_data" => ReturnCode(NoModuleData),
            "conv_err" => ReturnCode(ConversationError),
            "authtok_err" => ReturnCode(AuthTokenManipulationError),
            "authtok_recover_err" => ReturnCode(AuthTokenRecoverError),
            "authtok_lock_busy" => ReturnCode(AuthTokenLockBusy),
            "authtok_disable_aging" => ReturnCode(AuthTokenDisableAging),
            "try_again" => ReturnCode(TryAgain),
            "ignore" => ReturnCode(Ignore),
            "abort" => ReturnCode(Abort),
            "authtok_expired" => ReturnCode(AuthTokenExpired),
            "module_unknown" => ReturnCode(ModuleUnknown),
            "bad_item" => ReturnCode(BadItem),
            "conv_again" => ReturnCode(ConversationAgain),
            "incomplete" => ReturnCode(Incomplete),
            _ => return Err(()),
        })
    }
}

impl FromStr for Action {
    type Err = ControlParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ControlParseError::*;

        if s.is_empty() {
            return Err(EmptyString);
        }

        Ok(match s {
            "ignore" => Action::Ignore,
            "bad" => Action::Bad,
            "die" => Action::Die,
            "ok" => Action::Ok,
            "done" => Action::Done,
            "reset" => Action::Reset,
            _ => {
                let mut num = 0u32;

                for c in s.chars() {
                    let Some(digit) = c.to_digit(10) else {
                        return Err(ExpectedDigit);
                    };

                    num = num.checked_mul(10).ok_or(JumpOverflow)?;
                    num = num.checked_add(digit).ok_or(JumpOverflow)?;
                }

                if num == 0 {
                    // Linux-PAM: "There is a token ('ignore') for that"
                    return Err(ZeroJump);
                }

                Action::JumpOver(num)
            }
        })
    }
}

impl From<Value> for &'static str {
    fn from(value: Value) -> Self {
        use crate::return_code::ReturnCode::*;
        use Value::*;

        match value {
            Default => "default",
            ReturnCode(Success) => "success",
            ReturnCode(OpenError) => "open_err",
            ReturnCode(SymbolError) => "symbol_err",
            ReturnCode(ServiceError) => "service_err",
            ReturnCode(SystemError) => "system_err",
            ReturnCode(BufError) => "buf_err",
            ReturnCode(PermissionDenied) => "perm_denied",
            ReturnCode(AuthenticationError) => "auth_err",
            ReturnCode(CredentialsInsufficient) => "cred_insufficient",
            ReturnCode(AuthInfoUnavailable) => "authinfo_unavail",
            ReturnCode(UserUnknown) => "user_unknown",
            ReturnCode(MaximumTriesReached) => "maxtries",
            ReturnCode(NewAuthTokenRequired) => "new_authtok_reqd",
            ReturnCode(AccountExpired) => "acct_expired",
            ReturnCode(SessionError) => "session_err",
            ReturnCode(CredentialsUnavailable) => "cred_unavail",
            ReturnCode(CredentialsExpired) => "cred_expired",
            ReturnCode(CredentialsError) => "cred_err",
            ReturnCode(NoModuleData) => "no_module_data",
            ReturnCode(ConversationError) => "conv_err",
            ReturnCode(AuthTokenManipulationError) => "authtok_err",
            ReturnCode(AuthTokenRecoverError) => "authtok_recover_err",
            ReturnCode(AuthTokenLockBusy) => "authtok_lock_busy",
            ReturnCode(AuthTokenDisableAging) => "authtok_disable_aging",
            ReturnCode(TryAgain) => "try_again",
            ReturnCode(Ignore) => "ignore",
            ReturnCode(Abort) => "abort",
            ReturnCode(AuthTokenExpired) => "authtok_expired",
            ReturnCode(ModuleUnknown) => "module_unknown",
            ReturnCode(BadItem) => "bad_item",
            ReturnCode(ConversationAgain) => "conv_again",
            ReturnCode(Incomplete) => "incomplete",
        }
    }
}

impl Display for Control {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Control::Required => f.write_str("required"),
            Control::Requisite => f.write_str("requisite"),
            Control::Sufficient => f.write_str("sufficient"),
            Control::Optional => f.write_str("optional"),
            Control::Include => f.write_str("include"),
            Control::Substack => f.write_str("substack"),
            Control::Selection(selection) => selection.fmt(f),
        }
    }
}

impl Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;

        for (i, item) in self.0.iter().enumerate() {
            item.fmt(f)?;

            // If it is not the last item add a space
            if i != self.0.len() - 1 {
                f.write_str(" ")?;
            }
        }

        f.write_str("]")?;

        Ok(())
    }
}

impl Display for SelectionItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.value, self.action)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <&'static str>::from(*self).fmt(f)
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Action::*;

        match self {
            Ignore => f.write_str("ignore"),
            Bad => f.write_str("bad"),
            Die => f.write_str("die"),
            Ok => f.write_str("ok"),
            Done => f.write_str("done"),
            Reset => f.write_str("reset"),
            JumpOver(n) => write!(f, "{}", n),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control() {}

    #[test]
    fn selection() {
        // macro_rules! assert_test {
        //     ($s:literal =!> $err:expr) => {
        //         assert!(Selection::from_str($s).is_err());
        //     };
        //     ($s:literal => [$({ $value:expr, $action:expr }),*]) => {
        //         let selection = Selection::from_str($s);
        //         assert!(module_arg.is_ok());
        //         let SelectionItem { value, action } = module_arg.unwrap();
        //         assert_eq!(value, $value);
        //         assert_eq!(action, $action);
        //     };
        // }
        //
        // assert_test!("[incomplete=ignore]" => Value::ReturnCode(ReturnCode::Incomplete), Action::Ignore);
        // assert_test!("default=bad" => Value::Default, Action::Bad);
        // assert_test!("=xyz" =!> ControlParseError::UnknownValue);
        // assert_test!("default=xyz" =!> ControlParseError::UnknownAction);
    }

    #[test]
    fn selection_item() {
        macro_rules! assert_test {
            ($s:literal =!> $err:expr) => {
                assert_eq!(SelectionItem::from_str($s), Err($err));
            };
            ($s:literal => $value:expr, $action:expr) => {
                let item = SelectionItem::from_str($s);
                assert!(item.is_ok());
                let SelectionItem { value, action } = item.unwrap();
                assert_eq!(value, $value);
                assert_eq!(action, $action);
            };
        }

        assert_test!("incomplete=ignore" => Value::ReturnCode(ReturnCode::Incomplete), Action::Ignore);
        assert_test!("default=bad" => Value::Default, Action::Bad);
        assert_test!("=xyz" =!> ControlParseError::UnknownValue("".to_string()));
        assert_test!("default=xyz" =!> ControlParseError::UnknownAction("xyz".to_string()));
    }

    #[test]
    fn value() {
        macro_rules! assert_test {
            ($s:literal => !) => {
                assert!(Value::from_str($s).is_err());
            };
            ($s:literal => $variant:expr) => {
                assert_eq!(Value::from_str($s), Result::Ok($variant));
            };
        }

        use ReturnCode::*;

        assert_test!("default" => Value::Default);
        assert_test!("auth_err" => Value::ReturnCode(AuthenticationError));
        assert_test!("incomplete" => Value::ReturnCode(Incomplete));

        assert_test!("" => !);
        assert_test!("abc" => !);
    }

    #[test]
    fn action() {
        macro_rules! assert_test {
            ($s:literal => !) => {
                assert!(Action::from_str($s).is_err());
            };
            ($s:literal => $variant:expr) => {
                assert_eq!(Action::from_str($s), Result::Ok($variant));
            };
        }

        use Action::*;

        assert_test!("ignore" => Ignore);
        assert_test!("bad" => Bad);
        assert_test!("die" => Die);
        assert_test!("ok" => Ok);
        assert_test!("done" => Done);
        assert_test!("reset" => Reset);
        assert_test!("123" => JumpOver(123));

        assert_test!("" => !);
        assert_test!("abc" => !);
        assert_test!("-1" => !);
        assert_test!("123a" => !);
        assert_test!("9999999999999" => !);
    }
}
