# Magic School Bus
![Current crates.io version](https://img.shields.io/crates/v/magic-school-bus.svg)
> Seatbelts, everyone!

Magic School Bus is a terminal UI filesystem browser with Vi-inspired keybindings.

It's a work in progress, but runs on Windows, MacOS, and Linux!

## Installation

### Windows
Pre-built binaries are available on the [GitHub Releases page](https://github.com/LPGhatguy/magic-school-bus/releases).

### Other Platforms and Rust Developers
Magic School Bus needs **Rust 1.31** or newer to build.

If you already have Rust installed, you can grab Magic School Bus with:

```sh
cargo install magic-school-bus

# To upgrade, add --force to overwrite your current installation
cargo install --force magic-school-bus
```

## Usage
To start up Magic School Bus, just run:

```sh
# Start in the current directory
msb

# You can also pass a directory to start in
msb ../some-directory
```

This will start you on an educational adventure in your current directory:

![Example of Magic School Bus](images/demo.gif)

For detailed information on all options and flags, run:

```sh
msb --help
```

### Actions
Actions marked with '(repeatable)' can be prefixed by a number, which will repeat the command N times.

- `q`: Exit
- `j` or `<down arrow>`: Move down in the list (repeatable)
- `k` or `<up arrow>`: Move up in the list (repeatable)
- `g`: Move to the top of the list
- `G`: Move to the bottom of the list
- `<return>`: Activate an item in the list
	- If the item is a folder, it'll become the focus
	- If the item is a file, it will be opened according to your operating system preferences
- `f`: Find an entry starting with the given input
	- Use `<tab>` to cycle between options matching the current input
	- Use `<return>` or `<escape>` to exit find mode
- `n`: Create a new file, prompted for the name
	- Use `<escape>` to cancel
- `N`: Create a new directory, prompted for the name
	- Use `<escape>` to cancel
- `x`: Prompt to delete the selected entry
	- Press `y` to confirm or `<escape>` to cancel
- `r`: Refresh the directory list, useful for when an outside program modifies the directory

### Changing shell working directory on exit
Magic School Bus has a special mode intended to help move your shell to the location you navigated to when you exit!

Passing `--pwd` will cause the last working directory to be printed to stderr. You can set up an alias or function to capture stderr and `cd` to it if `msb` exited successfully!

The function I have configured in my `.profile` for Bash is:

```bash
function brw() {
	{ error=$(msb --pwd "$@" 2>&1 1>&$out); } {out}>&1

	if [ "$?" -eq 0 ]
	then
		cd "$error"
	fi
}
```

I can type `brw` anywhere to be dropped into a filesystem explorer, navigate around, and when I pop out, I'll be in the right spot!

## License
This project is available under the MIT license. Details are available in [LICENSE.txt](LICENSE.txt).