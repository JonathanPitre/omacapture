use std::path::PathBuf;
use std::process::{Command, Output};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "omacapture-keybinds-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn run(&self, args: &[&str]) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_omacapture"))
            .arg("keybinds")
            .args(args)
            .env("HOME", &self.0)
            .env("XDG_CONFIG_HOME", self.0.join(".config"))
            .env("XDG_DATA_HOME", self.0.join(".local/share"))
            .env_remove("HYPRLAND_INSTANCE_SIGNATURE")
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        output
    }

    fn config(&self) -> toml::Value {
        toml::from_str(&std::fs::read_to_string(self.0.join(".config/omacapture/config.toml")).unwrap()).unwrap()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn remove_clears_saved_preset_when_managed_block_is_absent() {
    for user_bindings in [None, Some("o.bind(\"SUPER + B\", \"Browser\", \"chromium\")\n")] {
        let home = Scratch::new();
        let file = home.0.join("bindings.lua");
        let path = file.to_str().unwrap();
        home.run(&["install", "print", "--force", "--file", path, "--no-reload"]);
        assert_eq!(home.config()["keybinds"]["preset"].as_str(), Some("print"));
        match user_bindings {
            Some(text) => std::fs::write(&file, text).unwrap(),
            None => std::fs::remove_file(&file).unwrap(),
        }
        home.run(&["remove", "--file", path, "--no-reload"]);
        assert!(home.config().get("keybinds").and_then(|k| k.get("preset")).is_none());
        home.run(&["restore", "--file", path, "--no-reload"]);
        assert_eq!(std::fs::read_to_string(&file).ok().as_deref(), user_bindings);
    }
}
