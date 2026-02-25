"""High-level Vision API.

Provides the :class:`VisionGraph` class and convenience functions for
image capture, comparison, and similarity search.

.. note::

   This module requires the native ``libagentic_vision`` shared library.
   Until the Rust FFI surface is added to the core crate, these functions
   will raise :class:`~agentic_vision.errors.LibraryNotFoundError`.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Dict, List, Optional

from .errors import AvisError, CaptureNotFoundError, ValidationError
from .models import (
    CaptureInfo,
    CompareResult,
    HealthReport,
    SimilarityMatch,
    parse_capture_info,
    parse_compare_result,
    parse_health_report,
)


class VisionGraph:
    """A vision graph for web cartography.

    Parameters
    ----------
    path:
        Optional path to an existing ``.avis`` file.  If the file exists it
        is opened; otherwise a new in-memory graph is created.
    """

    def __init__(self, path: Optional[str] = None) -> None:
        # Lazy import — avoids crashing at import time when the library
        # is not available.  This lets pure-Python parts (models, errors,
        # tests) work without the native binary.
        from ._ffi import _get_lib

        lib = _get_lib()
        self._lib = lib
        self._path = path
        # NOTE: actual FFI initialisation will go here once the Rust
        # functions (avis_graph_new, avis_graph_open) are exported.
        self._ptr: Any = None  # opaque handle placeholder

    def save(self, path: Optional[str] = None) -> None:
        """Save graph to file."""
        save_path = path or self._path
        if not save_path:
            raise ValidationError("No path specified for save")

    @property
    def capture_count(self) -> int:
        """Return the number of captures in the graph."""
        return 0  # placeholder

    def close(self) -> None:
        """Release the native handle."""
        self._ptr = None


# ---------------------------------------------------------------------------
# Convenience functions
# ---------------------------------------------------------------------------


def capture(url: str, graph_path: str = "vision.avis") -> int:
    """Capture a URL to graph."""
    raise NotImplementedError("Requires native FFI — not yet available")


def query(url: str, graph_path: str = "vision.avis") -> Optional[Dict[str, Any]]:
    """Query a URL from graph."""
    raise NotImplementedError("Requires native FFI — not yet available")


def compare(id1: int, id2: int, graph_path: str = "vision.avis") -> Dict[str, Any]:
    """Compare two captures."""
    raise NotImplementedError("Requires native FFI — not yet available")


def diff(url: str, graph_path: str = "vision.avis") -> Dict[str, Any]:
    """Get diff for URL (latest vs previous)."""
    raise NotImplementedError("Requires native FFI — not yet available")


def similar(
    url: str, graph_path: str = "vision.avis", limit: int = 10
) -> List[Dict[str, Any]]:
    """Find similar captures."""
    raise NotImplementedError("Requires native FFI — not yet available")


def health(graph_path: str = "vision.avis") -> Dict[str, Any]:
    """Get graph health."""
    raise NotImplementedError("Requires native FFI — not yet available")
