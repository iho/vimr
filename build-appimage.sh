#!/usr/bin/env bash
# Build a self-contained AppImage for vimr.
# Bundles WebKitGTK, GTK4, and all shared library dependencies.
# Run from the project root: ./build-appimage.sh
set -euo pipefail

APPDIR="vimr.AppDir"
OUT="vimr-$(uname -m).AppImage"
ARCH=$(uname -m)

# glibc and the dynamic linker must come from the target machine — exclude them.
EXCLUDE_PATTERNS=(
    "libc.so"
    "libm.so"
    "libpthread.so"
    "libdl.so"
    "librt.so"
    "libgcc_s.so"
    "ld-linux"
    "ld-musl"
    "libstdc++.so"   # comment out if target is very old
)

# ── helpers ─────────────────────────────────────────────────────────────────

log() { printf '\e[1;34m[appimage]\e[0m %s\n' "$*"; }
warn() { printf '\e[1;33m[appimage]\e[0m WARNING: %s\n' "$*"; }

should_exclude() {
    local lib="$1"
    for pat in "${EXCLUDE_PATTERNS[@]}"; do
        [[ "$lib" == *"$pat"* ]] && return 0
    done
    return 1
}

# Copy a library and its real symlink target into $APPDIR/usr/lib/
copy_lib() {
    local src="$1"
    [[ -f "$src" ]] || return
    should_exclude "$src" && return
    local real; real=$(readlink -f "$src")
    [[ -f "$real" ]] || return
    local base_real; base_real=$(basename "$real")
    local base_src;  base_src=$(basename "$src")
    cp -pn "$real" "$APPDIR/usr/lib/$base_real" 2>/dev/null || true
    # preserve the symlink name so loaders/dlopen find it
    if [[ "$base_src" != "$base_real" ]]; then
        (cd "$APPDIR/usr/lib" && ln -sf "$base_real" "$base_src" 2>/dev/null || true)
    fi
}

# Recursively collect and copy all shared-lib dependencies of $1
processed_libs=$(mktemp)
copy_deps() {
    local binary="$1"
    [[ -f "$binary" ]] || return
    while IFS= read -r lib; do
        [[ -f "$lib" ]]                         || continue
        should_exclude "$lib"                   && continue
        grep -qF "$lib" "$processed_libs"       && continue
        echo "$lib" >> "$processed_libs"
        copy_lib "$lib"
        copy_deps "$lib"          # recurse into the library's own deps
    done < <(ldd "$binary" 2>/dev/null \
             | grep "=>" \
             | awk '{print $3}' \
             | grep "^/")
}

# ── 1. build ─────────────────────────────────────────────────────────────────

log "Building release binary…"
cargo build --release
strip target/release/vimr

# ── 2. AppDir skeleton ───────────────────────────────────────────────────────

log "Creating AppDir…"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/lib"
mkdir -p "$APPDIR/usr/share/glib-2.0/schemas"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"
mkdir -p "$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders"
mkdir -p "$APPDIR/usr/lib/gio/modules"
mkdir -p "$APPDIR/webkit"

# ── 3. binary ────────────────────────────────────────────────────────────────

log "Copying vimr binary…"
cp target/release/vimr "$APPDIR/usr/bin/"

# ── 4. WebKit subprocesses ───────────────────────────────────────────────────

log "Locating WebKit 6 subprocesses…"
WEBKIT_DIR=""
for candidate in \
    "/usr/lib/webkitgtk-6.0" \
    "/usr/lib64/webkitgtk-6.0" \
    "$(pkg-config --variable=libdir webkitgtk-6.0 2>/dev/null || true)/webkitgtk-6.0"
do
    if [[ -f "$candidate/WebKitWebProcess" ]]; then
        WEBKIT_DIR="$candidate"
        break
    fi
done

if [[ -z "$WEBKIT_DIR" ]]; then
    warn "WebKitWebProcess not found — browser will not render pages"
else
    log "WebKit dir: $WEBKIT_DIR"
    for proc in WebKitWebProcess WebKitNetworkProcess WebKitGPUProcess; do
        [[ -f "$WEBKIT_DIR/$proc" ]] && cp "$WEBKIT_DIR/$proc" "$APPDIR/webkit/"
    done
    [[ -d "$WEBKIT_DIR/injected-bundle" ]] && \
        cp -r "$WEBKIT_DIR/injected-bundle" "$APPDIR/webkit/"

    # Copy any extra data files in the webkit dir (not subdirs already handled)
    for f in "$WEBKIT_DIR"/*; do
        [[ -d "$f" ]] && continue
        base=$(basename "$f")
        [[ "$base" == WebKit*Process ]] && continue
        cp "$f" "$APPDIR/webkit/" 2>/dev/null || true
    done
fi

# ── 5. shared libraries ──────────────────────────────────────────────────────

log "Collecting shared libraries (this takes a moment)…"
copy_deps "$APPDIR/usr/bin/vimr"
for proc in WebKitWebProcess WebKitNetworkProcess WebKitGPUProcess; do
    [[ -f "$APPDIR/webkit/$proc" ]] && copy_deps "$APPDIR/webkit/$proc"
done

# Also sweep the injected bundle
[[ -d "$APPDIR/webkit/injected-bundle" ]] && \
    find "$APPDIR/webkit/injected-bundle" -name "*.so*" -exec bash -c 'copy_deps "$1"' _ {} \; 2>/dev/null || true

# Extra passes — catch anything the first sweep missed
for _pass in 1 2; do
    find "$APPDIR/usr/lib" -name "*.so*" -type f | while read -r lib; do
        copy_deps "$lib"
    done
done
rm -f "$processed_libs"

# ── 6. GDK-pixbuf loaders ────────────────────────────────────────────────────

log "Bundling GDK-pixbuf loaders…"
PIXBUF_SRC=$(pkg-config --variable=gdk_pixbuf_moduledir gdk-pixbuf-2.0 2>/dev/null \
             || find /usr/lib -path "*/gdk-pixbuf-2.0/*/loaders" -type d 2>/dev/null | head -1)

if [[ -d "$PIXBUF_SRC" ]]; then
    cp "$PIXBUF_SRC"/*.so "$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders/" 2>/dev/null || true
    # Build cache with @APPDIR@ placeholder — replaced at runtime in AppRun
    GDK_PIXBUF_MODULEDIR="$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders" \
        gdk-pixbuf-query-loaders 2>/dev/null \
        | sed "s|$APPDIR|@APPDIR@|g" \
        > "$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders.cache.template" \
        || true
fi

# ── 7. GLib schemas ──────────────────────────────────────────────────────────

log "Compiling GLib schemas…"
# Gather all schemas into a temp dir and compile
SCHEMA_TMP=$(mktemp -d)
cp /usr/share/glib-2.0/schemas/*.xml "$SCHEMA_TMP/" 2>/dev/null || true
glib-compile-schemas "$SCHEMA_TMP" 2>/dev/null && \
    cp "$SCHEMA_TMP/gschemas.compiled" "$APPDIR/usr/share/glib-2.0/schemas/" || true
rm -rf "$SCHEMA_TMP"

# ── 8. GIO TLS modules ───────────────────────────────────────────────────────

log "Bundling GIO modules…"
GIO_MOD_SRC=$(pkg-config --variable=giomoduledir gio-2.0 2>/dev/null \
              || find /usr/lib -path "*/gio/modules" -type d 2>/dev/null | head -1)
[[ -d "$GIO_MOD_SRC" ]] && \
    cp "$GIO_MOD_SRC"/*.so "$APPDIR/usr/lib/gio/modules/" 2>/dev/null || true

# ── 9. icon ──────────────────────────────────────────────────────────────────

log "Creating icon…"
if command -v convert &>/dev/null; then
    convert -size 256x256 xc:'#1a1a1a' \
        -fill '#7ec8e3' -font DejaVu-Sans-Mono-Bold \
        -pointsize 140 -gravity Center -annotate 0 'V' \
        "$APPDIR/vimr.png" 2>/dev/null \
    || convert -size 256x256 xc:'#1a1a1a' "$APPDIR/vimr.png"
else
    # Minimal valid PNG (1×1 blue pixel — appimagetool just needs a file)
    printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\x0f\x00\x00\x01\x01\x00\x05\x18\xd8N\x00\x00\x00\x00IEND\xaeB`\x82' \
        > "$APPDIR/vimr.png"
fi
cp "$APPDIR/vimr.png" "$APPDIR/usr/share/icons/hicolor/256x256/apps/vimr.png"

# ── 10. .desktop ─────────────────────────────────────────────────────────────

cat > "$APPDIR/vimr.desktop" << 'EOF'
[Desktop Entry]
Name=vimr
Comment=Vim-like browser
Exec=vimr %U
Icon=vimr
Type=Application
Categories=Network;WebBrowser;
MimeType=text/html;text/xml;application/xhtml+xml;x-scheme-handler/http;x-scheme-handler/https;
StartupNotify=true
EOF

# ── 11. AppRun ───────────────────────────────────────────────────────────────

cat > "$APPDIR/AppRun" << 'APPRUN'
#!/bin/bash
APPDIR="$(dirname "$(readlink -f "$0")")"

# ── library path ─────────────────────────────────────────────────────────────
export LD_LIBRARY_PATH="$APPDIR/usr/lib:${LD_LIBRARY_PATH:-}"

# ── WebKit subprocess dir ─────────────────────────────────────────────────────
export WEBKIT_EXEC_PATH="$APPDIR/webkit"

# ── GTK / GLib data ──────────────────────────────────────────────────────────
export GSETTINGS_SCHEMA_DIR="$APPDIR/usr/share/glib-2.0/schemas"
export XDG_DATA_DIRS="$APPDIR/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

# ── GDK-pixbuf loaders ────────────────────────────────────────────────────────
CACHE_TEMPLATE="$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders.cache.template"
if [[ -f "$CACHE_TEMPLATE" ]]; then
    CACHE_FILE="/tmp/vimr-pixbuf-loaders-$$.cache"
    sed "s|@APPDIR@|$APPDIR|g" "$CACHE_TEMPLATE" > "$CACHE_FILE"
    export GDK_PIXBUF_MODULE_FILE="$CACHE_FILE"
    export GDK_PIXBUF_MODULEDIR="$APPDIR/usr/lib/gdk-pixbuf-2.0/loaders"
    trap 'rm -f "$CACHE_FILE"' EXIT
fi

# ── GIO modules (TLS, DNS…) ──────────────────────────────────────────────────
[[ -d "$APPDIR/usr/lib/gio/modules" ]] && \
    export GIO_EXTRA_MODULES="$APPDIR/usr/lib/gio/modules"

# ── rendering fixes ──────────────────────────────────────────────────────────
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export WEBKIT_DISABLE_COMPOSITING_MODE=1

exec "$APPDIR/usr/bin/vimr" "$@"
APPRUN
chmod +x "$APPDIR/AppRun"

# ── 12. appimagetool ─────────────────────────────────────────────────────────

TOOL="appimagetool-${ARCH}.AppImage"
if [[ ! -f "$TOOL" ]]; then
    log "Downloading appimagetool…"
    curl -fsSL -o "$TOOL" \
        "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-${ARCH}.AppImage"
    chmod +x "$TOOL"
fi

log "Packaging $OUT…"
ARCH="$ARCH" ./"$TOOL" "$APPDIR" "$OUT" 2>&1

echo ""
log "Done!"
log "  File : $OUT"
log "  Size : $(du -sh "$OUT" | cut -f1)"
log ""
log "  Copy to any x86_64 Linux machine and run:"
log "    chmod +x $OUT && ./$OUT"
