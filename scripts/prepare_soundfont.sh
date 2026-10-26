#!/usr/bin/env bash
# Build Toast Busters' bank, assets/soundfonts/toast_busters.sf2, and the LICENSE beside it,
# through the engine's scripts/prepare_game_soundfont.sh: the FluidR3 GM presets Danse Macabre
# plays. The subset is committed; this script is how anyone rebuilds it byte for byte.
#
# Usage: scripts/prepare_soundfont.sh [--bank path/to/FluidR3_GM.sf2]
#   The engine's script documents the fetch, the sha256 check and the 4 MiB cap.
#
# The presets, as bank:program: xylophone (the rattling bones, a Plain Wraith busted),
# clarinet (the solo violin's tune, a Rye-I.P. busted, a cannon hit, the last chord, the
# jam), tubular bells (midnight as a flyer enters, the bonus, a Bagelgeist busted),
# acoustic bass (the waltz's oom-pah-pah, a shot's pluck, a Sour-Doom singed and busted).
# No violin and no strings: any of them beside the xylophone is over 4 MiB.
set -euo pipefail

GAME_DIR="$(cd "$(dirname "$0")/.." && pwd)"
ENGINE_DIR="$(cd "$GAME_DIR/../../insiculous_2d" && pwd)"
OUTPUT_DIR="$GAME_DIR/assets/soundfonts"

exec "$ENGINE_DIR/scripts/prepare_game_soundfont.sh" "$@" --max-mib 4 "$OUTPUT_DIR" toast_busters 0:13 0:71 0:14 0:32
