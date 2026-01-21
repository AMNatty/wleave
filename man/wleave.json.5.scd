wleave.json(5)

# NAME

wleave.json - layout JSON file and configuration options

# TOP-LEVEL-OPTIONS

The top-level options directly correspond to their long-version command-line option counter parts.
Button layout is specified in the extra *"button"* field.

*"buttons"*: <array of buttons>
	See section *LAYOUT* for structure

*"button-layout"*: <*"grid"*>
    Specify the way buttons should be laid out, "grid" being the default. Currently, only "grid" is supported.

*"css"*: <string>
	Specify a custom CSS file instead of the default one

*"service"*: <boolean>
    Run the application as a service, with all instances of wleave opening this one. Allows faster startup at the
    cost of running in the background. ++
*Default*: false

*"buttons-per-row"*: <string>
	Set the number of buttons per row, or use a fraction to specify the number of rows to be used
	(e.g. "1/1" for all buttons in a single row, "1/5" to distribute the buttons over 5 rows) ++
*Default*: "3"

*"column-spacing"*: <number / "<number>px" / "<number>%">
	Set space between button columns ++
*Default*: "8px"

*"row-spacing"*: <number / "<number>px" / "<number>%">
	Set space between button rows ++
*Default*: "8px"

*"margin"*: <number / "<number>px" / "<number>%">
	Set margin on all sides ++
*Default*: "10%" (of window size)

*"margin-left"* <number / "<number>px" / "<number>%">
	Set margin for left of buttons ++
Falls back to the value set by *margin*

*"margin-right"* <number / "<number>px" / "<number>%">
	Set margin for right of buttons ++
Falls back to the value set by *margin*

*"margin-top"* <number / "<number>px" / "<number>%">
	Set margin for top of buttons ++
Falls back to the value set by *margin*

*"margin-bottom"* <number / "<number>px" / "<number>%">
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

*"protocol"*: <*"layer-shell"*/*"xdg"*/*"none"*>
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
- *"action"* (command string or object or list thereof) - the action that will be executed via shell upon pressing the button
- *"text"* (string) - the visible label text of the button
- *"keybind"* (string) - a GTK key name associated with the button
- *"icon"* (path string) - filename to load an icon image from
- *"height"* (*optional*, number) - value in the range 0.0 to 1.0 to override the placement of the label in the button
- *"width"* (*optional*, number) - value in the range 0.0 to 1.0 to override the placement of the label in the button
- *"circular"* (*optional*, boolean, default false)

The action field can be an object with exactly one of `shell` or `executable` properties set. The `shell` action executes
the given command in the system shell, while the `executable` action executes the specified binary, assuming it in `$PATH`.
`executable` actions get filtered if the specified executable does not exist, allowing multiple possible options.

Any extra properties in the `action` objects are interpreted as filters. Currently, only environment variable filtering
is implemented, where the values fields prefixed with `$` are matched as conditions for the given action.

The action field can also be an array where the first matching command is picked.

For example, in the following example, the `loginctl lock-session` command is executed on GNOME, while other desktop
environments will try either `gtklock` or `swaylock` in that order.

```
"action": [
    {
        "$DESKTOP_SESSION": "gnome",
        "shell": "loginctl lock-session"
    },
    {
        "executable": "gtklock"
    },
    {
        "executable": "swaylock"
    }
]
```

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
