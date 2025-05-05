# Arctis Battery Indicator

Adds a small icon to the "system tray" area of the Windows task bar, which displays the battery level of any connected SteelSeries Arctis headset.

![Screenshot of indicator on Windows task bar](docs/icon-screenshot.png)

## Features

* Works on Windows 10+
* Built using Rust, with very low resource usage (<1MB RAM)
* Supports all known Arctis headsets

## Installation

* Download the [latest release](https://github.com/aarol/arctis-battery-indicator/releases/latest) .zip file
* Extract all of the files into any folder (like Downloads).
* Right click `Install.ps1` and run the PowerShell script.

You can also update the program by downloading a new version and running `Install.ps1`.

## How to uninstall

Simply remove all of the files in `%localAppData%\ArctisBatteryIndicator` (copy the path, press Win+R and paste the path to enter)

The installer also creates a file in `%appData%\Microsoft\Windows\Start Menu\Programs\Startup` to run the program on startup.

## Troubleshooting

If you're experiencing crashes or other issues, you can try running the `arctis-battery-indicator-debug.exe` located at `%localAppData%\ArctisBatteryIndicator` or look at the log file located in the same folder.

### Why does it only show 100%, 75%, 50%, 25% or 0%?

This is limitation of the headsets themselves, as the devices only expose 5 possible battery states.

### My headset is connected, but it still shows "No headphone adapter found"

Your headset might be unsupported due to being a newer model. Either [create a new issue](https://github.com/aarol/arctis-battery-indicator/issues/new) or see [Adding a new headset](#adding-a-new-headset)

### Connection status updates slowly / unreliably

Often the adapter (plugged into your PC) is *really slow* at updating its information whenever something happens, like headphones disconnecting / reconnecting. I haven't found a way to work around this yet. Shutting down the SteelSeries GG app *may* help on some occasions.

## Development

Rust and Cargo need to be installed.

* Running the application: `cargo run --release`

* Installing the application locally: `cargo install`

* Debugging the application: press `F5` in VS Code with the Rust Analyzer extension and CodeLLDB installed.

## Todo

* Updated icons to show charging states, etc.
* Add better installation experience.

## Adding a new headset

I have a post on my website explaining that explains how to do this: <https://aarol.dev/posts/arctis-hid>

If you have found the necessary data, you can try it out by creating a custom config file:

1. Install version 2.0.0 or greater of Arctis Battery Indicator
2. Navigate to the program data folder (that would be `%localAppData%\Programs\ArctisBatteryIndicator` by default)
3. Rename the `example.config.toml` file to `config.toml`
4. Update the data in the `config.toml` file (most headphones have similar configurations, you can look at [here](./src/headphone_models.rs) for inspiration)
5. Launch the `arctis-battery-indicator-debug.exe` in the same directory to test your changes. It should say "Found custom config" in the logs.

New configurations can be tested by using a `config.toml` file. In this repository, there is an a file, [example.config.toml](./example.config.toml) which can be copied as `config.toml` to a file next to `arctis-battery-indicator.exe` and other files. When starting the program again, it will only try to find a headphone matching the configuration file. It's recommended to use the `arctis-battery-indicator-debug.exe` to test this.

If you have a working configuration file, you should create a [new issue](https://github.com/aarol/arctis-battery-indicator/issues/new) with the file contents, so that I can add it to the list of known headphones.

The parameters, such as `write_bytes` and `battery_percent_idx` can be discovered by sniffing the USB traffic with something like [WireShark](https://www.wireshark.org/) and [USBPcap](https://desowin.org/usbpcap/)
