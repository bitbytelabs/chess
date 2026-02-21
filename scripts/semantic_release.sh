#!/usr/bin/env bash
set -euo pipefail

VERSION_FILE="VERSION"
CARGO_FILE="Cargo.toml"
CHANGELOG_FILE="CHANGELOG.md"

if [[ ! -f "$VERSION_FILE" ]]; then
  if [[ -f "$CARGO_FILE" ]]; then
    awk -F '"' '/^version = / { print $2; exit }' "$CARGO_FILE" > "$VERSION_FILE"
  else
    echo "0.1.0" > "$VERSION_FILE"
  fi
fi

current_version="$(cat "$VERSION_FILE" | tr -d '[:space:]')"
if [[ ! "$current_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "Invalid version in $VERSION_FILE: $current_version" >&2
  exit 1
fi

last_tag="$(git tag --list 'v*' --sort=-v:refname | head -n 1)"
if [[ -n "$last_tag" ]]; then
  range="$last_tag..HEAD"
else
  range="HEAD"
fi

commit_data="$(git log "$range" --pretty=format:'%H%n%s%n%b%n==END==')"

bump="none"
while IFS= read -r line; do
  [[ "$line" == "==END==" ]] && continue

  if echo "$line" | grep -Eq 'BREAKING[[:space:]]CHANGE|^(feat|fix)(\([^)]+\))?!:'; then
    bump="major"
    break
  fi

  if echo "$line" | grep -Eq '^feat(\([^)]+\))?:'; then
    [[ "$bump" != "major" ]] && bump="minor"
  elif echo "$line" | grep -Eq '^fix(\([^)]+\))?:'; then
    [[ "$bump" == "none" ]] && bump="patch"
  fi
done <<< "$commit_data"

if [[ "$bump" == "none" ]]; then
  echo "No semantic version bump required."
  exit 0
fi

IFS='.' read -r major minor patch <<< "$current_version"
case "$bump" in
  major) major=$((major + 1)); minor=0; patch=0 ;;
  minor) minor=$((minor + 1)); patch=0 ;;
  patch) patch=$((patch + 1)) ;;
esac
new_version="$major.$minor.$patch"

echo "$new_version" > "$VERSION_FILE"

if [[ -f "$CARGO_FILE" ]]; then
  sed -i -E "0,/^version = \"[0-9]+\.[0-9]+\.[0-9]+\"/s//version = \"$new_version\"/" "$CARGO_FILE"
fi

if [[ ! -f "$CHANGELOG_FILE" ]]; then
  cat > "$CHANGELOG_FILE" <<'CHANGELOG'
# Changelog

All notable changes to this project will be documented in this file.
CHANGELOG
fi

release_date="$(date -u +%Y-%m-%d)"
entry_file="$(mktemp)"
{
  echo "## v$new_version - $release_date"
  git log "$range" --pretty=format:'- %s (%h)'
  echo
  echo
} > "$entry_file"

if ! rg -q "^## v$new_version\b" "$CHANGELOG_FILE"; then
  tmp_file="$(mktemp)"
  {
    head -n 3 "$CHANGELOG_FILE"
    cat "$entry_file"
    tail -n +4 "$CHANGELOG_FILE"
  } > "$tmp_file"
  mv "$tmp_file" "$CHANGELOG_FILE"
fi

echo "Bumped $current_version -> $new_version ($bump)"
