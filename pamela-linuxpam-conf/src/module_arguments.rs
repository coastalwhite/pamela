use std::fmt::Display;

#[derive(Debug)]
pub struct ModuleArguments(Vec<String>);

impl ModuleArguments {
    pub fn new(items: Vec<String>) -> Self {
        // TODO: Perform some checking
        Self(items)
    }
}


impl Display for ModuleArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
