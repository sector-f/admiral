Alright, here's what there is so far:

`admiral.d` goes in `XDG_CONFIG_HOME` or `$HOME/.config`.
A file named `admiral.toml` is looked for in the configuration directory.
Alternatively, a configuration file can be specified with the `-c` flag.

Let's look at the provided configuration file:

````
[admiral]
items = ["title", "clock"]

[title]
path = "title.sh"

[clock]
path = "clock.sh"
reload = 1000
````

The first section is `[admiral]`; this is used to specify the order of the items.
Defined items do not have to be used.

After that, items are defined. This requires a `path` and may have a `reload` value.
If a `path` is relative, it is relative to `admiral.d` (or the directory containing the configuration file, if `-c` was used).
If a path is absolute...well, it's absolute.
The `reload` value is the duration in milliseconds between each execution of the command.
If no `reload` value is specified, the command is only run once. This is for commands
that never actually exit, such as `xtitle -s`, which `title.sh` is a wrapper for.

Arguments can be passed to commands. This is done using a toml array.
For example, `path = ["foo.sh", "--bar"]`.
Note that `path = "script.sh"` and `path = ["script.sh"]` are equivalent.

The provided scripts are enough to demonstrate a working (albeit minimal) bar.
`admiral | lemonbar` will produce a bar showing the window title and the current time,
assuming `xtitle` is installed.
