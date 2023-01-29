mod control_flag;
mod facility;
mod module_arguments;
mod module_path;
mod parse;
mod return_code;
mod service;

use std::str::FromStr;

pub use control_flag::ControlFlag;
pub use facility::Facility;
pub use module_arguments::ModuleArguments;
pub use module_path::ModulePath;
pub use parse::{RuleToken, RuleTokenContent};
pub use service::Service;

use self::parse::RuleTokenIterator;

#[derive(Debug)]
pub struct Rule {
    do_log: bool,
    service: Service,
    facility: Facility,
    content: RuleContent,
}

#[derive(Debug)]
pub enum RuleContent {
    ServiceInclusion {
        method: InclusionMethod,
        service: Service,
    },
    Entry {
        control_flag: ControlFlag,
        module_path: ModulePath,
        module_arguments: ModuleArguments,
    },
}

#[derive(Debug, PartialEq)]
pub enum InclusionMethod {
    Include,
    Substack,
}

pub struct RuleIterator<'a> {
    rule_token_iterator: RuleTokenIterator<'a>,
}

impl Rule {
    pub fn do_log(&self) -> bool {
        self.do_log
    }

    pub fn service(&self) -> &str {
        self.service.as_ref()
    }

    pub fn facility(&self) -> Facility {
        self.facility
    }

    pub fn content(&self) -> &RuleContent {
        &self.content
    }
}

impl<'a> RuleIterator<'a> {
    pub fn with_service(source: &'a str, service: &'a str) -> Self {
        Self {
            rule_token_iterator: RuleTokenIterator::with_service(source, service),
        }
    }

    pub fn new(source: &'a str) -> Self {
        Self {
            rule_token_iterator: RuleTokenIterator::new(source),
        }
    }
}

impl<'a> Iterator for RuleIterator<'a> {
    type Item = Result<Rule, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        let rule_token = self.rule_token_iterator.next()?;

        let do_log = rule_token.do_log;
        let service = Service::try_from(rule_token.service).unwrap();
        let facility = Facility::from_str(&rule_token.facility).unwrap();

        let content = match rule_token.content {
            RuleTokenContent::ServiceInclusion { method, service } => {
                let service = Service::try_from(service).unwrap();
                RuleContent::ServiceInclusion { method, service }
            }
            RuleTokenContent::Entry {
                control_flag,
                module_path,
                module_arguments,
            } => {
                let control_flag = ControlFlag::from_str(&control_flag).unwrap();
                let module_path = ModulePath::from_str(&module_path).unwrap();
                let module_arguments = ModuleArguments::new(module_arguments);
                RuleContent::Entry {
                    control_flag,
                    module_path,
                    module_arguments,
                }
            }
        };

        Some(Ok(Rule {
            do_log,
            service,
            facility,
            content,
        }))
    }
}

impl FromStr for InclusionMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "include" => Ok(Self::Include),
            "substack" => Ok(Self::Substack),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    macro_rules! file_test {
        ($file_content:literal, $($service:literal, )? $name:ident) => {
            #[test]
            fn $name() {
                let file_content = $file_content;

                #[allow(unused_variables)]
                let rule_iterator = RuleIterator::new(file_content);
                $(
                let rule_iterator = RuleIterator::with_service(file_content, $service);
                )?

                let rules = rule_iterator.collect::<Result<Vec<Rule>, ()>>().unwrap();

                eprintln!("{:?}", file_content);

                insta::assert_debug_snapshot!(rules);
            }
        };
    }

    file_test!(
        r#"#%PAM-1.0
auth		sufficient	pam_rootok.so
auth		required	pam_console.so
#auth		include		system-auth
account		required	pam_permit.so"#,
        "reboot",
        int_test_reboot
    );
}
