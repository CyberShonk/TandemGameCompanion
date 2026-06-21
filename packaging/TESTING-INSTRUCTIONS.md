# Tandem Game Companion — Alpha Testing Guide

This is a very early alpha build, so things are still pretty manual. Right now you'll need to edit a configuration file yourself, and there isn't a graphical setup screen yet.

## What you'll need

- A Windows x64 game
- Any trainer or companion tool you want to launch
- A text editor

## 1. Put Tandem in your game folder

Place these files next to your game's executable:

```text
GameFolder/
├── TandemGameCompanion.exe
├── Tandem.toml
├── ExampleGame.exe
└── Tools/
    └── CompanionTool.exe
```

Keeping everything together makes the setup easier and more portable.

Don't rename or replace the original game executable.

## 2. Edit `Tandem.toml`

Open `Tandem.toml` in your favorite text editor.

Example configuration:

```toml
config_version = 1

[launcher]
log_file = "Tandem.log"
allow_external_paths = false
continue_on_optional_tool_failure = true

[game]
name = "Example Game"
path = "ExampleGame.exe"
arguments = []
working_directory = "."

[[tools]]
name = "Companion Tool"
path = "Tools/CompanionTool.exe"
arguments = []
working_directory = "Tools"
launch = "after-game"
delay_ms = 2000
required = false
close_when_game_exits = true
```

Change:

```toml
path = "ExampleGame.exe"
```

to match your actual game executable.

Change:

```toml
path = "Tools/CompanionTool.exe"
```

to match the trainer or tool you want to launch.

Use forward slashes in paths:

```toml
path = "Tools/MyTrainer.exe"
```

## 3. Choose when tools launch

You can launch a tool before or after the game starts.

```toml
launch = "before-game"
```

Starts the tool first.

```toml
launch = "after-game"
```

Starts the tool after the game launches.

For a trainer that must be configured before the game, add:

```toml
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
```

Tandem launches the trainer, shows a native OK/Cancel prompt, and starts the game only after OK.

For a one-shot setup utility, use:

```toml
launch = "before-game"
before_game_wait = "tool-exit"
required = true
```

Tandem waits for the utility and stops if it exits unsuccessfully.

You can also add a delay before launching a tool:

```toml
delay_ms = 2000
```

That's 2 seconds.

Some trainers need a little more time:

```toml
delay_ms = 5000
```

That's 5 seconds.

## 4. Tool behavior

```toml
required = false
```

If the tool fails to start, the game will still launch.

```toml
required = true
```

If the tool fails, Tandem treats the whole launch as a failure.

For most trainer testing, `false` is probably the better choice.

```toml
close_when_game_exits = true
```

Tandem will try to close the tool when the game closes.

```toml
close_when_game_exits = false
```

The tool will stay running after the game exits.

## 5. Adding more tools

Want to launch multiple tools? Just add another `[[tools]]` section.

Example:

```toml
[[tools]]
name = "Controller Utility"
path = "Tools/ControllerUtility.exe"
arguments = []
working_directory = "Tools"
launch = "before-game"
delay_ms = 0
required = false
close_when_game_exits = true
```

Currently supported:

- `.exe`
- `.com`
- `.bat`
- `.cmd`

BAT and CMD arguments are supported when they contain no shell metacharacters. Tandem rejects unsafe script paths or arguments instead of accepting free-form command text.

## 6. Set up GameNative

Open your game's container settings in GameNative.

Set the main executable to:

```text
TandemGameCompanion.exe
```

The working directory should be the folder containing:

```text
TandemGameCompanion.exe
Tandem.toml
the original game executable
```

Leave executable arguments empty unless you're specifically testing something.

Don't point GameNative directly at the game executable. Let Tandem handle launching the game.

## 7. Launch the game

Start the container normally.

Tandem should:

1. Read `Tandem.toml`
2. Launch any `before-game` tools
3. Launch the game
4. Launch any `after-game` tools
5. Stay running while the game is open
6. Close configured tools when the game exits
7. Write everything to `Tandem.log`

You may see a console window during testing. That's normal for this alpha version.

## 8. Check the log

Tandem creates:

```text
Tandem.log
```

in the same folder as `Tandem.toml`.

A successful log might look something like:

```text
Tandem Game Companion
Starting game: Example Game
Game process started with PID 1234
Starting companion tool: Companion Tool
Companion Tool started with PID 1250
Tandem is supervising the game process.
Game exited with status: exit code: 0
Tandem session finished.
```

If something goes wrong, please include `Tandem.log` when reporting it.

## 9. Validate your configuration

If you're running from Windows or a Wine command prompt, you can check your config with:

```text
TandemGameCompanion.exe --validate
```

A valid config should report something like:

```text
Configuration is valid: ...\Tandem.toml
```

You can also preview what Tandem plans to launch without actually starting anything:

```text
TandemGameCompanion.exe --dry-run
```

These commands are optional and may not be convenient to run through GameNative.

## 10. Current limitations

Since this is still an alpha, a lot of features are not finished yet.

Not included yet:

- Graphical setup interface
- File picker dialogs
- A full controller-driven setup interface (the before-game confirmation uses a standard Windows dialog)
- Notifications
- Silent/background operation
- Automatic config generation
- Automatic worker restart after crashes
- Full GameNative testing across devices
- Support for every launcher type

Whenever possible, point Tandem directly at the real game executable instead of a launcher.

Also keep in mind that Tandem can't stop the container from closing if GameNative shuts down the entire Wine environment or if the Tandem process itself is forcefully terminated.

## 11. Reporting your results

Please include as much of the following as possible:

- Device model
- Android version
- GameNative version
- Game name
- Game executable filename
- Trainer or tool filename
- Whether the game launched successfully
- Whether the trainer/tool launched successfully
- Whether the trainer detected the game
- Whether the container stayed open
- What happened when the game closed
- `Tandem.log`
- Your `Tandem.toml` file (remove any private information if needed)
