#!/bin/bash

# Exit on error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Patterns to search for (using fixed strings where possible)
PATTERNS=(
    'String::from("'         # String::from with literal
    '.to_string()'          # to_string() calls
)

# Excluded patterns (allowed hardcoded strings)
EXCLUDES=(
    '#\[derive'
    '#\[test'
    '#\[cfg'
    '#\[doc'
    '#\[allow'
    '#\[deny'
)

# Excluded files
EXCLUDED_FILES=(
    'src/common/i18n.rs'
)

# Build exclude pattern for grep
EXCLUDE_PATTERN=$(printf "|%s" "${EXCLUDES[@]}")
EXCLUDE_PATTERN=${EXCLUDE_PATTERN:1}

# Initialize error counter
ERRORS=0

for file in $(git diff --cached --name-only --diff-filter=ACMR | grep "\.rs$"); do
    # Skip excluded files
    if [[ " ${EXCLUDED_FILES[@]} " =~ " ${file} " ]]; then
        continue
    fi
    
    for pattern in "${PATTERNS[@]}"; do
        # Search for pattern but exclude allowed cases
        # Using -F for fixed strings where possible to avoid regex issues
        FINDINGS=$(git diff --cached --unified=0 "$file" | \
                  grep -E "^\+" | \
                  grep -F "$pattern" | \
                  grep -Ev "$EXCLUDE_PATTERN" || true)
        
        if [ ! -z "$FINDINGS" ]; then
            echo -e "${RED}Found potential hardcoded strings in $file:${NC}"
            echo "$FINDINGS"
            ERRORS=$((ERRORS + 1))
        fi
    done
done

if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}Error: Found $ERRORS potential hardcoded string(s)${NC}"
    echo "Please use i18n messages instead of hardcoded strings"
    exit 1
else
    echo -e "${GREEN}No hardcoded strings found${NC}"
    exit 0
fi 