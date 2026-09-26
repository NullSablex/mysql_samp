#!/usr/bin/env python3
"""Builds the natives reference page from the includes themselves.

The includes are the only place where a native's signature and its
documentation sit side by side, and `build.rs` already keeps them in step with
the Rust. Writing the same thing again by hand in Markdown would be a third
copy to forget, so the page is generated from them instead - what `cargo doc`
does for Rust, for the Pawn surface.

Two comment formats are understood, because a Pawn project is likely to see
both:

* **JavaDoc** - `@param name text`, `@return text`. This is what the includes
  here use, and what PawnPro renders on hover.
* **pawndoc** - the Pawn compiler's own XML tags: `<summary>`, `<param
  name="x">`, `<returns>`, `<remarks>`. The open.mp standard library documents
  itself this way.

They can be mixed inside one block; whichever tags are present are used.

Run from the repository root:

    python3 scripts/gen_natives_page.py            # writes docs/natives.md
    python3 scripts/gen_natives_page.py --check    # reports gaps, writes nothing
"""

from __future__ import annotations

import html
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BASE_INCLUDE = ROOT / "include/mysql_samp.inc"
OMP_INCLUDE = ROOT / "include/mysql_samp_omp.inc"
OUTPUT = ROOT / "docs/natives.md"


@dataclass
class Doc:
    """A parsed documentation block."""

    summary: str = ""
    remarks: list[str] = field(default_factory=list)
    params: list[tuple[str, str]] = field(default_factory=list)
    returns: str = ""
    examples: list[tuple[str, list[str]]] = field(default_factory=list)

    def is_empty(self) -> bool:
        return not (
            self.summary or self.remarks or self.params or self.returns or self.examples
        )


@dataclass
class Entry:
    """One `native` or `forward`, with whatever documented it."""

    kind: str
    name: str
    signature: str
    doc: Doc
    section: str
    alias: str = ""


def strip_comment_markers(block: str) -> list[str]:
    """Turns a raw comment block into its text lines.

    Handles `/** ... */` and runs of `///`, which both formats use.
    """
    lines: list[str] = []
    for raw in block.splitlines():
        line = raw.strip()
        if line.startswith("/**"):
            line = line[3:]
        elif line.startswith("///"):
            line = line[3:]
        elif line.startswith("*/"):
            continue
        elif line.startswith("*"):
            line = line[1:]
        line = line.removesuffix("*/").rstrip()
        lines.append(line.strip())
    return lines


def parse_pawndoc(text: str, doc: Doc) -> bool:
    """Reads the XML tags of pawndoc. Returns whether any were found."""
    found = False

    summary = re.search(r"<summary>(.*?)</summary>", text, re.S | re.I)
    if summary:
        doc.summary = " ".join(summary.group(1).split())
        found = True

    for match in re.finditer(
        r'<param\s+name\s*=\s*"([^"]+)"\s*>(.*?)</param>', text, re.S | re.I
    ):
        doc.params.append((match.group(1), " ".join(match.group(2).split())))
        found = True

    returns = re.search(r"<returns>(.*?)</returns>", text, re.S | re.I)
    if returns:
        doc.returns = " ".join(returns.group(1).split())
        found = True

    for match in re.finditer(r"<remarks>(.*?)</remarks>", text, re.S | re.I):
        doc.remarks.append(" ".join(match.group(1).split()))
        found = True

    return found


def parse_javadoc(lines: list[str], doc: Doc) -> None:
    """Reads `@param` / `@return`, and the free text above them as the summary."""
    summary: list[str] = []
    current: list[str] | None = None
    target: tuple[str, list[str]] | None = None

    def flush() -> None:
        nonlocal current, target
        if target is not None and current is not None:
            kind, name = target[0], target[1]
            text = " ".join(" ".join(current).split())
            if kind == "param":
                doc.params.append((name, text))  # type: ignore[arg-type]
            else:
                doc.returns = text
        current, target = None, None

    example: tuple[list[str], list[str]] | None = None

    def flush_example() -> None:
        nonlocal example
        if example is not None:
            caption = " ".join(" ".join(example[0]).split())
            doc.examples.append((caption, example[1]))
            example = None

    for line in lines:
        if example is not None:
            stripped = line.strip()
            if not stripped:
                continue
            # Inside an example, a line is code when it reads like a
            # statement; anything else continues the caption.
            if re.match(r"(new\s|[A-Za-z_][\w]*\s*\()", stripped):
                example[1].append(stripped)
                continue
            if not example[1]:
                example[0].append(stripped)
                continue
            flush_example()

        start = re.match(r"@example\s*(.*)", line)
        if start:
            flush()
            example = ([start.group(1)], [])
            continue

        param = re.match(r"@param\s+(\S+)\s*(.*)", line)
        ret = re.match(r"@returns?\s*(.*)", line)
        if param:
            flush()
            target, current = ("param", param.group(1)), [param.group(2)]  # type: ignore[assignment]
        elif ret:
            flush()
            target, current = ("return", ""), [ret.group(1)]  # type: ignore[assignment]
        elif current is not None:
            if line:
                current.append(line)
            else:
                flush()
        elif line:
            summary.append(line)
        elif summary:
            summary.append("")
    flush()
    flush_example()

    # The first paragraph is the summary; the rest is context worth keeping.
    chunks: list[list[str]] = [[]]
    for line in summary:
        if line:
            chunks[-1].append(line)
        elif chunks[-1]:
            chunks.append([])
    chunks = [c for c in chunks if c]
    if chunks and not doc.summary:
        doc.summary = " ".join(" ".join(chunks[0]).split())
    for extra in chunks[1:]:
        doc.remarks.append(" ".join(" ".join(extra).split()))


def parse_doc(block: str) -> Doc:
    doc = Doc()
    lines = strip_comment_markers(block)
    text = "\n".join(lines)

    # pawndoc first: when XML tags are present they are the structure, and any
    # leftover prose is handled by the JavaDoc pass below.
    had_xml = parse_pawndoc(text, doc)
    without_xml = re.sub(r"<[^>]+>.*?</[^>]+>", "", text, flags=re.S) if had_xml else text
    parse_javadoc([line.strip() for line in without_xml.splitlines()], doc)
    return doc


def parse_include(path: Path) -> list[Entry]:
    """Walks an include, pairing each declaration with the block above it."""
    source = path.read_text(encoding="utf-8")
    entries: list[Entry] = []

    section = "General"
    pending: list[str] = []
    in_block = False

    for raw in source.splitlines():
        line = raw.strip()

        if in_block:
            pending.append(raw)
            if "*/" in line:
                in_block = False
            continue

        if line.startswith("/**"):
            pending = [raw]
            in_block = "*/" not in line
            continue

        if line.startswith("///"):
            pending.append(raw)
            continue

        # A documented `#define` is part of the surface too: MYSQL_SYNC is how
        # a call is made blocking, and a page that lists only natives leaves
        # the reader with no way to find it.
        define = re.match(r"#define\s+([A-Za-z_@][\w@]*)\s+(.+)$", line)
        if define and pending:
            entries.append(
                Entry(
                    kind="define",
                    name=define.group(1),
                    signature=f"#define {define.group(1)} {define.group(2)}",
                    doc=parse_doc("\n".join(pending)),
                    section="Constants",
                )
            )
            pending = []
            continue

        decl = re.match(r"(native|forward)\s+([A-Za-z_@][\w@]*:)?([A-Za-z_@][\w@]*)\s*\(", line)
        if decl:
            kind = decl.group(1)
            entries.append(
                Entry(
                    kind=kind,
                    name=decl.group(3),
                    signature=line.rstrip(";"),
                    doc=parse_doc("\n".join(pending)) if pending else Doc(),
                    # A forward is not part of whatever native section it
                    # happens to sit next to - in the include it lands right
                    # after the enum blocks.
                    section="Forwards" if kind == "forward" else section,
                )
            )
            pending = []
            continue

        # A `// Section` line groups what follows, mirroring the include.
        header = re.match(r"//\s+([A-Z][^/]*)$", line)
        if header and not line.startswith("///"):
            # The include qualifies some headers - "Query (non-blocking)" -
            # which is noise once every entry states its own behaviour. The
            # section is just the subject.
            section = re.sub(r"\s*\(.*\)\s*$", "", header.group(1).strip())
            pending = []
            continue

        if not line:
            continue
        # Anything else (enum, define, brace) ends a dangling block.
        pending = []

    return entries


def parse_aliases(path: Path) -> dict[str, str]:
    """Maps a base native to the name the open.mp include gives it."""
    aliases: dict[str, str] = {}
    pattern = re.compile(
        r"^native\s+(?:[A-Za-z_@][\w@]*:)?([A-Za-z_@][\w@]*)\s*\(.*?=\s*([A-Za-z_@][\w@]*)\s*;"
    )
    for line in path.read_text(encoding="utf-8").splitlines():
        match = pattern.match(line.strip())
        if match:
            aliases[match.group(2)] = match.group(1)
    return aliases


# `MYSQL_SYNC`, `MYSQL_OPT_PORT`, `MYSQL_OK` - the names a reader is meant to
# type. In the include they are plain prose, which on a rendered page makes
# them indistinguishable from the sentence around them.
CONSTANT = re.compile(r"(?<![`\w])([A-Z][A-Z0-9]*(?:_[A-Z0-9]+)+)(?![`\w])")
EMPTY_STRING = re.compile(r'(?<![`\w])""(?![`\w])')


def emphasise(text: str) -> str:
    """Marks up the parts of a description that are code, not prose."""
    text = CONSTANT.sub(r"`\1`", text)
    return EMPTY_STRING.sub('`""`', text)


def anchor(heading: str) -> str:
    """The id Python-Markdown's toc extension gives a heading.

    Lowercase, spaces to hyphens, and anything that is neither a word
    character nor a hyphen dropped - underscores survive, which matters here
    since every native has one.
    """
    slug = heading.strip().lower()
    slug = re.sub(r"[^\w\s-]", "", slug)
    return re.sub(r"[\s]+", "-", slug)


def render(entries: list[Entry]) -> str:
    out: list[str] = []
    out.append("# Natives\n")
    out.append(
        "Generated from [`include/mysql_samp.inc`]"
        "(https://github.com/NullSablex/mysql_samp/blob/master/include/mysql_samp.inc.in) "
        "on every docs build, so it cannot drift from what the plugin registers. "
        "Every entry shows the snake_case name and, where one exists, the "
        "`Prefix_PascalCase` alias that [`<mysql_samp_omp>`](installation.md#place-the-include) "
        "declares for it — the two are the same native.\n"
    )
    natives = len([e for e in entries if e.kind == "native"])
    forwards = len([e for e in entries if e.kind == "forward"])
    out.append(f"**{natives} natives**, **{forwards} forward** and the constants below.\n")

    by_section: dict[str, list[Entry]] = {}
    for entry in entries:
        by_section.setdefault(entry.section, []).append(entry)

    # Constants and forwards read as preamble to the natives, not as an
    # afterthought at the end of a long page.
    for first in ("Forwards", "Constants"):
        if first in by_section:
            by_section = {first: by_section.pop(first), **by_section}

    out.append("## Index\n")
    for section, items in by_section.items():
        names = ", ".join(f"[`{e.name}`](#{anchor(e.name)})" for e in items)
        out.append(f"- **{section}** — {names}")
    out.append("")

    for section, items in by_section.items():
        out.append(f"## {section}\n")
        for entry in items:
            out.append(f"### {entry.name}\n")
            out.append("```pawn")
            # A declaration ends in a semicolon; a #define does not.
            out.append(entry.signature if entry.kind == "define" else f"{entry.signature};")
            if entry.alias:
                out.append(f"// open.mp style: {entry.alias}")
            out.append("```\n")

            if entry.doc.summary:
                out.append(f"{emphasise(entry.doc.summary)}\n")
            for remark in entry.doc.remarks:
                out.append(f"{emphasise(remark)}\n")
            if entry.doc.params:
                out.append("| Parameter | Description |")
                out.append("|---|---|")
                for name, text in entry.doc.params:
                    out.append(f"| `{html.escape(name)}` | {emphasise(text)} |")
                out.append("")
            if entry.doc.returns:
                out.append(f"**Returns:** {emphasise(entry.doc.returns)}\n")

            for caption, code in entry.doc.examples:
                if caption:
                    out.append(f"{emphasise(caption)}\n")
                out.append("```pawn")
                out.extend(code)
                out.append("```\n")
            if entry.doc.is_empty():
                out.append("*No documentation block in the include.*\n")

    return "\n".join(out) + "\n"


def build() -> tuple[str, list[str]]:
    entries = parse_include(BASE_INCLUDE)
    aliases = parse_aliases(OMP_INCLUDE)
    for entry in entries:
        entry.alias = aliases.get(entry.name, "")

    undocumented = [e.name for e in entries if e.doc.is_empty()]
    return render(entries), undocumented


SELFTEST_CASES: list[tuple[str, str, Doc]] = [
    (
        "javadoc",
        """/**
         * Opens a connection pool.
         *
         * Options are optional; port 3306 is the default.
         *
         * @param host      hostname, IPv4, or IPv6 in brackets
         * @param options   an options handle, or 0 for defaults
         * @return connection id (>= 1), or 0 on failure
         */""",
        Doc(
            summary="Opens a connection pool.",
            remarks=["Options are optional; port 3306 is the default."],
            params=[
                ("host", "hostname, IPv4, or IPv6 in brackets"),
                ("options", "an options handle, or 0 for defaults"),
            ],
            returns="connection id (>= 1), or 0 on failure",
        ),
    ),
    (
        "pawndoc block comment",
        """/**
         * <summary>Opens a connection pool.</summary>
         * <param name="host">hostname, IPv4, or IPv6 in brackets</param>
         * <param name="options">an options handle, or 0 for defaults</param>
         * <returns>connection id (&gt;= 1), or 0 on failure</returns>
         * <remarks>Options are optional; port 3306 is the default.</remarks>
         */""",
        Doc(
            summary="Opens a connection pool.",
            remarks=["Options are optional; port 3306 is the default."],
            params=[
                ("host", "hostname, IPv4, or IPv6 in brackets"),
                ("options", "an options handle, or 0 for defaults"),
            ],
            returns="connection id (&gt;= 1), or 0 on failure",
        ),
    ),
    (
        "pawndoc line comments",
        """/// <summary>Closes a connection.</summary>
        /// <param name="connId">connection id</param>
        /// <returns>true on success</returns>""",
        Doc(
            summary="Closes a connection.",
            params=[("connId", "connection id")],
            returns="true on success",
        ),
    ),
    (
        "pawndoc spanning several lines",
        """/**
         * <summary>
         *   Runs a statement and keeps the result.
         * </summary>
         * <param name="query">
         *   the SQL to run
         * </param>
         */""",
        Doc(
            summary="Runs a statement and keeps the result.",
            params=[("query", "the SQL to run")],
        ),
    ),
    (
        "mixed: pawndoc summary, javadoc params",
        """/**
         * <summary>Hashes a password.</summary>
         *
         * @param password  the plaintext
         * @return true if queued
         */""",
        Doc(
            summary="Hashes a password.",
            params=[("password", "the plaintext")],
            returns="true if queued",
        ),
    ),
    ("no documentation", "", Doc()),
]


def selftest() -> int:
    """Checks the parser against both comment formats.

    The includes here are written in JavaDoc, so nothing in the repository
    would notice pawndoc support breaking. These fixtures do.
    """
    failures = 0
    for name, block, expected in SELFTEST_CASES:
        got = parse_doc(block)
        for field_name in ("summary", "returns", "params", "remarks"):
            want = getattr(expected, field_name)
            have = getattr(got, field_name)
            if want != have:
                print(f"FAIL [{name}] {field_name}:\n  expected {want!r}\n  got      {have!r}")
                failures += 1
    if failures:
        print(f"{failures} check(s) failed.")
        return 1
    print(f"OK: {len(SELFTEST_CASES)} documentation fixtures parsed as expected.")
    return 0


def main() -> int:
    if "--selftest" in sys.argv:
        return selftest()

    page, undocumented = build()

    if "--check" in sys.argv:
        if undocumented:
            print("Natives with no documentation block:")
            for name in undocumented:
                print(f"  {name}")
            return 1
        print("OK: every native and forward carries a documentation block.")
        return 0

    OUTPUT.write_text(page, encoding="utf-8")
    print(f"Wrote {OUTPUT.relative_to(ROOT)}")
    if undocumented:
        print(f"warning: {len(undocumented)} entries have no documentation block")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
