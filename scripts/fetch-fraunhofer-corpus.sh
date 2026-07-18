#!/usr/bin/env bash
#
# Fetch the Fraunhofer SMPTE working-group ST 2067-203 / ST 2067-204
# test corpus, published alongside the IMFTool project. Ten MXFs total
# (~440 MB) — six HD video track files and four audio track files
# covering S-ADM and IAB / ADM audio essence.
#
# Corpus authored by the SMPTE ST 2067 working group, distributed under
# CC-BY-NC-ND 4.0 by Fraunhofer-Gesellschaft. See LICENSE.txt in the
# destination directory after fetch. Use is limited to non-commercial
# testing / research; you can't redistribute modified versions.
#
# The corpus is NOT vendored in the imferno repo (license + size). This
# script pulls it into an untracked directory for local integration
# testing.
#
# Usage:
#   scripts/fetch-fraunhofer-corpus.sh              # default target dir
#   scripts/fetch-fraunhofer-corpus.sh /custom/dir  # explicit target

set -euo pipefail

DEFAULT_TARGET="test-data/Fraunhofer-SMPTE-ST2067-203-204"
TARGET="${1:-$DEFAULT_TARGET}"

REPO="IMFTool/IMFTool"
BRANCH="master"
CORPUS_DIR="ADM_S-ADM_SampleTrackFiles/20240515_SMPTE_ST2067-203_and_ST2067-204_TestContent_Fraunhofer"
BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}/${CORPUS_DIR}"

FILES=(
    "LICENSE.txt"
    "ST2067-203_TestVector1_HD_50fps.mxf"
    "ST2067-203_TestVector2_HD_25fps.mxf"
    "ST2067-203_TestVector3_HD_50fps.mxf"
    "ST2067-203_audio_track_file_1.mxf"
    "ST2067-203_audio_track_file_2.mxf"
    "ST2067-204_TestVector1_HD_50fps.mxf"
    "ST2067-204_TestVector2_HD_25fps.mxf"
    "ST2067-204_TestVector3_HD_50fps.mxf"
    "ST2067-204_audio_track_file_1.mxf"
    "ST2067-204_audio_track_file_2.mxf"
)

mkdir -p "$TARGET"

echo "Fetching Fraunhofer SMPTE ST 2067-203/-204 corpus into: $TARGET"
echo "  source: https://github.com/${REPO}/tree/${BRANCH}/${CORPUS_DIR}"
echo "  license: CC-BY-NC-ND 4.0 (Fraunhofer-Gesellschaft, 2024)"
echo

need_download=0
for name in "${FILES[@]}"; do
    if [[ ! -f "$TARGET/$name" ]]; then
        need_download=1
        break
    fi
done

if [[ "$need_download" -eq 0 ]]; then
    echo "All 11 files already present, skipping download."
    echo "To force re-download: rm -rf $TARGET"
    exit 0
fi

# Parallel download; curl -f fails on 4xx/5xx so we get a clear error
# instead of a truncated file.
printf '%s\n' "${FILES[@]}" \
    | xargs -n1 -P4 -I{} sh -c 'curl -fsSL "$0/{}" -o "$1/{}" && echo "  ✓ {}"' "$BASE_URL" "$TARGET"

echo
echo "Fetched $(printf '%s\n' "${FILES[@]}" | wc -l | tr -d ' ') files into $TARGET"
echo "Run the corpus check with:"
echo "  cargo run --example fraunhofer_corpus_check -- $TARGET"
