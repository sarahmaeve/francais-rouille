#!/usr/bin/env bash
#
# Build script for Français Rouillé
#
# Compiles the Rust TTS CLI and verifies the site/ directory is ready for deployment.
# Audio generation requires GOOGLE_TTS_API_KEY to be set.
#
# Usage:
#   ./scripts/build.sh              # Build CLI only
#   ./scripts/build.sh --check      # Verify site integrity (broken links, missing audio)
#   ./scripts/build.sh --generate   # Regenerate all audio from content/ sources

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SITE_DIR="$REPO_ROOT/site"
CONTENT_DIR="$REPO_ROOT/content"

build_cli() {
    echo "Building Rust CLI..."
    cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"
    echo "CLI built: target/release/francais-rouille"
}

check_site() {
    echo "Checking site integrity..."
    local errors=0

    # Check that all audio files referenced in HTML actually exist
    while IFS= read -r html_file; do
        dir="$(dirname "$html_file")"
        while IFS= read -r audio_src; do
            audio_path="$dir/$audio_src"
            if [ ! -f "$audio_path" ]; then
                echo "  MISSING: $audio_src (referenced in $html_file)"
                errors=$((errors + 1))
            fi
        done < <(grep -oP 'src="\K[^"]*\.mp3' "$html_file" 2>/dev/null || true)
    done < <(find "$SITE_DIR" -name '*.html' -type f)

    # Check that shared assets exist
    for asset in quiz.js quiz.css crossword.js crossword.css; do
        if [ ! -f "$SITE_DIR/shared/$asset" ]; then
            echo "  MISSING: shared/$asset"
            errors=$((errors + 1))
        fi
    done

    # Check that each chapter has required files
    for chapter_dir in "$SITE_DIR"/chapters/*/; do
        chapter="$(basename "$chapter_dir")"
        for required in index.html style.css; do
            if [ ! -f "$chapter_dir/$required" ]; then
                echo "  MISSING: chapters/$chapter/$required"
                errors=$((errors + 1))
            fi
        done
    done

    if [ "$errors" -eq 0 ]; then
        echo "Site OK — no issues found."
    else
        echo "$errors issue(s) found."
        exit 1
    fi
}

generate_audio() {
    if [ -z "${GOOGLE_TTS_API_KEY:-}" ]; then
        echo "Error: GOOGLE_TTS_API_KEY not set" >&2
        exit 1
    fi

    local cli="$REPO_ROOT/target/release/francais-rouille"
    if [ ! -f "$cli" ]; then
        build_cli
    fi

    for chapter_content in "$CONTENT_DIR"/*/; do
        chapter_name="$(basename "$chapter_content")"
        audio_dir="$SITE_DIR/chapters/$chapter_name/audio"
        mkdir -p "$audio_dir"

        for txt_file in "$chapter_content"/*.txt; do
            [ -f "$txt_file" ] || continue
            base="$(basename "$txt_file" .txt)"
            output="$audio_dir/$base"

            if [ -d "$output" ] && [ -f "$output/combined.mp3" ]; then
                echo "Skipping $base (audio exists)"
                continue
            fi

            echo "Generating audio for $base..."
            "$cli" dialog "$txt_file" "$output"
        done
    done

    echo "Audio generation complete."
}

case "${1:-}" in
    --check)
        check_site
        ;;
    --generate)
        build_cli
        generate_audio
        ;;
    *)
        build_cli
        ;;
esac
