use pamoxide::PamConfig;

fn main() {
    let config = PamConfig::from_system().expect("Failed to parse pam configuration");

    for service in config.services() {
        println!("Service '{}':", service.name());
        for rule in service.rules() {
            println!("    '{}' '{}'", rule.domain(), rule.module_path());
        }
    }
}
