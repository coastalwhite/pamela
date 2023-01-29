use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facility {
    Auth,
    Account,
    Session,
    Password,
}

impl FromStr for Facility {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auth" => Ok(Self::Auth),
            "account" => Ok(Self::Account),
            "session" => Ok(Self::Session),
            "password" => Ok(Self::Password),
            _ => Err(()),
        }
    }
}
