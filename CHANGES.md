# Change History

## master
* *No changes*

## 0.6.0
- Switched from Crossterm to All-Term for a terminal backend
	- Arrow keys now work!
- Changed find mechanics ([#12](https://github.com/LPGhatguy/magic-school-bus/issues/12))
	- `f` now enters a prompt to enter a file prefix to find
		- tab cycles options (shift-tab can't work because of library limitations right now)
		- return or escape exits the prompt
	- `F`, `,` and `;` motions were removed

## 0.5.0
- Improved resize behavior: after resizing the terminal, any action should redraw the entire screen _much_ more quickly.
- Implemented command bar (`:`) with no commands
- Implemented new motions:
	- `n`: Create a new file, prompted for a name
	- `N`: Create a new directory, prompted for a name
	- `r`: Refresh the view, in case of changes from another program

## 0.4.0
- Added command repeat information to status bar
- Entries now show folders first, then files, both case-insensitive alphabetized.
- Implemented find motions:
	- `f [character]`: Jump to the next entry starting with `[character]` (repeatable)
	- `F [character]`: Jump to the previous entry starting with `[character]` (repeatable)
	- `;`: Jump to the next entry matching the most recent find command (repeatable)
	- `,`: Jump to the previous entry matching the most recent find command (repeatable)
- Fixed hanging when opening files when your default editor takes awhile to open (looking at you, Visual Studio)
- Implemented delete selected action, `x` ([#8](https://github.com/LPGhatguy/magic-school-bus/issues/8))

## 0.3.1
- Added `g` (top of list) and `G` bottom of list motions ([#5](https://github.com/LPGhatguy/magic-school-bus/issues/5))
- Implemented action repeat ([#6](https://github.com/LPGhatguy/magic-school-bus/issues/6))

## 0.3.0
- Updated status bar to be more readable
- Added working directory bar ([#1](https://github.com/LPGhatguy/magic-school-bus/issues/1))
- Made list rendering a little fancier
- Fixed cursor showing up after resizing

## 0.2.1
- Directory list is now windowed, so browsing larger directories is feasible.

## 0.2.0
- Switched to Clap frontend

## 0.1.0
- Initial releases