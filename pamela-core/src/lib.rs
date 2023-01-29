// pam.conf
// service      type        control         module-path     module-arguments
//
// pam.d/*
// type         control     module-path     module-arguments

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs::{self, File, ReadDir};
use std::io::{self, Read};
use std::path::Path;
use std::str::FromStr;

mod control;
mod management_group;
mod module_arguments;
mod module_path;
mod parsing;
mod return_code;

pub use self::control::Control;
use self::control::ControlParseError;
pub use self::management_group::Domain;
pub use self::module_arguments::ModuleArgument;
pub use self::module_path::ModulePath;
use self::parsing::*;
pub use self::return_code::ReturnCode;

const PAM_CONF_PATH: &'static str = "/etc/pam.conf";
const PAM_D_PATH: &'static str = "/etc/pam.d";

/// Configuration environment present on a system consisting of several services
#[derive(Debug)]
pub struct PamConfig {
    services: Vec<PamService>,
}

/// Named set of [`PamRule`]
#[derive(Debug)]
pub struct PamService {
    name: String,
    rules: Vec<PamRule>,
}

/// Single line a PAM configuration file
#[derive(Debug)]
pub struct PamRule {
    is_logging_enabled: bool,
    domain: Domain,
    control: Control,
    module_path: ModulePath,
    module_arguments: Vec<ModuleArgument>,
}

#[derive(Debug)]
pub enum PamConfigSyntaxError {
    HasNewLine,
    CommentLine,
    EmptyLine,

    UnclosedBracket,
    WrongDomain(String),
    WrongControl(ControlParseError),
    WrongModulePath(String),
    WrongModuleArgs(String),
}

#[derive(Debug)]
pub enum PamConfigError {
    Syntax(PamConfigSyntaxError),
    Io(io::Error),
    NotAFilename,
    NonUTF8Filename,
}

impl From<io::Error> for PamConfigError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<PamConfigSyntaxError> for PamConfigError {
    fn from(value: PamConfigSyntaxError) -> Self {
        Self::Syntax(value)
    }
}

impl PamConfig {
    /// Get the associated [`PamService`]s
    pub fn services(&self) -> &[PamService] {
        &self.services
    }

    /// Read a [`PamConfig`] from a packed configuration file
    ///
    /// On Linux, this is usually a file in the `/etc/pam.conf` file. To parse the a service from
    /// the `/etc/pam.d` directory look at [`PamService::from_file`]. 
    pub fn from_file(path: impl AsRef<Path>) -> Result<PamConfig, PamConfigError> {
        let mut file = File::open(path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(Self::from_str(&contents)?)
    }

    /// Read a [`PamConfig`] from all service files in a directory
    ///
    /// On Linux, these are usually the files in the `/etc/pam.d` directory. To parse a general
    /// configuration file such as `/etc/pam.conf` look at [`PamConfig::from_file`]. 
    pub fn from_dir(dir: ReadDir) -> Result<PamConfig, PamConfigError> {
        let services = dir
            .map(|dir_entry| PamService::from_file(dir_entry?.path()))
            .collect::<Result<Vec<PamService>, PamConfigError>>()?;

        Ok(PamConfig { services })
    }

    /// Read a [`PamConfig`] from the current system
    ///
    /// If `/etc/pam.d` exists, the service files there are used. If `/etc/pam.d` does not exist,
    /// the `/etc/pam.conf` is used.
    pub fn from_system() -> Result<PamConfig, PamConfigError> {
        // From the Linux-PAM manual page:
        //
        // > This dynamic configuration is set by the contents of the single Linux-PAM
        // > configuration file /etc/pam.conf. Alternatively, the configuration can be set by
        // > individual configuration files located in the /etc/pam.d/ directory. The presence of
        // > this directory will cause Linux-PAM to ignore /etc/pam.conf.

        let pam_d_path = Path::new(PAM_D_PATH);
        if pam_d_path.try_exists()? && pam_d_path.is_dir() {
            Self::from_dir(fs::read_dir(pam_d_path)?)
        } else {
            Self::from_file(PAM_CONF_PATH)
        }
    }
}

impl FromStr for PamConfig {
    type Err = PamConfigSyntaxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let packed_rules: Vec<Result<(String, PamRule), PamConfigSyntaxError>> =
            PamRule::packed_iter(s).collect();

        // Form a Map from "Service Name" --> Vec<PamRule> 
        let mut services: BTreeMap<String, Vec<PamRule>> = BTreeMap::new();
        for packed_rule in packed_rules.into_iter() {
            let (service_name, rule) = packed_rule?;

            if let Some(rules) = services.get_mut(&service_name) {
                rules.push(rule);
            } else {
                services.insert(service_name, vec![rule]);
            }
        }

        let services = services
            .into_iter()
            .map(|(name, rules)| PamService { name, rules })
            .collect();

        Ok(Self { services })
    }
}

impl PamService {
    /// Get the name of the service
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the rules belonging to the current service
    pub fn rules(&self) -> &[PamRule] {
        &self.rules
    }
    
    /// Append a rule to the rules list
    pub fn push(&mut self, rule: PamRule) {
        self.rules.push(rule)
    }

    /// Create a new [`PamService`]
    pub fn new(name: &str, rules: Vec<PamRule>) -> Self {
        let name = String::from(name);
        Self { name, rules }
    }

    /// Read a [`PamService`] from a service file
    ///
    /// The file-name is used as the service name. On Linux, this is usually a file in the
    /// `/etc/pam.d` directory. To parse the `/etc/pam.conf` file look at [`PamConfig::from_file`]. 
    pub fn from_file(path: impl AsRef<Path>) -> Result<PamService, PamConfigError> {
        use PamConfigError::{NonUTF8Filename, NotAFilename};

        let path = path.as_ref();

        let mut file = File::open(path)?;
        let name = path
            .file_name()
            .ok_or(NotAFilename)?
            .to_str()
            .ok_or(NonUTF8Filename)?
            .to_string();

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Parse the file contents
        let rules = Self::from_str(&contents)?;

        Ok(Self { name, rules })
    }

    /// Read a vector of [`PamRule`]s from a [`&str`]
    ///
    /// This is read in the separated service file syntax. This can then be used to create a
    /// [`PamService`].
    ///
    /// # Examples
    ///
    /// ```
    /// let service_file = r#"
    /// #%PAM-1.0
    /// 
    /// auth       required     pam_securetty.so
    /// auth       requisite    pam_nologin.so
    /// auth       include      system-local-login
    /// account    include      system-local-login
    /// session    include      system-local-login
    /// password   include      system-local-login
    /// "#;
    ///
    /// let rules = PamService::from_str(service_file)?;
    /// let service = PamService::new("login", rules);
    /// ```
    pub fn from_str(s: &str) -> Result<Vec<PamRule>, PamConfigSyntaxError> {
        PamRule::separated_iter(s).collect()
    }
}

impl PamRule {
    /// Get whether to log for this rule or not
    pub fn is_logging_enabled(&self) -> bool {
        self.is_logging_enabled
    }

    /// Get the [`Domain`]
    ///
    /// In the PAM configuration this can have one of 4 values: 'account', 'auth', 'password' or
    /// 'session'
    pub fn domain(&self) -> Domain {
        self.domain
    }

    /// Get the control parameters
    pub fn control(&self) -> &Control {
        &self.control
    }

    /// Get the module path
    pub fn module_path(&self) -> &ModulePath {
        &self.module_path
    }

    /// Get the module argument 
    pub fn module_arguments(&self) -> &[ModuleArgument] {
        &self.module_arguments
    }

    /// Create an iterator over rules in the packed configuration format
    ///
    /// The packed configuration format is the same format as used in `/etc/pam.conf`.
    pub fn packed_iter(s: &str) -> PackedRuleIterator {
        PackedRuleIterator(s)
    }

    /// Create an iterator over rules in the separated configuration format
    ///
    /// The separated configuration format is the same format as used in `/etc/pam.d`.
    pub fn separated_iter(s: &str) -> SeparatedRuleIterator {
        SeparatedRuleIterator(s)
    }
}

/// Iterator over the packed configuration format
///
/// # Packed Syntax
///
/// This format is used within the `/etc/pam.conf` file and has the following line by line syntax
///
/// ```text
/// service     domain      control     module      module-arguments
/// ```
///
/// * `service`: [`PamService::name`]
/// * `domain`: [`Domain`]
/// * `control`: [`Control`]
/// * `module`: [`ModulePath`]
/// * `module-arguments`: multiple [`ModuleArgument`]s
///
/// Lines that only contain spaces and tabs or lines that start with `#` are ignored. 
///
/// # Note
///
/// If a error is found on a line, the iterator will move to the next line. The iterator is stopped
/// when the end of file (`EOF`) is reached.
pub struct PackedRuleIterator<'a>(&'a str);

/// Iterator over the separated configuration format
///
/// # Separated Syntax
///
/// This format is used within the `/etc/pam.d` file and has the following line by line syntax
///
/// ```text
/// domain      control     module      module-arguments
/// ```
///
/// * `domain`: [`Domain`]
/// * `control`: [`Control`]
/// * `module`: [`ModulePath`]
/// * `module-arguments`: multiple [`ModuleArgument`]s
///
/// Lines that only contain spaces and tabs or lines that start with `#` are ignored. 
///
/// # Note
///
/// If a error is found on a line, the iterator will move to the next line. The iterator is stopped
/// when the end of file (`EOF`) is reached.
pub struct SeparatedRuleIterator<'a>(&'a str);

impl<'a> Iterator for PackedRuleIterator<'a> {
    type Item = Result<(String, PamRule), PamConfigSyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self(s) = self;
        let (s, escaped_line) = take_escaped_line(s)?;

        self.0 = s;

        Some(take_packed_rule(&escaped_line))
    }
}

impl<'a> Iterator for SeparatedRuleIterator<'a> {
    type Item = Result<PamRule, PamConfigSyntaxError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self(s) = self;
        let (s, escaped_line) = take_escaped_line(s)?;

        self.0 = s;

        Some(take_separated_rule(&escaped_line))
    }
}

fn take_packed_rule(escaped_line: &str) -> Result<(String, PamRule), PamConfigSyntaxError> {
    let (escaped_line, service_name) = take_service_name(escaped_line)?;
    let (escaped_line, _) = skip_whitespace(escaped_line);
    let rule = take_separated_rule(escaped_line)?;

    let service_name = String::from(service_name);

    Ok((service_name, rule))
}

fn take_separated_rule(escaped_line: &str) -> Result<PamRule, PamConfigSyntaxError> {
    let (escaped_line, domain, is_logging_enabled) = take_domain(escaped_line)?;
    let (escaped_line, _) = skip_whitespace(escaped_line);
    let (escaped_line, control) = take_control(escaped_line)?;
    let (escaped_line, _) = skip_whitespace(escaped_line);
    let (escaped_line, module_path) = take_module_path(escaped_line)?;
    let (escaped_line, _) = skip_whitespace(escaped_line);
    let module_arguments = take_module_arguments(escaped_line)?;

    Ok(PamRule {
        domain,
        is_logging_enabled,
        control,
        module_path,
        module_arguments,
    })
}

fn take_service_name(s: &str) -> Result<(&str, Cow<str>), PamConfigSyntaxError> {
    let (service_name, after) = take_string(s).ok_or(PamConfigSyntaxError::UnclosedBracket)?;
    let s = &s[after..];
    Ok((s, service_name))
}

fn take_domain(s: &str) -> Result<(&str, Domain, bool), PamConfigSyntaxError> {
    // Take group ('account', 'auth', 'password', 'session')
    let (domain, after) = till_whitespace(s);
    let s = &s[after..];

    // We disable logging if the '-' is prepended
    let is_logging_enabled = !domain.starts_with('-');
    let domain = &domain[if is_logging_enabled { 0 } else { 1 }..];
    let domain = Domain::from_str(domain)
        .map_err(|_| PamConfigSyntaxError::WrongDomain(domain.to_string()))?;

    Ok((s, domain, is_logging_enabled))
}

fn take_control(s: &str) -> Result<(&str, Control), PamConfigSyntaxError> {
    // Take control parameters
    let (control, after) = take_control_string(s).ok_or(PamConfigSyntaxError::UnclosedBracket)?;
    let control = Control::from_str(control).map_err(|e| PamConfigSyntaxError::WrongControl(e))?;
    let s = &s[after..];

    Ok((s, control))
}

fn take_module_path(s: &str) -> Result<(&str, ModulePath), PamConfigSyntaxError> {
    // Take module path
    let (module_path, after) = till_whitespace(s);
    let s = &s[after..];

    let module_path = ModulePath::from_str(module_path)
        .map_err(|_| PamConfigSyntaxError::WrongModulePath(module_path.to_string()))?;

    Ok((s, module_path))
}

fn take_module_arguments(s: &str) -> Result<Vec<ModuleArgument>, PamConfigSyntaxError> {
    // Take module arguments. This is the rest of list and basically is an env key-value pair.
    let module_arguments = if s.is_empty() {
        Vec::new()
    } else {
        take_all_strings(s)
            .ok_or(PamConfigSyntaxError::UnclosedBracket)?
            .into_iter()
            .map(|s| {
                ModuleArgument::from_str(&s)
                    .map_err(|_| PamConfigSyntaxError::WrongModuleArgs(s.to_string()))
            })
            .collect::<Result<Vec<ModuleArgument>, PamConfigSyntaxError>>()?
    };

    Ok(module_arguments)
}

fn take_escaped_line(mut s: &str) -> Option<(&str, Cow<str>)> {
    loop {
        if s.is_empty() {
            return None;
        }

        let (line, leftover) = till_end_of_line(&s);
        s = &s[leftover..];

        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        break Some((s, line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_finder() {
        let pam_conf = r#"
%PAM-1.1

auth    required    pam-login.so    password=xafalkjz    cool=dslkfajlyes
account    requisite    pam-dalkfj.so    pxxxassword=xyz   [axy=salfjdsaljf sdalkfjlsakdsa  dsakfj]  coodlksafjl=yes
-password    required    saldjfl.so    passwyyyord=xyz    cool=jlksadjfyes
"#;
        let entry_stream = PamRule::separated_iter(pam_conf);

        let entries = entry_stream
            .collect::<Result<Vec<PamRule>, PamConfigSyntaxError>>()
            .unwrap();

        println!("{:#?}", entries);
        assert!(false);
    }

    // pam.conf
    // service      type        control         module-path     module-arguments
    //
    // pam.d/*
    // type         control     module-path     module-arguments
}
