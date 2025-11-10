wleave.json(5)

# NAME

wleave.json - layout JSON file and configuration options

# TOP-LEVEL-OPTIONS

The top-level options directly correspond to their long-version command-line option counter parts.
Button layout is specified in the extra *"button"* field.

*"buttons"*: <array of buttons>
	See section *LAYOUT* for structure

*"css"*: <string>
	Specify a custom CSS file instead of the default one

*"buttons-per-row"*: <string>
	Set the number of buttons per row, or use a fraction to specify the number of rows to be used
	(e.g. "1/1" for all buttons in a single row, "1/5" to distribute the buttons over 5 rows) ++
*Default*: "3"

*"column-spacing"*: <number>
	Set space between buttons columns ++
*Default*: 8

*"row-spacing"*: <number>
	Set space between buttons rows ++
*Default*: 8

*"margin"*: <number>
	Set margin on all sides ++
*Default*: 200

*"margin-left"* <number>
	Set margin for left of buttons ++
Falls back to the value set by *margin*

*"margin-right"* <number>
	Set margin for right of buttons ++
Falls back to the value set by *margin*

*"margin-top"* <number>
	Set margin for top of buttons ++
Falls back to the value set by *margin*

*"margin-bottom"* <number>
	Set margin for bottom of buttons ++
Falls back to the value set by *margin*

*"button-aspect-ratio"* <string or number>
	Set the aspect ratio of the buttons, either as a float (as a number or string) or a ratio (e.g. "5/4"). ++
If unspecified, the buttons fill all available space between the margins.

*"close-on-lost-focus"*: <*true*/*false*>
	Closes the menu if focus is lost ++
*Default*: false

*"show-keybinds"*: <*true*/*false*>
	Show the associated key binds for each button ++
*Default*: false

*"protocol"*: <*"layer-shell"*/*"xdg"*>
	The layer-shell protocol is the intended protocol to be used with wleave, however it requires compositor support ++
*Default*: "layer-shell"

*"no-version-info"*: <*true*/*false*>
	Hides the version label ++
*Default*: false

*"delay-command-ms"*: <number>
	The number of milliseconds to wait after an action before the associated command is executed ++
*Default*: 100

# LAYOUT

wleave buttons may have the following properties:

- *"label"* (string) - internal name of the button that can be used in CSS as an identifier
- *"action"* (command string) - the action that will be executed via shell upon pressing the button
- *"text"* (string) - the visible label text of the button
- *"keybind"* (string) - a GTK key name associated with the button
- *"icon"* (path string) - filename to load an icon image from
- *"height"* (*optional*, number) - value in the range 0.0 to 1.0 to override the placement of the label in the button
- *"width"* (*optional*, number) - value in the range 0.0 to 1.0 to override the placement of the label in the button
- *"circular"* (*optional*, boolean, default false)

# FILE

Example file:

```
{
    "close-on-lost-focus": true,
    "show-keybinds": true,
    "buttons-per-row": "1/1",
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

# AUTHORS

Based on Wlogout by Haden Collins <collinshaden@gmail.com>. For more information about wlogout, see <https://github.com/ArtsyMacaw/wlogout>.
Rewrite by Natty <natty.sh.git@gmail.com>, see <https://github.com/AMNatty/wleave>.

# SEE ALSO

*wleave*(5)
*wleave*(1)
