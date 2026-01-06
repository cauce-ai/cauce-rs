#!/usr/bin/env bash

# Consolidated prerequisite checking script
#
# This script provides unified prerequisite checking for Spec-Driven Development workflow.
# It replaces the functionality previously spread across multiple scripts.
#
# Usage: ./check-prerequisites.sh [OPTIONS]
#
# OPTIONS:
#   --json                  Output in JSON format
#   --require-tasks         Require tasks.md to exist (for implementation phase)
#   --include-tasks         Include tasks.md in AVAILABLE_DOCS list
#   --paths-only            Only output path variables (no validation)
#   --validate-constitution Validate Constitution Check gates in plan.md
#   --validate-coverage     Validate coverage tool is specified in plan.md
#   --help, -h              Show help message
#
# OUTPUTS:
#   JSON mode: {"FEATURE_DIR":"...", "AVAILABLE_DOCS":["..."]}
#   Text mode: FEATURE_DIR:... \n AVAILABLE_DOCS: \n ✓/✗ file.md
#   Paths only: REPO_ROOT: ... \n BRANCH: ... \n FEATURE_DIR: ... etc.
#
# CONSTITUTION VALIDATION (--validate-constitution):
#   Parses plan.md to verify all 11 Constitution principles are addressed:
#   - Each principle (I-XI) must be marked as "Pass" or "N/A"
#   - Blocking violations are flagged for review
#
# COVERAGE VALIDATION (--validate-coverage):
#   Verifies plan.md has proper TDD configuration per Principle XI:
#   - Coverage Tool must be specified (not NEEDS CLARIFICATION)
#   - Coverage Threshold must be 95%

set -e

# Parse command line arguments
JSON_MODE=false
REQUIRE_TASKS=false
INCLUDE_TASKS=false
PATHS_ONLY=false
VALIDATE_CONSTITUTION=false
VALIDATE_COVERAGE=false

for arg in "$@"; do
    case "$arg" in
        --json)
            JSON_MODE=true
            ;;
        --require-tasks)
            REQUIRE_TASKS=true
            ;;
        --include-tasks)
            INCLUDE_TASKS=true
            ;;
        --paths-only)
            PATHS_ONLY=true
            ;;
        --validate-constitution)
            VALIDATE_CONSTITUTION=true
            ;;
        --validate-coverage)
            VALIDATE_COVERAGE=true
            ;;
        --help|-h)
            cat << 'EOF'
Usage: check-prerequisites.sh [OPTIONS]

Consolidated prerequisite checking for Spec-Driven Development workflow.

OPTIONS:
  --json                  Output in JSON format
  --require-tasks         Require tasks.md to exist (for implementation phase)
  --include-tasks         Include tasks.md in AVAILABLE_DOCS list
  --paths-only            Only output path variables (no prerequisite validation)
  --validate-constitution Validate Constitution Check gates in plan.md (all must be Pass/N/A)
  --validate-coverage     Validate coverage tool is specified in plan.md (not NEEDS CLARIFICATION)
  --help, -h              Show this help message

EXAMPLES:
  # Check task prerequisites (plan.md required)
  ./check-prerequisites.sh --json

  # Check implementation prerequisites (plan.md + tasks.md required)
  ./check-prerequisites.sh --json --require-tasks --include-tasks

  # Validate constitution compliance before implementation
  ./check-prerequisites.sh --validate-constitution --validate-coverage

  # Get feature paths only (no validation)
  ./check-prerequisites.sh --paths-only

EOF
            exit 0
            ;;
        *)
            echo "ERROR: Unknown option '$arg'. Use --help for usage information." >&2
            exit 1
            ;;
    esac
done

# Source common functions
SCRIPT_DIR="$(CDPATH="" cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/common.sh"

# Get feature paths and validate branch
eval $(get_feature_paths)
check_feature_branch "$CURRENT_BRANCH" "$HAS_GIT" || exit 1

# If paths-only mode, output paths and exit (support JSON + paths-only combined)
if $PATHS_ONLY; then
    if $JSON_MODE; then
        # Minimal JSON paths payload (no validation performed)
        printf '{"REPO_ROOT":"%s","BRANCH":"%s","FEATURE_DIR":"%s","FEATURE_SPEC":"%s","IMPL_PLAN":"%s","TASKS":"%s"}\n' \
            "$REPO_ROOT" "$CURRENT_BRANCH" "$FEATURE_DIR" "$FEATURE_SPEC" "$IMPL_PLAN" "$TASKS"
    else
        echo "REPO_ROOT: $REPO_ROOT"
        echo "BRANCH: $CURRENT_BRANCH"
        echo "FEATURE_DIR: $FEATURE_DIR"
        echo "FEATURE_SPEC: $FEATURE_SPEC"
        echo "IMPL_PLAN: $IMPL_PLAN"
        echo "TASKS: $TASKS"
    fi
    exit 0
fi

# Validate required directories and files
if [[ ! -d "$FEATURE_DIR" ]]; then
    echo "ERROR: Feature directory not found: $FEATURE_DIR" >&2
    echo "Run /speckit.specify first to create the feature structure." >&2
    exit 1
fi

if [[ ! -f "$IMPL_PLAN" ]]; then
    echo "ERROR: plan.md not found in $FEATURE_DIR" >&2
    echo "Run /speckit.plan first to create the implementation plan." >&2
    exit 1
fi

# Check for tasks.md if required
if $REQUIRE_TASKS && [[ ! -f "$TASKS" ]]; then
    echo "ERROR: tasks.md not found in $FEATURE_DIR" >&2
    echo "Run /speckit.tasks first to create the task list." >&2
    exit 1
fi

# Validate Constitution Check gates if requested
if $VALIDATE_CONSTITUTION; then
    echo "Validating Constitution Check gates in plan.md..." >&2

    # Extract Constitution Check section from plan.md
    # Look for gates that are not marked as Pass or N/A
    constitution_errors=()

    # Check each principle gate (I through XI)
    for principle in "I" "II" "III" "IV" "V" "VI" "VII" "VIII" "IX" "X" "XI"; do
        # Look for the principle line in the Constitution Check table
        gate_line=$(grep -E "^\| ${principle}\. " "$IMPL_PLAN" 2>/dev/null || echo "")

        if [[ -z "$gate_line" ]]; then
            constitution_errors+=("Principle ${principle}: Not found in Constitution Check table")
        elif ! echo "$gate_line" | grep -qE "(Pass|N/A)"; then
            constitution_errors+=("Principle ${principle}: Not marked as Pass or N/A")
        fi
    done

    # Check for blocking violations section
    if grep -q "BLOCKING VIOLATIONS" "$IMPL_PLAN" 2>/dev/null; then
        blocking_section=$(sed -n '/BLOCKING VIOLATIONS/,/^##/p' "$IMPL_PLAN" | grep -v "^##" | grep -v "BLOCKING VIOLATIONS" | grep -v "^$" | head -5)
        if [[ -n "$blocking_section" ]] && ! echo "$blocking_section" | grep -qiE "(none|n/a)"; then
            constitution_errors+=("Blocking violations documented - review before proceeding")
        fi
    fi

    if [[ ${#constitution_errors[@]} -gt 0 ]]; then
        echo "ERROR: Constitution Check validation failed:" >&2
        for err in "${constitution_errors[@]}"; do
            echo "  - $err" >&2
        done
        echo "Update plan.md to address all Constitution principles before proceeding." >&2
        exit 1
    fi

    echo "✓ All Constitution Check gates validated" >&2
fi

# Validate coverage tool specification if requested
if $VALIDATE_COVERAGE; then
    echo "Validating coverage tool specification in plan.md..." >&2

    coverage_errors=()

    # Check Coverage Tool field
    coverage_tool=$(grep -E "^\*\*Coverage Tool\*\*:" "$IMPL_PLAN" 2>/dev/null | head -1 || echo "")

    if [[ -z "$coverage_tool" ]]; then
        coverage_errors+=("Coverage Tool field not found in plan.md")
    elif echo "$coverage_tool" | grep -qiE "NEEDS CLARIFICATION"; then
        coverage_errors+=("Coverage Tool is marked as NEEDS CLARIFICATION")
    fi

    # Check Coverage Threshold field
    coverage_threshold=$(grep -E "^\*\*Coverage Threshold\*\*:" "$IMPL_PLAN" 2>/dev/null | head -1 || echo "")

    if [[ -z "$coverage_threshold" ]]; then
        coverage_errors+=("Coverage Threshold field not found in plan.md")
    elif ! echo "$coverage_threshold" | grep -qE "95%"; then
        coverage_errors+=("Coverage Threshold must be 95% per Constitution Principle XI")
    fi

    if [[ ${#coverage_errors[@]} -gt 0 ]]; then
        echo "ERROR: Coverage validation failed:" >&2
        for err in "${coverage_errors[@]}"; do
            echo "  - $err" >&2
        done
        echo "Update plan.md to specify coverage tool and 95% threshold (Principle XI)." >&2
        exit 1
    fi

    echo "✓ Coverage configuration validated" >&2
fi

# Build list of available documents
docs=()

# Always check these optional docs
[[ -f "$RESEARCH" ]] && docs+=("research.md")
[[ -f "$DATA_MODEL" ]] && docs+=("data-model.md")

# Check contracts directory (only if it exists and has files)
if [[ -d "$CONTRACTS_DIR" ]] && [[ -n "$(ls -A "$CONTRACTS_DIR" 2>/dev/null)" ]]; then
    docs+=("contracts/")
fi

[[ -f "$QUICKSTART" ]] && docs+=("quickstart.md")

# Include tasks.md if requested and it exists
if $INCLUDE_TASKS && [[ -f "$TASKS" ]]; then
    docs+=("tasks.md")
fi

# Output results
if $JSON_MODE; then
    # Build JSON array of documents
    if [[ ${#docs[@]} -eq 0 ]]; then
        json_docs="[]"
    else
        json_docs=$(printf '"%s",' "${docs[@]}")
        json_docs="[${json_docs%,}]"
    fi
    
    printf '{"FEATURE_DIR":"%s","AVAILABLE_DOCS":%s}\n' "$FEATURE_DIR" "$json_docs"
else
    # Text output
    echo "FEATURE_DIR:$FEATURE_DIR"
    echo "AVAILABLE_DOCS:"
    
    # Show status of each potential document
    check_file "$RESEARCH" "research.md"
    check_file "$DATA_MODEL" "data-model.md"
    check_dir "$CONTRACTS_DIR" "contracts/"
    check_file "$QUICKSTART" "quickstart.md"
    
    if $INCLUDE_TASKS; then
        check_file "$TASKS" "tasks.md"
    fi
fi
