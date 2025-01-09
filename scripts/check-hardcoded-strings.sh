#!/bin/bash

# Exit on error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Patterns to search for string literals
PATTERNS=(
    '"[^"]*"'               # Match any string literal
)

# Excluded patterns (allowed hardcoded strings)
EXCLUDES=(
    '#\[derive'
    '#\[test'
    '#\[cfg'
    '#\[doc'
    '#\[allow'
    '#\[deny'
    'r#"'                   # Raw string literals
    'include_str!'          # Include string macro
    'env!'                  # Environment variable macro
    'concat!'               # Concatenation macro
    'format!'               # Format macro
)

# Excluded files
EXCLUDED_FILES=(
    'src/common/i18n.rs'
    'src/tests/'
    'benches/'
)

# Build exclude pattern for grep
EXCLUDE_PATTERN=$(printf "|%s" "${EXCLUDES[@]}")
EXCLUDE_PATTERN=${EXCLUDE_PATTERN:1}

# Initialize error counter
ERRORS=0

for file in $(git diff --cached --name-only --diff-filter=ACMR | grep "\.rs$"); do
    # Skip excluded files
    for excluded in "${EXCLUDED_FILES[@]}"; do
        if [[ "$file" == *"$excluded"* ]]; then
            continue 2
        fi
    done
    
    # First, get the added lines
    ADDED_LINES=$(git diff --cached --unified=0 "$file" | grep -E "^\+" || true)
    
    if [ ! -z "$ADDED_LINES" ]; then
        for pattern in "${PATTERNS[@]}"; do
            # Search for string literals in added lines
            FINDINGS=$(echo "$ADDED_LINES" | grep -E "$pattern" || true)
            
            if [ ! -z "$FINDINGS" ]; then
                # Filter out excluded patterns
                FILTERED_FINDINGS=$(echo "$FINDINGS" | grep -Ev "$EXCLUDE_PATTERN" || true)
                
                if [ ! -z "$FILTERED_FINDINGS" ]; then
                    echo -e "${RED}Found potential hardcoded strings in $file:${NC}"
                    echo "$FILTERED_FINDINGS"
                    ERRORS=$((ERRORS + 1))
                fi
            fi
        done
    fi
done

if [ $ERRORS -gt 0 ]; then
    echo -e "${RED}Error: Found $ERRORS potential hardcoded string(s)${NC}"
    echo "Please use i18n messages instead of hardcoded strings"
    exit 1
else
    echo -e "${GREEN}No hardcoded strings found${NC}"
    exit 0
fi 