Alright, here's what there is so far:

admiral.d goes in `XDG_CONFIG_HOME` or `$HOME/.config`.
This directory currently contains three files: a toml file and two bash scripts.
The toml file has two tables. Each table has a path to a script (which you're gonna have to change unless your username is adam) and a time.
Upon running, the toml file is read, and the scripts listed in it are run continuously. 
The time listed in the toml file is the duration between each execution of each script.

Heck, lemme just show it to you: 

````
[foo]
path = "/home/adam/.config/admiral.d/foo.sh"
time = 1

[bar]
path = "/home/adam/.config/admiral.d/bar.sh"
time = 5
````

So, `foo.sh` gets run every second and `bar.sh` gets run every 5 seconds.
