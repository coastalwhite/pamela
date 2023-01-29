#[derive(Debug)]
pub struct Service(String);
#[derive(Debug)]
pub struct UnproperServiceName(usize, String);

impl AsRef<str> for Service {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Service {
    type Error = UnproperServiceName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let unproper_position =
            value.find(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-'));
        
        if let Some(unproper_position) = unproper_position {
            return Err(UnproperServiceName(unproper_position, value));
        }

        Ok(Self(value))
    }
}
