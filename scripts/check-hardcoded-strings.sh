#!/bin/bash

# Exit on error
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

# Patterns to search for
PATTERNS=(
    'String::from("(?!test)'  # String::from with literal
    'to_string()"'           # Literal followed by to_string()
    '".*"\.to_string()'      # String literal with to_string()
)

# Excluded patterns (allowed hardcoded strings)
EXCLUDES=(
    'derive'
    'test'
    'cfg'
    'doc'
    'allow'
    'deny'
)

EXCLUDE_PATTERN=$(IFS=\|; echo "${EXCLUDES[*]}")

# Initialize error counter
ERRORS=0

for file in $(git diff --cached --name-only --diff-filter=ACMR | grep "\.rs$"); do
    for pattern in "${PATTERNS[@]}"; do
        # Search for pattern but exclude allowed cases
        FINDINGS=$(git diff --cached --unified=0 "$file" | \
                  grep -E "^\+" | \
                  grep -E "$pattern" | \
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