use std::process::Command;

fn main() {
    Command::new("sh").arg("-c").arg("mkdir -p ~/.config/chameleon/ && mv config.toml ~/.config/chameleon/; mv ./wordlists/ ~/.config/chameleon/").status().unwrap();
}
