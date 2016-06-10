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
	* [Sections of the admiral.toml](#sections-of-the-admiraltoml)
		* [path](#path)
		* [shell](#shell)
		* [reload](#reload)
		* [static](#static)
	* [Newlines](#newlines)
* [Example](#example)
	* [[admiral]](#admiral-2)
	* [Scripts](#scripts)
	* [Formatting](#formatting)
* [Bugs](#bugs)

## Introduction

*If you just want to get Admiral running, skip to the [next section](#getting-started).*

Programs like [lemonbar](https://github.com/LemonBoy/bar) and
[i3bar](https://i3wm.org/i3bar/) have become popular in recent years,
and with good reason. For those of you who are unfamiliar with them,
these programs read from standard input and then output what they receive on a bar.
For the most part, their output is identical to their input, but they accept
some format strings which allow the user to specify things like colors,
justification, and clickable areas.

The advantage of this system is that it is very powerful. The disadvantage is that
it can be rather difficult to configure.

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

Another major problem is that shell scripts of these types are bug-prone and
tend to become very bloated.
Admiral has a declarative style which allows you to configure your bar with ease
and avoid pesky bugs.

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

An example `admiral.d/` (complete with the `admiral.toml`) is included in this
repository.

### [admiral]

`[admiral]` is the section where Admiral's output is configured.
It has one required entry: `items`. Here is an example `[admiral]` section:

```
[admiral]
items = ["music", "workspaces", "clock"]
```

Each entry in the `items` table specifies a section of the config file that will be run.
Note that the order specified here is the order that Admiral will use
for the scripts' output.

### Sections of the admiral.toml

Each section of the `admiral.toml` contains a command that produces some output;
you can use shell scripts, python scripts, executable binaries, etc.

Here is an example script section:

```
[clock]
path = "date '+%I:%M %p'"
reload = 1
```

#### path

`path` is the only required entry for a script. The specified command is executed
by a shell running in the same directory as the `admiral.toml`.
So, `path = "./mpd.sh"` will run the `mpd.sh` in the same directory as the
`admiral.toml`, and `path = "echo 'Hello, world!'"` will output "Hello, world!" using
your shell. Note that the shell is determined via the `$SHELL` environment variable.

#### shell

`shell` is an optional variable that specifies an alternate shell to execute commands
with. The default shell is your `$SHELL` environment variable. Using an alternate
shell may be useful if you wish to leverage features of a specific shell for a certain
command. An example use is `shell = "/usr/bin/fish"`.

#### reload

The `reload` value is the optional duration in seconds between
each execution of the script. It may be either an integer such as `10`
or a float such as `0.5`.

If no `reload` value is specified, and `static` is not set to `true`, this indicates
that the script should never exit. It will be run, and each line it outputs will be
used separately. This is for commands such as `xtitle -s`, which handle polling
themselves and output new information on a new line. If the process is killed, it will
automatically be restarted.

#### static

`static` is an optional boolean variable. It is set to false by default. It is for
scripts that only need to be run once. Here is an example:

```
[center]
path = "echo '%{c}'"
static = true
```

This script is used to add a format sequence for `lemonbar`. It only needs to be
run once, and its output will never change.

### Newlines

Bars expect newline characters to be used only at the end of each full line of input;
Admiral tries to respect this by trimming newline characters from the output
of scripts. Users should be aware of how this is handled:

* Both `\r` and `\n` characters are removed from the start and end of a script's
output

* If no `reload` value is specified and `static` is false, Admiral
uses each line produced by the script. This means that each line meant to be displayed
must end in either `\n` or `\r\n`. However, these characters will still be
stripped from Admiral's output so as to keep its complete output on a single line.

## Example

An example `admiral.d/` directory is included with admiral. The example is designed for
use with [bspwm](https://github.com/baskerville/bspwm), and also relies on
[xtitle](https://github.com/baskerville/xtitle) to get the window title. Its
output is designed to be piped to `lemonbar`.
The command `admiral | lemonbar -g x30 | sh` should work for a demonstration, although
a greater number of clickable areas may need to be specified with `lemonbar -a`
if you have more than 8 desktops.

The example bar has three sections: BSPWM workspace information, the current window
title, and the current time. The workspace section uses the
letters `f`, `o`, and `u` to represent free (empty), occupied, and urgent desktops,
respectively. Lowercase letters represent unfocused desktops, and an uppercase
letter represents the focused desktop.

This workspace section is clickable. Left-clicking on a letter will switch to the
corresponding desktop. Scrolling up with the mouse wheel while the cursor is over
the workspace section will switch to the previous desktop, and scrolling
down with the mouse wheel will switch to the next desktop.

This directory contains two files: an `admiral.toml` file and `bspwm_workspaces.sh`.
Let's look at the example `admiral.toml`:

### [admiral]

This is the provided `[admiral]` section:

```
[admiral]
items = ["left", "workspaces",
         "center", "title",
         "right", "clock"]
```

Six scripts are listed. Three are used to provide information, and the other
three are used for formatting the output.

### Scripts

These are the first three scripts listed in the example `admiral.toml` file:

```
[workspaces]
path = "./bspwm_workspaces.sh"

[title]
path = "xtitle -s -t 100"

[clock]
path = "date '+%-I:%M %p  '"
reload = 1
```

`bspwm_workspaces.sh` is a Bash script that parses the output of `bspc subscribe report`
and converts it into a clickable, human-readable format for `lemonbar`. Because
`bspc subscribe report` (and therefore `bspwm_workspaces.sh`) never exits, no reload
value is specified.

### Formatting

These are the last three scripts:

```
[left]
path = "echo '%{l}'"
static = true

[center]
path = "echo '%{c}'"
static = true

[right]
path = "echo '%{r}'"
static = true
```

These output format strings to be interpreted by `lemonbar`. As their output only
needs to be captured once, `static` is set to `true`.
Remember that `admiral` removes trailing newline characters; this means that
using `echo` rather than `echo -n` will still work here.

Keeping the format strings outside of the main scripts allows for quicker, easier
formatting.

## Bugs

* Specifying a toml file in the current directory as `admiral -c admiral.toml`
causes scripts with relative paths to fail.
	* Workaround: Give the "directory name" as well, i.e. `admiral -c ./admiral.toml`

Any bugs that are found should be reported
[here](https://github.com/sector-f/admiral/issues).
