#!/bin/sh
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec open "$here/launcher/src-tauri/target/release/bundle/macos/infamous1-recomp.app"
