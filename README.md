# tally

[![Crates.io](https://img.shields.io/crates/v/tally-cli.svg)](https://crates.io/crates/tally-cli)
![Docker Image Version](https://img.shields.io/docker/v/uhryniuk/tally/latest?color=blue&label=docker)

A CLI tool to manage global counters from the command line. 

Written in Rust with SQLite, it's a process-safe way to keep count across your system.

## Installation

### Cargo

Tally is published to [crate.io](https://crates.io/) and is easily installed with the following command.

```
cargo install tally-cli
```

### Docker

You can also play around with tally in a container.

```
docker run --name tally --rm -it docker.io/uhryniuk/tally
```

### Build

Alternatively, you can build it locally.

```
git clone https://github.com/uhryniuk/tally.git
cd tally
cargo build --release
```

## Getting started

Tally will create the sqlite database upon first invocation, the database is written to `~/.tally/tally.db`.

```bash
$ tally
0

$ tally list
Name   Count  Step  Template  Default
tally  0      1     {}        *

```

Adding and subtracting from counters is quite straight-forward.

```
$ tally add
1

# tally add 5
6

$ tally new-counter add 5
5

$ tally list
Name         Count  Step  Template  Default
tally        6      1     {}        *
new-counter  5      1     {}
```

Setting the default counter is possible too

```bash

$ tally new-counter set --default

$ tally list
Name         Count  Step  Template  Default
tally        6      1     {}
new-counter  5      1     {}        *
```

You can also change how much the counter steps each invocation

```bash

$ tally set --step 5

$ tally add
10

$ tally list
Name         Count  Step  Template  Default
tally        6      1     {}
new-counter  10     5     {}        *
```

Templating and referencing counters between each other is supported.

```bash

$ tally set --template "new-counter: {}"
$ tally
new-counter: 10

$ tally set --template "new-counter: {}, tally: {tally}"
$ tally add
new-counter: 11, tally: 6

$ tally list
Name         Count  Step  Template                         Default
tally        6      1     {}
new-counter  11     5     new-counter: {}, tally: {tally}  *
```

Need to clean up the counters? Simple run the `nuke` subcommand.

```bash

$ tally nuke
Are you sure wish to nuke? (y/n): y
Database deleted successfully.

$ tally list
Name   Count  Step  Template  Default
tally  0      1     {}        *
```

## Acknowledgements

Created by `uhryniuk`. Licensed under the [GPL-3.0 license](LICENSE).

