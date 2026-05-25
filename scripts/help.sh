#!/usr/bin/env bash

FILE=$1
KEY=$2

awk -v target="$KEY" '
  /^#\s/ {
    line = $0
    sub(/^#\s?/, "", line)
    comment = comment line "\n"
    next
  }

  {
    if (target == "") {
      sub(/\n$/, "", comment)
      print comment
      found = 1
      exit 0
    }
  }

  /^[A-Za-z0-9_-]+:\s?/ {
    key = $0
    sub(/:.*$/, "", key)
    if (key == target) {
      sub(/\n$/, "", comment)
      if (comment == "") {
        print "No help available for target `" target "`." > "/dev/stderr"
        found = 1
        exit 1
      }
      print "Help for target `" target "`:"
      print ""
      print comment
      print ""
      found = 1
      exit 0
    }
    comment = ""
    next
  }

  {
    comment = ""
    next
  }

  END {
    if (found == 0) {
      print "Target `" target "` not found." > "/dev/stderr"
    }
    exit 1
  }
' "$FILE"
