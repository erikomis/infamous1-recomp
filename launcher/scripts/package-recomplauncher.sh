#!/bin/zsh
# The release zip RecompLauncher installs from: "infamous1-recomp.app" and
# recomplauncher.json (the repository's) at the top, named
# infamous1-recomp-<launcher version>-macos-arm64.zip. Local only: it builds
# and writes a file, it never uploads or publishes anything (attaching the
# zip to a GitHub release is a separate, manual step).
#
#   scripts/package-recomplauncher.sh [--app <infamous1-recomp.app>] [--out <folder>]
#
#   --app  an .app built already; without it the release .app is built here
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
        -h|--help) sed -n '2,18p' $0 | sed 's/^# \{0,1\}//'; exit 0 ;;
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
    print -r -- "==> building $name (release)"
    # stripping the release binary breaks the link on this toolchain
    # ("mis-aligned LINKEDIT"); the symbols stay in
    (cd "$launcher" && CARGO_PROFILE_RELEASE_STRIP=false npx tauri build --bundles app \
        --config "$launcher/src-tauri/tauri.bundle.conf.json")
    app=$launcher/src-tauri/target/release/bundle/macos/$name
fi
[ -d "$app" ] || fail "$app: no such .app"
[ "${app:t}" = "$name" ] || fail "$app: the .app must be called $name (recomplauncher.json starts $program)"
exe=$app/${program#*/}
[ -x "$exe" ] || fail "$exe: missing or not executable"
/usr/bin/lipo -archs "$exe" | grep -qw arm64 || fail "$exe: not built for Apple silicon (arm64)"
[ -f "$app/Contents/Resources/port/infamous1-lle/tools/rebuild.py" ] ||
    fail "$app has no port/infamous1-lle/tools/rebuild.py (built without tauri.bundle.conf.json?): --prepare needs it"
/usr/bin/python3 -I "$here/check_bundle.py" "$app" || fail "$app carries something from the game"

mkdir -p "$out"
zip=$out/infamous1-recomp-$version-macos-arm64.zip
stage=$(mktemp -d "${TMPDIR:-/tmp}/inf1-package.XXXXXX")
trap 'rm -rf "$stage"' EXIT
# ditto keeps the bundle as it is (symlinks, permissions, its signature)
/usr/bin/ditto "$app" "$stage/$name"
cp "$profile" "$stage/recomplauncher.json"
rm -f "$zip"
/usr/bin/ditto -c -k --norsrc --noextattr "$stage" "$zip"

print -r -- "==> $zip"
/usr/bin/zipinfo -1 "$zip" | grep -E '^[^/]+/?$|^[^/]+\.app/Contents/MacOS/[^/]+$'
/usr/bin/shasum -a 256 "$zip"
print -r -- "Not uploaded: attach it to a GitHub release (tag v$version) by hand when you publish one."
