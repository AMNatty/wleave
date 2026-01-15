wleave(1)

# NAME

wleave - A Wayland logout menu

# SYNOPSIS

*wleave* [options] [command]

# OPTIONS

*-h, --help*
	Show help message and stop

*-l, --layout* <layout>
	Specify a custom layout file, specify - to read the config from stdin

*-v, --version*
	Show version number and stop

*-C, -css* <css>
	Specify a custom css file

*-b, --buttons-per-row* <num>
	Set the number of buttons per row, or use a fraction to specify the number of rows to be used (e.g. "1/1" for all buttons in a single row, "1/5" to distribute the buttons over 5 rows)

*-c, --column-spacing* <space>
	Set space between buttons columns

*-r, --row-spacing* <space>
	Set space between buttons rows

*-m, --margin* <padding>
	Set margin on all sides

*-L, --margin-left* <padding>
	Set margin for left of buttons

*-R, --margin-right* <padding>
	Set margin for right of buttons

*-T, --margin-top* <padding>
	Set margin for top of buttons

*-B, --margin-bottom* <padding>
	Set margin for bottom of buttons

*-A, --button-aspect-ratio* <ratio>
	Set the aspect ratio of the buttons, either as a float (as a number or string) or a ratio (e.g. "5/4").
	If unspecified, the buttons fill all available space between the margins.

*-f, --close-on-lost-focus*
	Closes the menu if focus is lost

*-k, --show-keybinds*
	Show the associated key binds for each button

*-p, --protocol* <protocol>
	Takes layer-shell, xdg, or none.
	The "layer-shell" allows transparency effects; however, only a few compositors correctly support it.
	The "xdg" protocol will work on almost all compositors, but does not allow for transparency.
	The "none" option shows the window as is, without full-screening it

*-d, --delay-command-ms* <number>
	The number of milliseconds to wait after an action before the associated command is executed

*-x, --no-version-info*
	Hide the version label

# DESCRIPTION

wleave is a Wayland-native logout script. It is a modern rewrite of Wlogout and a drop-in replacement.

# CONFIGURATION

wleave searches for a layout and style.css file in the following locations, in this order:

. $XDG_CONFIG_HOME/wleave/
. $XDG_CONFIG_HOME/wlogout/
. /etc/wleave/
. /etc/wlogout/
. /usr/local/etc/wleave
. /usr/local/etc/wlogout

If unset, $XDG_CONFIG_HOME defaults to *~/.config/*.

An error is raised when no layout file is found; However, the style.css file is optional. If you would like to customise either it is recommended that you copy the defaults from */etc/wleave/* into  *~/.config* and make any changes there.

# AUTHORS

Based on Wlogout by Haden Collins <collinshaden@gmail.com>. For more information about wlogout, see <https://github.com/ArtsyMacaw/wlogout>.
Rewrite by Natty <natty.sh.git@gmail.com>, see <https://github.com/AMNatty/wleave>.

# SEE ALSO

*wleave*(5)
