Alright, here's what there is so far:

`admiral.d` goes in `XDG_CONFIG_HOME` or `$HOME/.config`.
This directory contains a number of scripts, as well as the `admiral.toml` file.
Let's look at it.

````
[admiral]
items = ["ls", "loop", "foo", "baz", "bar"]

[foo]
path = "foo.sh"
reload = 1

[bar]
path = "bar.sh"
reload = 5

[ls]
path = ["/usr/bin/ls", "--version"]

[loop]
path = "loop.sh"

[unused]
path = "unused.sh"
reload = 60
````

The first section is `[admiral]`; this is used to specify the order of the scripts.
At least, it will be. The `BarItem` struct does have a `position` field, but I'm not doing anything with it.
Note that an error is printed about the "baz" command not existing.
Any item not listed in the `items` array will be ignored.

After that, commands are defined. This requires a `path` and may have a `reload` value.
If a `path` is relative, it is relative to `admiral.d`.
If it is absolute...well, it's absolute.
The `reload` value is the duration between each execution of the command.
If no `reload` value is specified, the command is only run once. This is for commands
that never actually exit, such as `xtitle -s`.

Arguments can be passed to commands. This is done using a toml array; similar to the `items` section of `[admiral]`
