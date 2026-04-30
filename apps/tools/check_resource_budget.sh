#!/bin/sh

set -eu

usage() {
  echo "usage: $0 <resource-root> <manifest> [install-root]" >&2
  echo "manifest format: <class> <relative-path> <max-bytes>" >&2
}

fail() {
  echo "resource budget error: $*" >&2
  exit 1
}

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
  usage
  exit 2
fi

root=${1%/}
manifest=$2
install_root=${3:-}

[ -d "$root" ] || fail "resource root not found: $root"
[ -f "$manifest" ] || fail "manifest not found: $manifest"

if [ -n "$install_root" ]; then
  mkdir -p "$install_root"
fi

count=0
total=0
line_no=0

while IFS= read -r line || [ -n "$line" ]; do
  line_no=$((line_no + 1))

  case "$line" in
    ""|\#*)
      continue
      ;;
  esac

  set -- $line
  if [ "$#" -ne 3 ]; then
    fail "$manifest:$line_no expected 3 fields: <class> <relative-path> <max-bytes>"
  fi

  class=$1
  rel=$2
  max=$3

  case "$class" in
    runtime|demo|fixture|license)
      ;;
    *)
      fail "$manifest:$line_no unsupported resource class '$class'"
      ;;
  esac

  case "$rel" in
    ""|/*|..|../*|*/..|*/../*)
      fail "$manifest:$line_no invalid relative path '$rel'"
      ;;
  esac

  case "$max" in
    ""|*[!0-9]*)
      fail "$manifest:$line_no invalid max-bytes '$max'"
      ;;
  esac

  src=$root/$rel
  [ -f "$src" ] || fail "$manifest:$line_no missing resource: $rel"

  size=$(LC_ALL=C wc -c < "$src" | tr -d '[:space:]')
  if [ "$size" -gt "$max" ]; then
    fail "$manifest:$line_no $class resource '$rel' is ${size} bytes, max ${max}"
  fi

  count=$((count + 1))
  total=$((total + size))

  if [ -n "$install_root" ]; then
    dst=$install_root/$rel
    mkdir -p "$(dirname "$dst")"
    cp "$src" "$dst"
  fi
done < "$manifest"

if [ "$count" -eq 0 ]; then
  fail "$manifest has no resources"
fi

echo "Resource budget OK: $count files, $total bytes ($manifest)"
if [ -n "$install_root" ]; then
  echo "Installed resource manifest into $install_root"
fi
