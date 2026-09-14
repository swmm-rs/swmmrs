"""Check generated API contracts and local links throughout the rendered JS docs."""

from collections import Counter
from html.parser import HTMLParser
import json
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit


class Page(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = []
        self.links = []
        self.text = []
        self.headings = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if "id" in attrs:
            self.ids.append(attrs["id"])
        if tag == "a" and "href" in attrs:
            self.links.append(attrs["href"])
        if tag in {"h2", "h3", "h4", "h5"}:
            self.headings.append(attrs.get("id"))

    def handle_data(self, data):
        self.text.append(data)


root = Path(__file__).resolve().parents[2]
site = root / "site"
cache = {}


def load(path):
    if path not in cache:
        page = Page()
        page.feed(path.read_text())
        cache[path] = page
    return cache[path]


files = sorted((site / "javascript").rglob("*.html"))
assert files, "No rendered JavaScript pages"
link_count = 0
for path in files:
    page = load(path)
    relative = path.relative_to(site)
    duplicates = [key for key, count in Counter(page.ids).items() if count > 1]
    assert not duplicates, f"{relative}: duplicate HTML anchors: {duplicates}"
    base = "/" + str(relative.parent) + "/"
    for href in page.links:
        target = urlsplit(urljoin(base, href))
        if target.scheme or target.netloc:
            continue
        destination = site / unquote(target.path).lstrip("/")
        if destination.is_dir():
            destination /= "index.html"
        assert destination.is_file(), f"{relative}: missing local link: {href}"
        if target.fragment and destination.suffix == ".html":
            assert unquote(target.fragment) in load(destination).ids, f"{relative}: broken fragment: {href}"
        link_count += 1


config = json.loads((root / "docs/typescript-docs/typedoc.json").read_text())
families = [Path(entry).stem for entry in config["entryPoints"]]
for family in families:
    page = load(site / f"javascript/api/{family}/index.html")
    text = "".join(page.text)
    assert "--8<--" not in text, f"{family}: unexpanded API include"
    assert "{#" not in text, f"{family}: unparsed custom heading IDs"
    assert len(page.headings) > 3, f"{family}: missing generated TOC headings"

expected = {
    "node": ["configure(patch: NodePatch)", "Parameters", "Returns", "Throws", "Defaults to", "merge supplied IDs", "not a flow rate", "NodeQualitySnapshot", "readonly (readonly number[])[]"],
    "simulation": ["saveCheckpoint", "CheckpointBundle", "cleanupError"],
    "output": ["readBulkSeries", "OutputReadOptions", "half-open", "invalid_period_range"],
    "lid": ["Initial saturation percentage, 0–100", "equivalent depth in in or mm", "before end()"],
    "subcatchment": ["Surface slope as a fraction, not percent", "SubcatchmentQualitySnapshot"],
    "rdii": ["AmmAssignment", "UnitHydrographResponse", "RdiiAssignment"],
    "definitions": ["AquiferPatch", "SnowmeltSurfacePatch"],
}
for family, snippets in expected.items():
    page = load(site / f"javascript/api/{family}/index.html")
    text = "".join(page.text)
    for snippet in snippets:
        assert snippet in text, f"{family}: missing rendered API content: {snippet}"
assert "configure" in load(site / "javascript/api/node/index.html").headings

# Keep entry links published before the full migration working.
for family, fragment in [
    ("simulation", "hotstarts-checkpoints-and-forks"),
    ("snapshots", "binary-output-reader"),
    ("collections", "specialized-families"),
    ("subcatchment", "lid-units"),
    ("node", "noderesults-1"),
]:
    assert fragment in load(site / f"javascript/api/{family}/index.html").ids
print(f"Checked {len(families)} generated API pages and {link_count} local links across {len(files)} JavaScript pages.")
