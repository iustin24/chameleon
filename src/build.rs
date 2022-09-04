use std::process::Command;

fn main() {
    Command::new("sh").arg("-c").arg("mkdir -p ~/.config/chameleon/ && mv -n config.toml ~/.config/chameleon/; mv -n ./wordlists/ ~/.config/chameleon/").status().unwrap();
}
