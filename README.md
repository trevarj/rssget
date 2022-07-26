# rssget
A simple tool to read RSS from the terminal. Load it up with feeds and waste all your time!

## Install
You can install using `cargo` with:

```
cargo install rssget
```

## Configuration
You can configure `rssget` by copying [`config.yaml`](./config.yaml) to `~/.config/rssget/config.yaml`. or by using the command line args.

## Usage

```
Usage: rssget [<channels...>] [--display-by <display-by>]

a RSS channel retriever

Positional Arguments:
  channels          a list of RSS feed urls

Options:
  --display-by      display ordering for RSS items [date | channel]
  --help            display usage information
```