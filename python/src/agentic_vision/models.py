"""Data models for the agentic_vision package.

All models are frozen dataclasses — immutable and thread-safe.
Enums use ``(str, Enum)`` for natural JSON serialization.
"""

from __future__ import annotations

import dataclasses
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------


class CaptureSourceType(str, Enum):
    """How an image was captured."""

    FILE = "file"
    BASE64 = "base64"
    SCREENSHOT = "screenshot"
    CLIPBOARD = "clipboard"


# ---------------------------------------------------------------------------
# Core data models
# ---------------------------------------------------------------------------


@dataclass(frozen=True)
class Rect:
    """A rectangular region in pixels."""

    x: int = 0
    y: int = 0
    w: int = 0
    h: int = 0


@dataclass(frozen=True)
class ObservationMeta:
    """Metadata about a visual observation."""

    width: int = 0
    height: int = 0
    original_width: int = 0
    original_height: int = 0
    labels: tuple[str, ...] = ()
    description: Optional[str] = None
    quality_score: float = 0.0


@dataclass(frozen=True)
class CaptureInfo:
    """Summary of a captured visual observation."""

    id: int = 0
    timestamp: int = 0
    session_id: int = 0
    source_type: str = ""
    width: int = 0
    height: int = 0
    labels: tuple[str, ...] = ()
    description: Optional[str] = None
    quality_score: float = 0.0
    memory_link: Optional[int] = None


@dataclass(frozen=True)
class CompareResult:
    """Result of comparing two captures."""

    before_id: int = 0
    after_id: int = 0
    similarity: float = 0.0
    pixel_diff_ratio: float = 0.0
    changed_regions: tuple[Rect, ...] = ()


@dataclass(frozen=True)
class SimilarityMatch:
    """A similarity search match."""

    id: int = 0
    similarity: float = 0.0


@dataclass(frozen=True)
class HealthReport:
    """Health report for the visual memory store."""

    observation_count: int = 0
    embedding_dim: int = 0
    session_count: int = 0
    created_at: int = 0
    updated_at: int = 0

    @property
    def is_empty(self) -> bool:
        """True if there are no observations."""
        return self.observation_count == 0


@dataclass(frozen=True)
class StoreInfo:
    """Summary statistics for a visual memory store."""

    path: str = ""
    observation_count: int = 0
    embedding_dim: int = 0
    session_count: int = 0
    created_at: int = 0
    updated_at: int = 0

    @property
    def is_empty(self) -> bool:
        """True if there are no observations."""
        return self.observation_count == 0


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------


def parse_capture_info(data: dict) -> CaptureInfo:  # type: ignore[type-arg]
    """Build a :class:`CaptureInfo` from a raw JSON dict."""
    source = data.get("source", {})
    source_type = source.get("type", "") if isinstance(source, dict) else ""
    meta = data.get("metadata", {})
    labels = tuple(meta.get("labels", [])) if isinstance(meta, dict) else ()
    return CaptureInfo(
        id=data.get("id", 0),
        timestamp=data.get("timestamp", 0),
        session_id=data.get("session_id", 0),
        source_type=source_type,
        width=meta.get("width", 0) if isinstance(meta, dict) else 0,
        height=meta.get("height", 0) if isinstance(meta, dict) else 0,
        labels=labels,
        description=meta.get("description") if isinstance(meta, dict) else None,
        quality_score=meta.get("quality_score", 0.0) if isinstance(meta, dict) else 0.0,
        memory_link=data.get("memory_link"),
    )


def parse_compare_result(data: dict) -> CompareResult:  # type: ignore[type-arg]
    """Build a :class:`CompareResult` from a raw JSON dict."""
    regions = tuple(
        Rect(
            x=r.get("x", 0),
            y=r.get("y", 0),
            w=r.get("w", 0),
            h=r.get("h", 0),
        )
        for r in data.get("changed_regions", [])
    )
    return CompareResult(
        before_id=data.get("before_id", 0),
        after_id=data.get("after_id", 0),
        similarity=data.get("similarity", 0.0),
        pixel_diff_ratio=data.get("pixel_diff_ratio", 0.0),
        changed_regions=regions,
    )


def parse_health_report(data: dict) -> HealthReport:  # type: ignore[type-arg]
    """Build a :class:`HealthReport` from a raw JSON dict."""
    return HealthReport(
        observation_count=data.get("observation_count", 0),
        embedding_dim=data.get("embedding_dim", 0),
        session_count=data.get("session_count", 0),
        created_at=data.get("created_at", 0),
        updated_at=data.get("updated_at", 0),
    )
