# wleave

![AUR version](https://img.shields.io/aur/version/wleave-git)
![GitHub](https://img.shields.io/github/license/AMNatty/wleave)

A Wayland layer-shell logout prompt, now ported to GTK4!

A fork of [wlogout](https://github.com/ArtsyMacaw/wlogout) with a bunch of quality-of-life features.

![The default Wleave menu look](/example.png)

## Installation

### Arch Linux

**wleave** can be installed from the **AUR**:

```shell
paru -S wleave-git
```

### Building from sources

Dependencies:

- gtk4-layer-shell
- gtk4
- librsvg (for SVG images)
- libadwaita
- a stable version of the Rust toolchain

You can run the application using `cargo run --release` or GNU make:

```shell
make
./target/release/wleave
```

## Usage

The command line options are backwards-compatible with **wlogout**.
See `--help` for a list of options.

### Help, how do I close the menu?

The `<Esc>` key closes the menu, an option to change this may be added eventually.

## Configuration

**wleave** is backwards-compatible with **wlogout** configuration files.

Since **version 0.6.0**, *full JSON configuration* can be used in place of the `wlogout`-based
configuration. The default configuration file can be copied from `/etc/wleave/layout.json`.
The new configuration system is more flexible as it removes the need for extra command-line
arguments.

*Example configuration* with one button that executes `swaylock` on click:

```json
{
    "margin": 200,
    "buttons-per-row": "1/1",
    "delay-command-ms": 100,
    "close-on-lost-focus": true,
    "show-keybinds": true,
    "buttons": [
        {
            "label": "lock",
            "action": "swaylock",
            "text": "Lock",
            "keybind": "l",
            "icon": "/usr/share/wleave/icons/lock.svg"
        }
    ]
}
```

Layout files may also be read from *stdin* with `--layout -`.
For example, with `jq`, buttons can be picked out:

```shell
$ jq '.buttons[] |= select([.label] | inside(["lock", "logout"]))' layout.json | wleave --layout -
```

## Styling

By default, `wleave` follows `libadwaita` colors and uses CSS variables.
This allows following the system light/dark theme preference from GNOME settings.
In other desktop environments, this may be changed with
`gsettings set org.gnome.desktop.interface color-scheme "'prefer-dark'"` or
`gsettings set org.gnome.desktop.interface color-scheme "'prefer-light'"` correspondingly.

The stylesheet in `/etc/wleave/style.css` is fully customizable and can be edited.

### Colorized icons

Icon colors are *disabled by default* as they may be hard to read for light theme users.

Each button has an identifier set in the layout file, which allows custom-styling each button
one-by-one.

Uncomment the following lines in the `/etc/wleave/style.css` CSS stylesheet to enable them:

```css
/*
 ... snip ...
 */

button#shutdown {
    color: #ff8d8d;
}

button#hibernate {
    color: #a8c0ff;
}

button#reboot {
    color: #84ffaa;
}

button#lock {
    color: #f9e2af;
}

button#logout {
    color: #f9c5af;
}

button#suspend {
    color: #caaff9;
}

/*
 ... snip ...
 */
```

## Keybinds reference

See <https://gitlab.gnome.org/GNOME/gtk/-/blob/4.18.0/gdk/keynames.txt> for a list of valid keybinds.

## Enhancements

- SVG icons can be colorized via CSS `color`
- Libadwaita accent colors
- Automatic light theme by default since 0.6
- Natively GTK4 since version 0.5
- New pretty icons by [@earth-walker](https://github.com/earth-walker)
- Autoclose when window focus is lost (the `-f/--close-on-lost-focus` flag)
- Mnemonic labels (the `-k/--show-keybinds` flag)
- Pretty gaps by default
- Less error-prone
- Keybinds accept modifier keys and Unicode characters
- Easier to extend
