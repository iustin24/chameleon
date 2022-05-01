use std::process::Command;

fn main() {
    Command::new("sh").arg("-c").arg("mkdir -p ~/.config/content/ && mv config.toml ~/.config/content/; mv ./wordlists/ ~/.config/content/").status().unwrap();
}
