# Swayboard

Sway keyboard's layout daemon.
This utility helps you automatically change a keyboard layout based on per-window policy.

## Installation
```bash
cargo install swayboard
```
Then you can add swayboard to your sway config:
```
exec swayboard
```

## Configuration

Swayboard supports next configuration locations 
 - `/etc/swayboard/config.toml` 
 - `~/.config/swayboard/config.toml`

```toml
[logging]
level = "Info"

[device]
identifier = "1:1:AT_Translated_Set_2_keyboard"
```

Your device identifier you can check with swaymsg
```bash
swaymsg -t get_inputs -r | jq '.[] | select(.type | contains("keyboard")).identifier'
```
