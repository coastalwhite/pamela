use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug)]
pub enum ModuleArgument {
    KeyValue { key: String, value: String },
    Set(String),
}

impl FromStr for ModuleArgument {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(equals_position) = s.find('=') else {
            if s.contains(|c| c == ' ' || c == '\t' || c == '\n') {
                return Err(());
            }

            return Ok(ModuleArgument::Set(String::from(s)));
        };

        if equals_position == 0 {
            return Err(());
        }

        let key = s[..equals_position].to_string();
        let value = s[equals_position + 1..].to_string();

        Ok(ModuleArgument::KeyValue { key, value })
    }
}

impl Display for ModuleArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ModuleArgument::*;

        match self {
            KeyValue { key, value } => {
                if value.contains(|c| c == ' ' || c == '\t') {
                    write!(f, "[{}={}]", key, value)
                } else {
                    write!(f, "{}={}", key, value)
                }
            }
            Set(key) => key.fmt(f),
        }
    }
}
