# Windowsorter

Minimal Rust prototype that decides whether to move windows between workspaces based on simple rules.

## Configuration

Windowsorter reads rules from a TOML file located at `$XDG_CONFIG_HOME/windowsorter/config.toml`. If
`$XDG_CONFIG_HOME` is unset the program falls back to `~/.config/windowsorter/config.toml`.

Example `config.toml`:

```toml
[[app]]
name = "browsers"
classes = ["firefox", "chromium", "brave", "chrome"]
default_workspace = 1
mandatory_workspace = 1

[[app]]
name = "terminals"
classes = ["alacritty", "kitty", "wezterm", "gnome-terminal", "foot"]
default_workspace = 2
forbidden = [1, 8, 9]
```

## Usage

- By default the program performs an initial scan of existing windows and applies configured rules.
- To skip the initial scan, run the program with `--no-initial-scan`.
- CLI flags take precedence over configuration file behavior where applicable.

## License & origin

This project is licensed under the GPLv3 License (see `LICENSE`), mostly to satisfy the requirement of the Hyprland crate.
This code was written as an experiment in using LLMs for programming. The code was checked for copyright violations.
Because of the simplicity of the code, I am fairly certain that there is no violation of any copyright. However if you disagree, 
please leave an issue on this repository.

## Maintenance status

This project is passively maintained. If you have a feature you would like to see, feel free to leave a PR or fork this project. My experience with "vibe" coding a project like this has been prodominantly negative and the usage of LLMs in this project will probably be reduced to "fancy" autocomplete until the tools have substancially improved.