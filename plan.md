
**TASK: Generate a complete, production-ready Rust project called `luavimb` (Luau-powered Vim-like Browser).**

**Project Goal**  
Build a **minimalist, extremely lightweight, keyboard-driven web browser** that feels exactly like **vimb** (the classic Vim-like WebKitGTK browser) but is rewritten in modern Rust using **GTK4 + WebKitGTK 6.0+**.  
It must be fully configurable and scriptable via **Luau** (Roblox’s fast, typed Lua dialect) for both user config and runtime extensions. Include **strong built-in adblocking** (network + cosmetic filtering).

**Core Requirements**

- **Rendering Engine**: `webkit6` crate (GTK4 + libsoup 3 version of WebKitGTK 6.0+)
- **UI**: `gtk4-rs` + `libadwaita` (for modern native look) – keep the UI extremely minimal (no unnecessary chrome)
- **Scripting & Config**: Embed **Luau** using the best available crate (`luau-rs` on crates.io or `lu` crate – choose the most mature one). All user configuration and scripting must be done in Luau.
- **Adblocking**: Full support for EasyList / uBlock Origin style filters using the official `adblock` crate (Brave’s adblock-rust engine) + WebKitGTK’s native `WebKitUserContentFilterStore` / `WebKitUserContentManager` for both network blocking and cosmetic filtering. Support automatic filter list updates.

**Key Features (must implement all)**

1. **Vim-like Modal Interface** (exactly like vimb):
   - Normal mode (default)
   - Insert / Passthrough mode (keys go to the web page)
   - Command mode (`:` prompt at the bottom)
   - Hint mode (`f` / `F` – visual hints over links, images, form fields)

2. **Vim Keybindings** (fully mappable via Luau):
   - `j/k/h/l`, `Ctrl+d/u/f/b`, `gg/G`, `0/$` – scrolling
   - `f` / `F` / `;` / `,` – link hints (quick-hint and extended)
   - `o` / `O` / `t` / `T` – open URL / new tab
   - `yy` – yank current URL
   - `gt` / `gT` / `g0` / `g$` – tab navigation
   - `r` – reload, `R` – hard reload
   - `:` – enter command mode
   - All bindings must be overridable and extendable from Luau config

3. **Luau Configuration & Scripting System**:
   - Config file: `~/.config/luavimb/config.lua` (loaded at startup)
   - Users can define:
     - Key mappings (`bind("normal", "j", "scroll-down")`)
     - Settings (`set("adblock.enabled", true)`, `set("homepage", "https://...")`)
     - Custom commands (`command("mycommand", function() ... end)`)
     - User scripts / userscripts (run on page load, like Greasemonkey)
     - Event hooks (on_navigate, on_load_finished, etc.)
   - Expose a rich Rust → Luau API (webview control, adblock control, UI, history, bookmarks, downloads, etc.)
   - Support `require()` for modular user scripts

4. **Adblocking**:
   - Built-in filter lists (EasyList, EasyPrivacy, uBlock Origin filters, etc.)
   - Automatic download/update on first run or via `:adblock-update`
   - Network blocking + cosmetic filtering (element hiding)
   - Toggle per-domain or globally via Luau or command

5. **Minimal UI Components**:
   - Single main `GtkApplicationWindow`
   - Optional URL bar (can be hidden with `set("ui.urlbar", false)`)
   - Bottom status bar showing current mode, progress, URL, adblock status
   - Tab support (simple `GtkNotebook` or custom tab bar that can be hidden)
   - Hint overlays rendered as floating labels (use CSS or separate overlay window)

6. **Additional Modern Features**:
   - History & bookmarks (stored in SQLite via `rusqlite`)
   - Downloads manager
   - Private/incognito mode
   - Dark mode following system
   - JavaScript toggle, image toggle, etc.
   - `:open`, `:tabopen`, `:view-source`, `:inspect`, `:devtools`
   - Config reload at runtime (`:reload-config`)

**Technical Stack (exact crates)**

```toml
[dependencies]
gtk4 = "0.9"
adw = "0.7"                    # libadwaita
webkit6 = "0.4"                # or latest
gio = "0.20"
glib = "0.20"
adblock = "0.1"                # Brave's adblock-rust
luau-rs = "*"                  # or "lu" crate if better – pick the best
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"                   # optional fallback
dirs = "5.0"
reqwest = { version = "0.12", features = ["blocking", "json"] }
tokio = { version = "1", features = ["full"] }  # for async filter updates
```

**Project Structure** (generate all folders and files):

```
luavimb/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── build.rs                  # optional for Luau C++ build if needed
├── src/
│   ├── main.rs
│   ├── app.rs
│   ├── ui/
│   ├── webview.rs
│   ├── keyboard.rs           # mode manager + key handler
│   ├── hint.rs               # hint mode engine
│   ├── config/
│   │   ├── mod.rs
│   │   ├── luau_engine.rs    # Luau VM + Rust API bindings
│   │   └── default_config.lua
│   ├── adblock/
│   │   └── manager.rs
│   ├── commands.rs
│   ├── history.rs
│   └── utils.rs
├── config/
│   └── default_config.lua    # shipped default config
├── filters/                  # default filter list URLs
└── assets/
```

**Implementation Guidelines**

- Use GTK4 event controllers (`EventControllerKey`) for low-level key handling.
- Implement a clean state machine for the four modes.
- For hints: Inject a small JS script that finds all interactive elements, assign letters/numbers, then create overlay `GtkLabel` widgets positioned absolutely over the webview (or use WebKit’s JavaScriptCore to communicate).
- Luau integration must be safe, fast, and expose hundreds of Rust functions as Lua globals/tables.
- Adblock: On startup, load compiled filter lists into `WebKitUserContentManager`. Support live updates.
- Make the binary as small and fast-starting as possible (use `strip`, release profile optimizations).
- Include a `justfile` or detailed `README.md` with build instructions for Ubuntu/Debian, Arch, Fedora.

**Deliverables**  
Generate the **entire project** in a single response (or clearly marked multiple files). Include:

1. Full `Cargo.toml`
2. All source files with complete, compilable code and detailed comments
3. Default `config.lua` with many examples
4. Build & run instructions
5. How to extend with custom Luau plugins

Make the code clean, idiomatic Rust, well-documented, and ready to `cargo run`. Prioritize **minimal resource usage** and **Vim muscle memory fidelity**.

Start coding now.

---

Copy and paste the entire block above into your favorite LLM. It is engineered to produce a very high-quality, feature-complete starting point for your lightweight Vim + Luau browser.

If you want me to tweak the prompt (e.g., remove tabs for even more minimalism, add specific keybinds, or change the Luau crate), just say the word!
