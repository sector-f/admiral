# Admiral

An asynchronous bar wrapper written in Rust

## Table of Contents

* [Introduction](#introduction)
	* [What Admiral Does](#what-admiral-does)
* [Getting Started](#getting-started)
	* [Prerequisites](#prerequisites)
	* [Installation](#installation)
* [Configuration](#configuration)
	* [[admiral]](#admiral-1)
	* [Scripts](#scripts)
		* [path](#path)
		* [reload](#reload)
		* [static](#static)
* [Bugs](#bugs)

## Introduction

*If you just want to get Admiral running, skip to the [next section](#getting-started).*

Programs like [lemonbar](https://github.com/LemonBoy/bar) and
[i3bar](https://i3wm.org/i3bar/) have become popular in recent years,
and with good reason. For those of you who are unfamiliar with them,
these programs read from standard input and then ouput what they receive on a bar.
For the most part, their output is identical to their input, but they accept
some format strings which allow the user to specify things like colors,
justification, and clickable areas.

The pro of this system is that it is very powerful. The con is that it can
be rather difficult to configure.

A typical bar script looks something like this: a shell script, probably bash,
is used to collect and format the output of various commands. This is done in
an infinite loop, probably ending with `sleep 0.1` or something similar.
The output of this script is then piped into the bar program, which receives a new
line (which may be identical to its previous line) ten times per second.

The main problem with this method is that it's rather difficult to
handle timing correctly. A counter showing the number of
outdated programs on the system can be updated less frequently than a clock.

Actually, a previous bar script I had contained both of the aforementioned
items. One day, I noticed the clock on my bar had stopped working.
The cause? The internet had died. This prevented the package counter command
(`checkupdates | wc -l`) from ever finishing, which caused the entire
bar to stop working.

### What Admiral Does

Admiral runs scripts specified by the user and prints their output. It allows
for a clear separation of different sections, which simplifies configuration.

The best part? **Each script is handled asynchronously by its own thread.**
This means that each script is updated independently of the other scripts.
It also means that **if something fails, it fails independently** of the other scripts.
That example where the clock stopped? This would never happen with Admiral.
Sure, `checkupdates` would still hang indefinitely and prevent that
number from changing. But the clock (and the rest of the bar) would be unaffected.

Admiral also limits its output. Whenever one of its scripts updates, it checks to see
if anything has actually changed since it last printed a message. If nothing has
changed, Admiral prints nothing—this limits the amount of refreshing that your bar
program has to do.


## Getting Started

### Prerequisites

* `rustc` — the Rust compiler

* `cargo` — the Rust package manager

* `git` (optional but recommended)

These can probably be installed via your distribution's package manager.
If the Rust compiler and Cargo are not packaged for your distro,
you can download them [here](https://www.rust-lang.org/).

### Installation

1. Clone this repository with `git clone https://github.com/sector-f/admiral.git`
	* Alternatively, a .zip file of the master branch can
	be downloaded [here](https://github.com/sector-f/admiral/archive/master.zip)
2. `cd` into the newly-created `admiral/` directory
3. Run `cargo build --release`

This will create an `admiral` excutable in the `./target/release/` directory.
You may want to copy this to somewhere in your `$PATH`,
like `/usr/local/bin/` or `~/.local/bin/`

You may then copy the provided `admiral.d/` directory to `~/.config/`
(or your `$XDG_CONFIG_HOME` directory, if you have that environment variable set).

## Configuration

Configuration is done with an `admiral.toml` file. This file is looked for in
`~/.config/admiral.d/` (Or `$XDG_CONFIG_HOME/admiral.d/`, if that environment
variable is set). Alternatively, a configuration file may be specified with
the `-c` flag, e.g. `admiral -c /path/to/file.toml`.

An example `admiral.d/` (complete with `admiral.toml` file) is included in this
repository.

### [admiral]

`[admiral]` is the section where Admiral's output is configured.
It has one required entry: `items`. Here is the example `[admiral]` section
from the provided `admiral.toml` file:

```
[admiral]
items = ["left", "workspaces",
         "center", "title",
         "right", "clock"]
```

Each entry in the `items` table is a script that will be run.
Note that the order specified here is the order that Admiral will use
for the scripts' output.

### Scripts

The word "scripts" isn't entirely accurate; these can be anything that produce
output on the command line: shell scripts, python scripts, executable binaries, etc.

Here is an example script section:

```
[clock]
path = ["/usr/bin/date", "+%-I:%M %p"]
reload = 1
```

Note that leading and trailing newline characters are stripped from the output
of scripts.

#### path

`path` is the only required entry for a script. This will normally be a string,
such as `path = "workspaces.sh"` If arguments need to be passed, it is done using
an array, as shown above. If an array is used, the first item in it is the path
to the script/command, and additional items in the array are the arguments passed
to it.

If a `path` is relative, it is relative to `admiral.d` (or the directory containing
the configuration file, if `-c` was used).

#### reload

The `reload` value is the optional duration in seconds between
each execution of the script. It may be either an integer such as `10`
or a float such as `0.5`.

If no `reload` value is specified, and `static` is not set to `true`, this indicates
that the script should never exit. It will be run, and each line it outputs will be used
separately. This is for commands such as `xtitle -s`, which handle polling themselves
and output new information on a new line. If the process is killed, it will
automatically be restarted.

#### static

`static` is an optional boolean variable. It is set to false by default. It is for
scripts that only need to be run once. Here is an example: 

```
[center]
path = ["/usr/bin/echo", "%{c}"]
static = true
```

This script is used to add a format sequence for `lemonbar`. It only needs to be
run once, and its output will never change.

## Bugs

* Specifying a toml file in the current directoy as `admiral -c admiral.toml`
causes scripts with relative paths to fail.
	* Workaround: Give the "directory name" as well, i.e. `admiral -c ./admiral.toml`

Any bugs that are found should be reported
[here](https://github.com/sector-f/admiral/issues).
