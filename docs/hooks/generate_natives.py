"""MkDocs hook: regenerates the API reference before the site is built.

Keeping the page out of the repository and building it here is what makes
"generated from the includes" true rather than aspirational: there is no
committed copy that can be stale, and `mkdocs serve` picks up a change to the
includes on the next reload.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

GENERATOR = Path(__file__).resolve().parents[2] / "scripts/gen_natives_page.py"


def on_config(config):  # noqa: ANN001, ANN201 - MkDocs hook signature
    """Runs before the file tree is read, so the page exists in time for nav."""
    spec = importlib.util.spec_from_file_location("gen_natives_page", GENERATOR)
    if spec is None or spec.loader is None:  # pragma: no cover - broken checkout
        raise RuntimeError(f"cannot load the natives generator at {GENERATOR}")

    module = importlib.util.module_from_spec(spec)
    sys.modules["gen_natives_page"] = module
    spec.loader.exec_module(module)

    page, undocumented = module.build()
    module.OUTPUT.write_text(page, encoding="utf-8")
    if undocumented:
        print(f"WARNING - API reference: {len(undocumented)} entries have no doc block")
    return config
