"""Check that a prose rewrite keeps links, code, and numeric facts."""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

LINK = re.compile(r"]\(([^)\s]+)")
INLINE_CODE = re.compile(r"(?<!`)`([^`\n]+)`(?!`)")
FENCED_CODE = re.compile(r"^```[^\n]*\n.*?^```[ \t]*$", re.MULTILINE | re.DOTALL)
NUMBER = re.compile(
    r"(?<![A-Za-z0-9_./-])"
    r"\d+(?:,\d{3})*(?:\.\d+)*(?:%|[A-Za-z]+)?"
    r"(?![A-Za-z0-9_/-])"
)


def anchors(text: str) -> dict[str, set[str]]:
    without_links = LINK.sub("](", text)
    return {
        "links": set(LINK.findall(text)),
        "inline code": set(INLINE_CODE.findall(text)),
        "code blocks": set(FENCED_CODE.findall(text)),
        "numbers": set(NUMBER.findall(without_links)),
    }


def from_git(revision: str, path: Path) -> str:
    result = subprocess.run(
        ["git", "show", f"{revision}:{path.as_posix()}"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Report factual anchors dropped by documentation rewrites."
    )
    parser.add_argument("files", nargs="+", type=Path)
    parser.add_argument("--base", default="HEAD", help="Git revision to compare")
    args = parser.parse_args()

    failures = 0
    for path in args.files:
        before = anchors(from_git(args.base, path))
        after = anchors(path.read_text())
        for label, old_values in before.items():
            missing = old_values - after[label]
            if not missing:
                continue
            failures += 1
            print(f"{path}: dropped {label}:")
            for value in sorted(missing):
                print(f"  {value!r}")

    if failures:
        return 1

    print(f"All factual anchors remain in {len(args.files)} rewritten file(s).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
