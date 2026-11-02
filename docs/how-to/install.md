# How to Install apple-message-forensics

This guide walks through downloading pre-compiled binaries, extracting them, and configuring your environment so you can run `apple-message-forensics` from any directory.

## Prerequisites

- macOS 13 or later.
- `Homebrew` installed.

## Install Tools

Install `just` and `sevenzip` via Homebrew:

```bash
brew install just sevenzip
```

## Determine Your CPU Architecture

Run the following command to determine whether your Mac uses Apple silicon or an Intel processor:

```bash
sysctl -n machdep.cpu.brand_string | grep -qi 'Apple' && echo "Apple silicon" || echo "Intel CPU"
```

## Download and Extract the Binary

1. Go to the [Releases page](https://github.com/McFlip/apple-message-forensics/releases).
2. Download the release archive matching your architecture.
3. Extract the archive using 7-Zip:

```bash
7zz x apple-message-forensics_v*.7z
```

4. Make the binary executable:

```bash
chmod +x apple-message-forensics
```

## Make Runnable from Anywhere

To run the tool without specifying its path:

1. Create a personal `bin` folder if one does not already exist:

```bash
mkdir -p ~/bin
```

2. Move the binary into `~/bin`:

```bash
mv apple-message-forensics ~/bin/
chmod +x ~/bin/apple-message-forensics
```

3. Add `~/bin` to your `PATH` if it is not already included. For zsh (the default macOS shell):

```bash
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

For bash:

```bash
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

## Verification

Run the binary with the `--version` flag:

```bash
apple-message-forensics --version
```

The command outputs the installed version string.
