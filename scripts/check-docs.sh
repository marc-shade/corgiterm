#!/usr/bin/env bash
set -euo pipefail

required_docs=(
  "ROADMAP.md"
  "docs/roadmap/implementation-backlog.md"
  "docs/specs/beginner-experience-spec.md"
  "docs/specs/current-feature-audit.md"
  "docs/specs/documentation-index.md"
  "docs/specs/feature-completion-spec.md"
  "docs/specs/release-readiness-checklist.md"
  "docs/specs/test-verification-strategy.md"
)

for doc in "${required_docs[@]}"; do
  if [[ ! -s "$doc" ]]; then
    echo "missing required doc: $doc" >&2
    exit 1
  fi
done

for link in \
  "ROADMAP.md" \
  "docs/specs/documentation-index.md"; do
  if ! grep -q "$link" README.md; then
    echo "README.md does not link $link" >&2
    exit 1
  fi
done

if grep -q "GPU-accelerated rendering.*144fps" README.md; then
  echo "README.md still contains the old GPU rendering overclaim" >&2
  exit 1
fi

if grep -q "Every command is previewed before execution" README.md; then
  echo "README.md still overclaims Safe Mode coverage" >&2
  exit 1
fi

echo "documentation checks passed"

