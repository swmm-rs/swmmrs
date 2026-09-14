#!/bin/sh
set -eu

/usr/bin/git worktree list --porcelain |
  sed -n 's/^worktree //p' |
  tail -n +2 |
  while IFS= read -r path; do
    /usr/bin/git worktree remove --force "$path"
  done

/usr/bin/git worktree prune