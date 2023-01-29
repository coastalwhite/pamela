use super::InclusionMethod;

struct LineIterator<'a> {
    source: &'a str,
    current_offset: usize,
}

struct StringTokenIterator<'a> {
    source: &'a str,
    current_offset: usize,
}

pub struct RuleTokenIterator<'a> {
    known_service: Option<&'a str>,
    line_iterator: LineIterator<'a>,
}

#[derive(Debug, PartialEq)]
pub struct RuleToken {
    pub do_log: bool,
    pub service: String,
    pub facility: String,
    pub content: RuleTokenContent,
}

#[derive(Debug, PartialEq)]
pub enum RuleTokenContent {
    ServiceInclusion {
        method: InclusionMethod,
        service: String,
    },
    Entry {
        control_flag: String,
        module_path: String,
        module_arguments: Vec<String>,
    },
}


impl<'a> LineIterator<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            current_offset: 0,
        }
    }
}

impl<'a> StringTokenIterator<'a> {
    fn new(source: &'a str) -> Self {
        debug_assert!(!source.contains('\n'));
        Self {
            source,
            current_offset: 0,
        }
    }
}

impl<'a> RuleTokenIterator<'a> {
    pub fn with_service(source: &'a str, service: &'a str) -> Self {
        Self {
            known_service: Some(service),
            line_iterator: LineIterator::new(source),
        }
    }

    pub fn new(source: &'a str) -> Self {
        Self {
            known_service: None,
            line_iterator: LineIterator::new(source),
        }
    }
}

fn trim_start_with_length(source: &str) -> (&str, usize) {
    for (i, c) in source.char_indices() {
        match c {
            ' ' | '\t' | '\n' => {}
            _ => return (&source[i..], i),
        }
    }

    return ("", source.len());
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = String::new();

        loop {
            let source = &self.source[self.current_offset..];
            let (source, length) = trim_start_with_length(source);
            self.current_offset += length;

            // Two notes here:
            // 1. We are using UTF-8 find, so that is different from Linux-PAM
            // 2. Similar to Linux-PAM, we require a newline at the end of the file
            let Some(newline_position) = source.find('\n') else {
                // No new line found. Skip to the end.
                // TODO: add some debug output to say that this is probably a mistake on the
                // configuration part.
                self.current_offset += source.len();
                if buffer.is_empty() && source.is_empty() {
                    return None;
                }

                if let Some(comment_position) = source.find('#') {
                    if comment_position == 0 {
                        return None;
                    }

                    buffer.push_str(&source[..comment_position]);
                } else {
                    buffer.push_str(source);
                }

                return Some(buffer);
            };

            self.current_offset += newline_position + 1;

            // If the entire line is a comment.
            if source.starts_with('#') {
                continue;
            }

            let line = &source[..newline_position];
            if let Some(comment_position) = line.find('#') {
                // If we find a comment, we cannot extend the line any more
                buffer.push_str(&line[..comment_position]);
                return Some(buffer);
            }

            // If there is nothing extending the line, return the buffer with this line.
            if !line.ends_with('\\') {
                buffer.push_str(line);
                return Some(buffer);
            }

            // Line is being extended. Replace the escaped newline with a ' ' and continue adding
            // lines.
            buffer.push_str(&line[..line.len() - 1]);
            buffer.push(' ');
        }
    }
}

impl<'a> Iterator for StringTokenIterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let source = &self.source[self.current_offset..];

        let (source, length) = trim_start_with_length(source);
        self.current_offset += length;

        if source.is_empty() {
            return None;
        }

        if source.starts_with('[') {
            let mut is_escaped = false;
            let mut last_copy = 1;
            let mut buffer = String::new();

            for (i, c) in source.char_indices() {
                if !is_escaped && c == ']' {
                    self.current_offset += i + 1;
                    buffer.push_str(&source[last_copy..i]);
                    return Some(buffer);
                }

                if is_escaped && c == ']' {
                    buffer.push_str(&source[last_copy..i - 1]);
                    last_copy = i;
                }

                is_escaped = c == '\\';
            }

            Some(source.to_string())
        } else {
            let whitespace = source.find(&[' ', '\t', '\n']).unwrap_or(source.len());
            self.current_offset += whitespace;
            Some(source[..whitespace].to_string())
        }
    }
}

impl<'a> Iterator for RuleTokenIterator<'a> {
    type Item = RuleToken;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.line_iterator.next()?;

        let do_log = !line.starts_with('-');
        let line = if line.starts_with('-') {
            &line[1..]
        } else {
            &line[..]
        };
        let mut str_tokens = StringTokenIterator::new(line);

        let service = if let Some(known_service) = self.known_service {
            known_service.to_string()
        } else {
            str_tokens.next()?
        };
        let facility = str_tokens.next()?;
        let control_flag = str_tokens.next()?;
        let module_path = str_tokens.next()?;

        let content = match &control_flag[..] {
            "include" => RuleTokenContent::ServiceInclusion {
                method: InclusionMethod::Include,
                service: module_path,
            },
            "substack" => RuleTokenContent::ServiceInclusion {
                method: InclusionMethod::Substack,
                service: module_path,
            },
            _ => RuleTokenContent::Entry {
                control_flag,
                module_path,
                module_arguments: str_tokens.collect(),
            },
        };

        Some(RuleToken {
            do_log,
            service,
            facility,
            content,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_iterator() {
        macro_rules! assert_line {
            ($source:literal, [$($line:literal),* $(,)?]) => {
                let line_iterator = LineIterator::new($source);
                let correct_lines: &[&str] = &[$($line),*];

                let found_lines = line_iterator.collect::<Vec<String>>();

                assert_eq!(found_lines.len(), correct_lines.len());

                for (correct, found) in correct_lines.into_iter().zip(found_lines.into_iter()) {
                    assert_eq!(found, *correct);
                }
            };
        }

        assert_line!("", []);
        assert_line!("\n\n\n\n\n", []);
        assert_line!("#012345678", []);

        assert_line!("a", ["a"]);
        assert_line!("a\nb", ["a", "b"]);
        assert_line!("a\\\nb", ["a b"]);
        assert_line!("a\n#b\nc", ["a", "c"]);
        assert_line!("a\n#b\nc#d", ["a", "c"]);

        assert_line!("abc\\ndef", ["abc\\ndef"]);
        assert_line!("abc\\\ndef", ["abc def"]);
    }

    #[test]
    fn string_token_iterator() {
        macro_rules! assert_str_token {
            ($source:literal, [$($line:literal),* $(,)?]) => {
                let str_token_iterator = StringTokenIterator::new($source);
                let correct_lines: &[&str] = &[$($line),*];

                let found_lines = str_token_iterator.collect::<Vec<String>>();

                assert_eq!(found_lines.len(), correct_lines.len());

                for (correct, found) in correct_lines.into_iter().zip(found_lines.into_iter()) {
                    assert_eq!(found, *correct);
                }
            };
        }

        assert_str_token!("", []);
        assert_str_token!("a", ["a"]);
        assert_str_token!("a b", ["a", "b"]);
        assert_str_token!("[a b]", ["a b"]);
        assert_str_token!("[a b] c", ["a b", "c"]);
        assert_str_token!("[a \\]b]", ["a ]b"]);
        assert_str_token!("[... [ ... \\] ...]", ["... [ ... ] ..."]);
    }

    #[test]
    fn rule_iterator() {
        macro_rules! assert_rules {
            ($source:literal$(+ ($service:literal))?, [$($rule:expr),* $(,)?]) => {
                let rule_iterator = RuleTokenIterator::new($source);
                $(let rule_iterator = RuleTokenIterator::with_service($source, $service);)?
                let correct_lines: &[RuleToken] = &[$($rule),*];

                let found_lines = rule_iterator.collect::<Vec<RuleToken>>();

                assert_eq!(found_lines.len(), correct_lines.len());

                for (correct, found) in correct_lines.into_iter().zip(found_lines.into_iter()) {
                    assert_eq!(found, *correct);
                }
            };
        }

        assert_rules!("", []);
        assert_rules!(
            "login auth required pam_unix.so",
            [RuleToken {
                do_log: true,
                service: "login".to_string(),
                facility: "auth".to_string(),
                content: RuleTokenContent::Entry {
                    control_flag: "required".to_string(),
                    module_path: "pam_unix.so".to_string(),
                    module_arguments: Vec::new()
                }
            }]
        );
        assert_rules!(
            "login auth include common-auth",
            [RuleToken {
                do_log: true,
                service: "login".to_string(),
                facility: "auth".to_string(),
                content: RuleTokenContent::ServiceInclusion {
                    method: InclusionMethod::Include,
                    service: "common-auth".to_string(),
                }
            }]
        );
    }

    macro_rules! file_test {
        ($file_content:literal, $($service:literal, )? $name:ident) => {
            #[test]
            fn $name() {
                let file_content = $file_content;

                #[allow(unused_variables)]
                let rule_iterator = RuleTokenIterator::new(file_content);
                $(
                let rule_iterator = RuleTokenIterator::with_service(file_content, $service);
                )?

                let rules = rule_iterator.collect::<Vec<RuleToken>>();

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

    file_test!(
        r#"#%PAM-1.0
auth        include     system-auth
account     include     system-auth
session     include     system-auth"#,
        "sudo",
        int_test_sudo_arch
    );

    file_test!(
        r#"#%PAM-1.0
auth        sufficient  pam_rootok.so
auth        required    pam_unix.so
account     required    pam_unix.so
session     required    pam_unix.so
password    required    pam_permit.so"#,
        "usermod",
        int_test_user_mod_arch
    );

    file_test!(
        r#"#%PAM-1.0

auth       required                    pam_faillock.so      preauth
# Optionally use requisite above if you do not want to prompt for the password
# on locked accounts.
-auth      [success=2 default=ignore]  pam_systemd_home.so
auth       [success=1 default=bad]     pam_unix.so          try_first_pass nullok
auth       [default=die]               pam_faillock.so      authfail
auth       optional                    pam_permit.so
auth       required                    pam_env.so
auth       required                    pam_faillock.so      authsucc
# If you drop the above call to pam_faillock.so the lock will be done also
# on non-consecutive authentication failures.

-account   [success=1 default=ignore]  pam_systemd_home.so
account    required                    pam_unix.so
account    optional                    pam_permit.so
account    required                    pam_time.so

-password  [success=1 default=ignore]  pam_systemd_home.so
password   required                    pam_unix.so          try_first_pass nullok shadow sha512
password   optional                    pam_permit.so

-session   optional                    pam_systemd_home.so
session    required                    pam_limits.so
session    required                    pam_unix.so
session    optional                    pam_permit.so"#,
        "system-auth",
        int_test_system_auth_arch
    );
}