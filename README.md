# dirqd

This is a deamon that listes for new files in a directory. For each
file matching a given pattern, it invokes a process with the filename as an argument,
and after the process completes, it deletes the file.

![Build status](https://github.com/kennep/dirqd/actions/workflows/ci.yml/badge.svg)

## Download 

Pre-compiled binaries for Linux, Windows and (Intel) macOS are available on the
[releases](https://github.com/kennep/dirqd/releases/latest) page.

## Building

If you have Rust and Cargo installed, you can download the source and build it yourself
in the regular way:

```bash
$ cargo build
```

## Usage

TBD

## Feedback

Feedback, PRs, etc. are most welcome.

