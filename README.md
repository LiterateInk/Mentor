# Mentor

A simple CLI tool to quickly release a new version of a LiterateInk library.

Of course, this is only useful for LiterateInk libraries and is only meant to be used by the maintainers of the libraries.

## Motivation

In some implementations, such as JS, we had custom tools to do this but it was only bound to that specific implementation. For example, `release-it` for the JS implementation. But, I wanted a tool that could be used in any implementation without doing any extra config, work or setup.

So, we created this tool to automate the process without any configuration or setup. It's a simple CLI tool that can be used in any of our library repositories to quickly release a new version.

## Build

You can only install it by building it from source.
Make sure you have `cargo` and `rust` installed.

```bash
cargo build --release
```

## Installation

You can install it by creating a symlink to the binary in your local bin directory.

```bash
sudo ln -s $(pwd)/target/release/mentor /usr/local/bin/mentor
```

This will create a symlink to the binary in `/usr/local/bin/mentor`, which is in your PATH, so you can run it from anywhere.

## Usage

Be in a LiterateInk repository and you can directly run the command.

```bash
mentor
```

It'll ask you for the type of bump you want for the new version, and then it'll create a new commit and tag and push it to the current branch.

It'll also redirect you to the GitHub page to create a new release with the tag, release name and the release notes - generated using a `git log`.

## Uninstallation

You can simply remove the symlink created during [Installation](#installation) to uninstall the tool.

```bash
sudo rm /usr/local/bin/mentor
```

It doesn't create any other files or directories, so you don't have to worry about that.

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.
