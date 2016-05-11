Alright, here's what there is so far:

admiral.d goes in `XDG_CONFIG_HOME` or `$HOME/.config`.
This directory currently contains three files: a toml file and two bash scripts.
The toml file has two tables. Each table has a script name and a time.
Upon running, the toml file is read, and the scripts listed in it are run continuously.
The time listed in the toml file is the duration between each execution of each script.

Heck, lemme just show it to you:

````
[foo]
path = "foo.sh"
reload = 1

[bar]
path = "bar.sh"
reload = 5
````

So, `foo.sh` gets run every second and `bar.sh` gets run every 5 seconds.

`reload` is optional; if it is not specified, the command is only run once.
This may be useful for things that poll on their own, such as `bspc subscribe report`

If a `path` is relative, it is relative to `admiral.d`.
