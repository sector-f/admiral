Alright, here's what there is so far:

`admiral.d` goes in `XDG_CONFIG_HOME` or `$HOME/.config`.
A file named `admiral.toml` is looked for in the configuration directory.
Alternatively, a configuration file can be specified with the `-c` flag.

Let's look at the provided configuration file:

````
[admiral]
items = ["left", "workspaces",
         "center", "title",
         "right", "clock"]

[workspaces]
path = "workspaces.sh"

[title]
path = ["/usr/bin/xtitle", "-s", "-t", "100"]

[clock]
path = ["/usr/bin/date", "+%-I:%M %p  "]
reload = 1

# Used for formatting lemonbar
[left]
path = ["/usr/bin/echo", "%{l}"]

[center]
path = ["/usr/bin/echo", "%{c}"]

[right]
path = ["/usr/bin/echo", "%{r}"]
````

The first section is `[admiral]`. It has an `items` table, which is used to
specify the order of items. Note that the items may be on the same line;
the example's formatting is just to improve readability.

After that, the items themselves are defined. Items may be anything that produces
an output: shell scripts, Python scripts, commands, etc.
Each item has a `path` value and optionally a `reload` value.
If a `path` is relative, it is relative to `admiral.d` (or the directory containing the configuration file, if `-c` was used).
If a path is absolute...well, it's absolute.
The `reload` value is the duration in seconds between each execution of the command.
It may be either a nonnegative integer (whole number) such as `10` or a float (number with a decimal place) such as `1.5`.
If no `reload` value is specified, the command is only run once. This is for commands
that never actually exit, such as `xtitle -s`.
Defined items do not have to be used.

Arguments can be passed to items. This is done using a toml array.
The first item in the array is the command or script, and subsequent
items are the arguments that are passed to it.
For example, the `[clock]` section has a path of `["/usr/bin/date", "+%-I:%M %p  "]`.
This is equivalent to `/usr/bin/date '+%-I:%M %p  '` from the command line
(The two spaces at the end are there to add a gap between the date and the end of the bar).
Note that `path = "script.sh"` and `path = ["script.sh"]` are equivalent.

The provided scripts are enough to demonstrate a working (albeit minimal) bar.
It is designed to be used with bspwm; it relies on `bspc` and `xtitle`.
Assuming these are installed and you are using bspwm, `admiral | lemonbar`
will produce a bar showing basic workspace information, the window title, and the current time,
