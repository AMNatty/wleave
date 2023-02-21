# wleave
A Wayland-native logout script written in Gtk3

Basically a fork of [wlogout](https://github.com/ArtsyMacaw/wlogout), rewritten in Rust.

**wleave** is compatible with **wlogout** configuration files.

## Usage

The command line options are identical to **wlogout**.
See `--help` for a list of options.

## Help, how do I close the menu

The `<Esc>` key closes the menu, an option to change this may be added eventually.

## Keybinds reference

See <https://gitlab.gnome.org/GNOME/gtk/-/blob/gtk-3-24/gdk/keynames.txt> for a list of valid keybinds.

## Enhancements

* Autoclose on when window focus is lost (the `-f/--close-on-lost-focus` flag)
* Pretty gaps by default
* Less error-prone
* Keybinds accept modifier keys and Unicode characters
* Easier to extend
