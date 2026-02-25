"""AgenticVision — Binary web cartography for AI agents.

Public API
----------

Models (pure Python — always available):

* :class:`CaptureInfo`, :class:`CompareResult`, :class:`SimilarityMatch`
* :class:`HealthReport`, :class:`StoreInfo`, :class:`ObservationMeta`
* :class:`Rect`, :class:`CaptureSourceType`
* :func:`parse_capture_info`, :func:`parse_compare_result`, :func:`parse_health_report`

Errors:

* :class:`AvisError` (base), :class:`CaptureError`, :class:`StorageError`
* :class:`EmbeddingError`, :class:`CaptureNotFoundError`
* :class:`LibraryNotFoundError`, :class:`ValidationError`

High-level wrapper (requires native library):

* :class:`VisionGraph`
* :func:`capture`, :func:`query`, :func:`compare`, :func:`diff`,
  :func:`similar`, :func:`health`
"""

from __future__ import annotations

__version__ = "0.1.0"

# -- errors (always available) --------------------------------------------
from .errors import (
    AvisError,
    CaptureError,
    CaptureNotFoundError,
    EmbeddingError,
    LibraryNotFoundError,
    StorageError,
    ValidationError,
)

# -- models (always available) --------------------------------------------
from .models import (
    CaptureInfo,
    CaptureSourceType,
    CompareResult,
    HealthReport,
    ObservationMeta,
    Rect,
    SimilarityMatch,
    StoreInfo,
    parse_capture_info,
    parse_compare_result,
    parse_health_report,
)

# -- high-level wrapper (requires native library) -------------------------
from .vision import (
    VisionGraph,
    capture,
    compare,
    diff,
    health,
    query,
    similar,
)

__all__ = [
    # package
    "__version__",
    # errors
    "AvisError",
    "CaptureError",
    "CaptureNotFoundError",
    "EmbeddingError",
    "LibraryNotFoundError",
    "StorageError",
    "ValidationError",
    # models
    "CaptureInfo",
    "CaptureSourceType",
    "CompareResult",
    "HealthReport",
    "ObservationMeta",
    "Rect",
    "SimilarityMatch",
    "StoreInfo",
    "parse_capture_info",
    "parse_compare_result",
    "parse_health_report",
    # high-level
    "VisionGraph",
    "capture",
    "compare",
    "diff",
    "health",
    "query",
    "similar",
]
