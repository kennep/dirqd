# dirqd

This is a deamon that listes for new files in a directory. For each
file matching a given pattern, it invokes a process with the filename as an argument,
and after the process completes, it either deletes the file or moves it to a
different directory.

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

```
Invoke processes based on incoming files in a directory

Usage: dirqd.exe [OPTIONS] <DIRECTORY> [COMMAND]...

Arguments:
  <DIRECTORY>   Directory to watch
  [COMMAND]...  Command to invoke

Options:
  -p, --pattern <PATTERN>
          Files must match this shell pattern [default: *]
  -P, --processed-queue <PROCESSED_QUEUE>
          After successfully processing, move files here. Cannot be used with --delete
      --delete
          After successful processing, delete file. Cannot be used with --processed-queue
  -E, --error-queue <ERROR_QUEUE>
          If invoking command fails, move files here. Cannot be used with --delete-on-error
      --delete-on-error
          If invoking command fails, delete file. Cannot be used with --error-queue
  -h, --help
          Print help
  -V, --version
          Print version
```

## Feedback

Feedback, PRs, etc. are most welcome.

