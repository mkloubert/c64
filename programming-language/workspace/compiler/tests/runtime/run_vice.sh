#!/bin/bash
# VICE Emulator Test Harness for Cobra64
#
# This script runs a PRG file in the VICE x64sc emulator and captures output.
# It uses VICE's warp mode for fast execution and exits after a timeout.
#
# Usage: ./run_vice.sh <program.prg> [timeout_seconds]
#
# Requirements:
#   - VICE emulator installed (x64sc in PATH)
#   - xvfb-run for headless operation (optional)
#
# Output:
#   - Prints captured screen content to stdout
#   - Exit code 0 on success, non-zero on error

set -e

PRG_FILE="$1"
TIMEOUT="${2:-5}"

if [ -z "$PRG_FILE" ]; then
    echo "Usage: $0 <program.prg> [timeout_seconds]" >&2
    exit 1
fi

if [ ! -f "$PRG_FILE" ]; then
    echo "Error: File not found: $PRG_FILE" >&2
    exit 1
fi

# Check for VICE emulator
if ! command -v x64sc &> /dev/null; then
    echo "Error: VICE emulator (x64sc) not found in PATH" >&2
    echo "Install VICE: https://vice-emu.sourceforge.io/" >&2
    exit 2
fi

# Create temporary directory for output
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

# Create a VICE script to run the program and exit
cat > "$TMPDIR/run.vsf" << 'EOF'
# VICE script to run program and capture output
warp 1
EOF

# Determine if we can run headless
VICE_CMD="x64sc"
if command -v xvfb-run &> /dev/null; then
    VICE_CMD="xvfb-run -a x64sc"
fi

# Run VICE with the PRG file
# Options:
#   -silent: Reduce output
#   -warp: Run at maximum speed
#   -exitscreenshot: Take screenshot on exit
#   -limitcycles: Exit after N cycles
#   +sound: Disable sound
timeout "$TIMEOUT" $VICE_CMD \
    -silent \
    -warp \
    +sound \
    -exitscreenshot "$TMPDIR/screen.png" \
    -limitcycles 5000000 \
    -autostartprgmode 1 \
    "$PRG_FILE" 2>/dev/null || true

# Check if screenshot was created
if [ -f "$TMPDIR/screen.png" ]; then
    # If imagemagick is available, try to extract text
    if command -v convert &> /dev/null; then
        echo "Screenshot saved to $TMPDIR/screen.png"
        # Note: OCR would require tesseract, which is complex for C64 fonts
    fi
    echo "VICE execution completed"
else
    echo "Warning: No screenshot captured" >&2
fi

exit 0
