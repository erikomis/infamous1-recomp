#!/bin/zsh
# The release zip RecompLauncher installs from: "infamous1-recomp.app" and
# recomplauncher.json (the repository's) at the top, named
# infamous1-recomp-<launcher version>-macos-arm64.zip. Local only: it builds
# and writes a file, it never uploads or publishes anything (attaching the
# zip to a GitHub release is a separate, manual step).
#
#   scripts/package-recomplauncher.sh [--app <infamous1-recomp.app>] [--out <folder>]
#
#   --app  an .app built already (it must not hold this machine's home
#          folder: the check refuses it); without it the release .app is built
#          from a copy under /Users/Shared with $HOME remapped
#          (scripts/stage_port.py, then npx tauri build --bundles app with
#          src-tauri/tauri.bundle.conf.json, so port/ is in the bundle)
#   --out  where the zip goes (default: launcher/release/, git-ignored)
#
# Before zipping it checks the .app: its program is where
# recomplauncher.json's start and prepare say, it runs on Apple silicon, it
# carries port/infamous1-lle/tools/rebuild.py (what --prepare runs), and
# scripts/check_bundle.py finds nothing from the game in it.
set -e
setopt pipefail
here=${0:A:h}; launcher=${here:h}; repo=${launcher:h}
app=; out=$launcher/release
while (( $# )); do
    case $1 in
        --app) app=${2:A}; shift 2 ;;
        --out) out=${2:A}; shift 2 ;;
        -h|--help) sed -n '2,21p' $0 | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) print -u2 "unknown argument: $1 (see --help)"; exit 2 ;;
    esac
done
fail() { print -u2 -r -- "ERROR: $*"; exit 1; }

profile=$repo/recomplauncher.json
[ -f "$profile" ] || fail "no $profile"
# the version and the program the profile starts, read with Python's json
version=$(/usr/bin/python3 -I -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$launcher/src-tauri/tauri.conf.json") ||
    fail "no version in src-tauri/tauri.conf.json"
program=$(/usr/bin/python3 -I -c '
import json, sys
p = json.load(open(sys.argv[1]))
starts = {p["start"]["program"][0], p["prepare"]["program"][0]}
assert len(starts) == 1, "start and prepare name different programs"
print(starts.pop())' "$profile") || fail "$profile: unreadable, or start and prepare differ"
name=${program%%/*}   # infamous1-recomp.app

if [ -z "$app" ]; then
    print -r -- "==> staging the port's sources (port/)"
    /usr/bin/python3 -I "$here/stage_port.py" --out "$launcher/stage/port"
    # A local build writes where it was built into the binary: the
    # checkout's path (env!("CARGO_MANIFEST_DIR")) and, in panic messages,
    # ~/.cargo/registry and ~/.rustup -- the account's name in a public
    # zip. So it builds from a copy under /Users/Shared, with $HOME
    # remapped, and the check below refuses any file that still has it. The
    # copy's folder is private and new (mktemp, 0700, ours): /Users/Shared is
    # writable by every account, and a folder of a fixed name there could be
    # someone else's.
    build=$(/usr/bin/mktemp -d /Users/Shared/infamous1-recomp-build.XXXXXXXX) || fail "mktemp in /Users/Shared failed"
    chmod 700 "$build"
    [ -O "$build" ] && [ ! -L "$build" ] || fail "$build is not this account's own folder"
    trap 'rm -rf "$build"' EXIT
    print -r -- "==> copying the launcher to $build (a path without the account's name)"
    /usr/bin/rsync -a --delete --exclude 'src-tauri/target/' --exclude 'release/' "$launcher/" "$build/launcher/"
    print -r -- "==> building $name (release)"
    # The toolchain's linker corrupts stripped release binaries and proc-macro
    # dylibs here ("mis-aligned LINKEDIT"): nothing is stripped.
    (cd "$build/launcher" && \
        RUSTFLAGS="--remap-path-prefix=$HOME=/home/build --remap-path-prefix=$build=/build" \
        CARGO_PROFILE_RELEASE_STRIP=false CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP=false \
        npx tauri build --bundles app --config "$build/launcher/src-tauri/tauri.bundle.conf.json")
    app=$build/launcher/src-tauri/target/release/bundle/macos/$name
    # The debug map still names the object files the link used, among them
    # Rust's own (~/.rustup) and C ones compiled in ~/.cargo/registry: the
    # debug symbols go (strip -S, after the link: not the step the linker
    # bug hits), and the bundle is signed again, ad hoc as tauri left it.
    for bin in "$app"/Contents/MacOS/*(N.); do /usr/bin/strip -S "$bin" || fail "strip -S $bin failed"; done
    /usr/bin/codesign --force --sign - "$app" >/dev/null 2>&1 || fail "codesign $app failed"
fi
[ -d "$app" ] || fail "$app: no such .app"
[ "${app:t}" = "$name" ] || fail "$app: the .app must be called $name (recomplauncher.json starts $program)"
exe=$app/${program#*/}
[ -x "$exe" ] || fail "$exe: missing or not executable"
/usr/bin/lipo -archs "$exe" | grep -qw arm64 || fail "$exe: not built for Apple silicon (arm64)"
[ -f "$app/Contents/Resources/port/infamous1-lle/tools/rebuild.py" ] ||
    fail "$app has no port/infamous1-lle/tools/rebuild.py (built without tauri.bundle.conf.json?): --prepare needs it"
/usr/bin/python3 -I "$here/check_bundle.py" "$app" || fail "$app carries something from the game"
# Nothing of the machine it was built on: no file in the .app may hold the
# home folder or the account's name.
leaks=$(/usr/bin/find "$app" -type f -exec /usr/bin/grep -l -a -e "$HOME" -e "/Users/$USER/" {} + 2>/dev/null || true)   # grep: 1 = none found
[ -z "$leaks" ] || fail "$app has this machine's home folder in: ${leaks//$'\n'/, } (build it with this script, not from the checkout)"

mkdir -p "$out"
zip=$out/infamous1-recomp-$version-macos-arm64.zip
stage=$(mktemp -d "${TMPDIR:-/tmp}/inf1-package.XXXXXX")
trap 'rm -rf "$stage" ${build:+"$build"}' EXIT
# ditto keeps the bundle as it is (symlinks, permissions, its signature)
/usr/bin/ditto "$app" "$stage/$name"
cp "$profile" "$stage/recomplauncher.json"
rm -f "$zip"
/usr/bin/ditto -c -k --norsrc --noextattr "$stage" "$zip"

print -r -- "==> $zip"
/usr/bin/zipinfo -1 "$zip" | grep -E '^[^/]+/?$|^[^/]+\.app/Contents/MacOS/[^/]+$'
/usr/bin/shasum -a 256 "$zip"
if /usr/bin/zipinfo "$zip" | /usr/bin/grep -q -e "$USER"; then fail "$zip: a file name holds the account's name"; fi
print -r -- "Not uploaded: attach it to a GitHub release (tag v$version) by hand when you publish one."
