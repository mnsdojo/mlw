# MLW (Multi-Language Watcher)

MLW is a flexible command-line tool that monitors files for changes and automatically restarts scripts, regardless of the language. It is highly customizable and supports multiple file types and configurations.

## Features

- **Automatic Script Restart**: Automatically restarts your script when any watched file changes.
- **Multi-Language Support**: Supports various programming languages (e.g., Python, JavaScript, Go, C,etc.) by recognizing file extensions.
- **Configurable**: Easily customizable via an `mwl.toml` configuration file.
- **File Watcher**: Watches for file changes in specific directories and for specified file extensions.
- **Customizable Watch Interval**: Adjust the interval at which files are checked for changes.
- **Logging**: Outputs detailed logs based on the specified logging level.

## Installation

You can install MLW using your package manager or by downloading it directly from the release page.

## Basic Usage

### 1. Generate Configuration

To generate the default configuration file for MLW, run the following command:

```bash
mlw --gen-config

```
