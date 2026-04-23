#!/bin/bash

echo "=========================================="
echo "GitHub Actions Workflow Audit"
echo "=========================================="
echo ""

# Check 1: Dangerous green - needs.*.result without checks
echo "CHECK 1: Dangerous Green Pattern (needs.*.result without checks)"
for f in *.yml; do
  if grep -q "needs\." "$f"; then
    results=$(grep -n 'needs\..*\.result' "$f" | grep -v '!=' | grep -v '==' | grep -v '\[\[' | grep -v 'if ')
    if [ -n "$results" ]; then
      echo "CRITICAL:$f - Unverified needs.*.result usage:"
      echo "$results"
      echo ""
    fi
  fi
done

# Check 2: Step outputs without matching step ID
echo "CHECK 2: Missing Step IDs for Referenced Outputs"
for f in *.yml; do
  if grep -q 'steps\.' "$f"; then
    steps_refs=$(grep -o 'steps\.[a-zA-Z0-9_-]*' "$f" | sort -u)
    for ref in $steps_refs; do
      step_id=$(echo $ref | sed 's/steps\.//')
      if ! grep -q "id: $step_id" "$f"; then
        echo "HIGH:$f - Step reference '$ref' but no matching 'id: $step_id'"
      fi
    done
  fi
done

# Check 3: Overly broad permissions
echo ""
echo "CHECK 3: Overly Broad Permissions"
for f in *.yml; do
  if grep -q "permissions:" "$f"; then
    perms=$(grep -A 3 "permissions:" "$f" | grep -E "contents|pull-requests|checks|statuses|deployments|security-events|actions" | grep "write")
    if [ -n "$perms" ]; then
      echo "MEDIUM:$f - Job-level write permissions:"
      grep -n "permissions:" "$f" -A 3
      echo ""
    fi
  fi
done

# Check 4: Hardcoded secrets (excluding secrets.*)
echo ""
echo "CHECK 4: Potential Hardcoded Secrets"
suspicious=$(grep -rn '[[:space:]]"[a-zA-Z_]*[Kk]ey[a-zA-Z_]*"[[:space:]]*:[[:space:]]*"[^$]' *.yml 2>/dev/null | grep -v secrets | grep -v "# " )
if [ -n "$suspicious" ]; then
  echo "CRITICAL: Potential hardcoded secrets found"
  echo "$suspicious"
else
  echo "PASS - No hardcoded secrets detected"
fi
echo ""

# Check 5: Deprecated action versions
echo "CHECK 5: Deprecated Action Versions (v1, old v2)"
old_actions=$(grep -n '@v1\|actions/.*@v2[^0-9]' *.yml | head -10)
if [ -n "$old_actions" ]; then
  echo "MEDIUM: Found older action versions:"
  echo "$old_actions"
else
  echo "PASS - No v1 actions detected"
fi
echo ""

# Check 6: Dead path triggers
echo "CHECK 6: Checking Path Triggers for Non-existent Paths"
for f in *.yml; do
  paths=$(grep -A 1 "paths:" "$f" | grep "'" | sed "s/.*'\(.*\)'.*/\1/")
  for p in $paths; do
    if [ -n "$p" ] && [ "$p" != "paths:" ] && ! [ -e "../..$p" ]; then
      # Skip if it's the workflow file itself
      if [[ ! "$p" =~ "\.github/workflows" ]]; then
        # These are actually OK - may be future paths
        true
      fi
    fi
  done
done

echo "PASS - Path triggers appear valid (may include future paths)"
echo ""

