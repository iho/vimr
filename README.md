# vimr

A minimalist, keyboard-driven web browser built with GTK4 + WebKitGTK 6. Vim modal interface, built-in adblocking, Lua 5.4 configuration, and SQLite history. Page JavaScript is off by default.

## Features

- **Vim modal interface** — Normal, Insert, Command, and Hint modes
- **Adblocking** — EasyList + EasyPrivacy via Brave's adblock-rust engine, loaded automatically on first run
- **Lua config** — `~/.config/vimr/config.lua` loaded at startup
- **History** — SQLite database at `~/.local/share/vimr/history.db`
- **No JS by default** — page scripts are blocked; browser internals (scrolling, hints) always work

## Building

### Dependencies

**Arch Linux**
```sh
sudo pacman -S gtk4 webkitgtk-6.0 base-devel
```

**Ubuntu 24.04+**
```sh
sudo apt install libgtk-4-dev libwebkitgtk-6.0-dev build-essential pkg-config
```

**Fedora 40+**
```sh
sudo dnf install gtk4-devel webkitgtk6.0-devel gcc pkg-config
```

### Compile

```sh
cargo build --release
./target/release/vimr
```

### AppImage (portable, no dependencies)

```sh
./build-appimage.sh
./vimr-x86_64.AppImage
```

The AppImage bundles WebKitGTK and all shared libraries. Runs on any x86\_64 Linux with glibc ≥ 2.17.

## Keybindings

### Modes

| Key | Action |
|-----|--------|
| `Escape` | → Normal mode (from anywhere) |
| `i` | → Insert mode (keys pass through to page) |
| `:` | → Command mode |
| `f` | → Hint mode (follow link in current tab) |
| `F` | → Hint mode (follow link in new tab) |

### Scrolling (Normal mode)

| Key | Action |
|-----|--------|
| `j` / `k` | Scroll down / up |
| `h` / `l` | Scroll left / right |
| `Ctrl+d` / `Ctrl+u` | Half page down / up |
| `Ctrl+f` / `Ctrl+b` | Full page down / up |
| `gg` | Top of page |
| `G` | Bottom of page |

### Navigation (Normal mode)

| Key | Action |
|-----|--------|
| `H` / `L` | Back / forward |
| `r` | Reload |
| `R` | Hard reload (bypass cache) |
| `o` | Open URL or search (enters command mode with `o `) |
| `O` | Edit current URL |
| `yy` | Copy current URL to clipboard |
| `?` | Show this help |

### Tabs (Normal mode)

| Key | Action |
|-----|--------|
| `t` | New tab (enters command mode with `t `) |
| `gt` / `gT` | Next / previous tab |
| `g0` / `g$` | First / last tab |

### JavaScript (Normal mode)

| Key | Action |
|-----|--------|
| `zi` | Toggle page JavaScript on/off |

### Commands (`:`)

| Command | Action |
|---------|--------|
| `:o <url>` | Open URL or search in current tab |
| `:t <url>` | Open URL in new tab |
| `:r` / `:R` | Reload / hard reload |
| `:back` / `:forward` | Navigate history |
| `:bd` | Close current tab |
| `:tabnew` | New empty tab |
| `:q` | Quit |
| `:js` | Toggle page JavaScript |
| `:js on` / `:js off` | Enable / disable page JavaScript |
| `:noscript` | Disable page JavaScript |
| `:adblock-update` | Re-download filter lists |
| `:help` | Open keyboard reference page |

URLs are opened with `https://` if no scheme is given. Bare words without a dot are sent to DuckDuckGo.

## Configuration

Create `~/.config/vimr/config.lua`:

```lua
-- Homepage
set("homepage", "https://example.com")

-- Show/hide UI elements
set("ui.urlbar",    true)
set("ui.statusbar", true)

-- JavaScript (page scripts)
set("javascript.enabled", false)  -- default: false

-- Adblocking
set("adblock.enabled", true)

-- Custom keybindings (Normal mode)
bind("normal", "d",  "scroll-half-down")
bind("normal", "u",  "scroll-half-up")
bind("normal", "gh", "open-url")          -- remap
bind("normal", "w",  "tab-next")
bind("normal", "W",  "tab-prev")
```

### Available `set()` keys

| Key | Type | Default |
|-----|------|---------|
| `homepage` | string | `about:blank` |
| `ui.urlbar` | bool | `true` |
| `ui.statusbar` | bool | `true` |
| `javascript.enabled` | bool | `false` |
| `adblock.enabled` | bool | `true` |

### Available actions for `bind()`

```
scroll-down        scroll-up          scroll-left       scroll-right
scroll-half-down   scroll-half-up     scroll-page-down  scroll-page-up
scroll-top         scroll-bottom
back               forward            reload            reload-hard
open-url           open-url-current   open-tab
hint-follow        hint-follow-new-tab
yank-url           toggle-js          help
insert-mode        normal-mode
tab-next           tab-prev           tab-first         tab-last
```

## Adblocking

On first run, vimr downloads **EasyList** and **EasyPrivacy** (~4 MB total) into `~/.cache/vimr/adblock/`. Subsequent starts load from cache instantly.

Filter lists are applied via WebKit's resource policy decision API. Blocked resources are never rendered.

To refresh lists:
```
:adblock-update
```

## JavaScript model

vimr separates the JS engine from page scripts:

- **JS engine** is always on — required for browser internals (scrolling, link hints)
- **Page scripts** (`<script>` tags, external `.js` files) are off by default

Toggle page scripts with `zi` or `:js`. The engine itself is never disabled.

## Memory profile

- Hardware acceleration: disabled (software rendering via CPU)
- Page cache (back/forward): disabled
- WebAudio / WebGL / MediaStream: disabled
- DNS prefetch: disabled
- Autoplay: blocked (requires user gesture)

## Files

| Path | Purpose |
|------|---------|
| `~/.config/vimr/config.lua` | User configuration |
| `~/.local/share/vimr/history.db` | SQLite browsing history |
| `~/.cache/vimr/adblock/` | Cached filter lists |
