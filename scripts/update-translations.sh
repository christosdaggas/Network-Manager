#!/usr/bin/env bash
# Extract translatable strings and merge them into every catalog.
#
# Regenerates po/cd-network-manager.pot from the `tr("...")` and
# `ntr("...", "...", n)` calls in the Rust sources, then updates each
# po/<lang>.po listed in po/LINGUAS. Existing translations are kept; new
# strings are added untranslated and changed ones are marked fuzzy.
#
# Usage: scripts/update-translations.sh
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

for cmd in git xgettext msgmerge; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "'$cmd' is required (install: sudo dnf install git gettext)" >&2
        exit 1
    fi
done

POT="po/cd-network-manager.pot"
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')

# xgettext only recognises bare calls, so sources must use `tr(...)`,
# not `crate::tr(...)`.
git ls-files 'src/*.rs' | xargs xgettext \
    --language=Rust \
    --from-code=UTF-8 \
    --keyword= \
    --keyword=tr \
    --keyword=ntr:1,2 \
    --add-comments=TRANSLATORS: \
    --sort-by-file \
    --package-name="network-manager" \
    --package-version="$VERSION" \
    --msgid-bugs-address="https://github.com/christosdaggas/Network-Manager/issues" \
    --copyright-holder="Christos A. Daggas" \
    -o "$POT"

echo "Extracted $(grep -c '^msgid "' "$POT") strings into $POT (including the header)"

grep -v '^#' po/LINGUAS | grep -v '^$' | while read -r lang; do
    msgmerge --quiet --update --backup=none --no-fuzzy-matching "po/$lang.po" "$POT"
    echo "Updated po/$lang.po"
done
