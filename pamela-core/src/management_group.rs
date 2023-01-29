use std::str::FromStr;
use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Domain {
    Account,
    Auth,
    Password,
    Session,
}

impl FromStr for Domain {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Domain::*;

        Ok(match s {
            "account" => Account,
            "auth" => Auth,
            "password" => Password,
            "session" => Session,
            _ => return Err(()),
        })
    }
}

impl From<Domain> for &'static str {
    fn from(group: Domain) -> Self {
        use Domain::*;

        match group {
            Account => "account",
            Auth => "auth",
            Password => "password",
            Session => "session",
        }
    }
}

impl Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <&'static str>::from(*self).fmt(f)
    }
}
