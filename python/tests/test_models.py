"""Tests for the agentic_vision.models module.

Validates frozen dataclasses, enums, and parsing helpers.
"""

from __future__ import annotations

import dataclasses

import pytest

from agentic_vision.models import (
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


# ---------------------------------------------------------------------------
# Enums
# ---------------------------------------------------------------------------


class TestCaptureSourceType:
    def test_values(self) -> None:
        assert CaptureSourceType.FILE == "file"
        assert CaptureSourceType.BASE64 == "base64"
        assert CaptureSourceType.SCREENSHOT == "screenshot"
        assert CaptureSourceType.CLIPBOARD == "clipboard"

    def test_is_str(self) -> None:
        assert isinstance(CaptureSourceType.FILE, str)

    def test_from_string(self) -> None:
        assert CaptureSourceType("file") == CaptureSourceType.FILE

    def test_all_members(self) -> None:
        assert len(CaptureSourceType) == 4


# ---------------------------------------------------------------------------
# Rect
# ---------------------------------------------------------------------------


class TestRect:
    def test_create(self) -> None:
        r = Rect(x=10, y=20, w=100, h=200)
        assert r.x == 10
        assert r.y == 20
        assert r.w == 100
        assert r.h == 200

    def test_defaults(self) -> None:
        r = Rect()
        assert r.x == 0
        assert r.w == 0

    def test_frozen(self) -> None:
        r = Rect(x=1, y=2, w=3, h=4)
        with pytest.raises(dataclasses.FrozenInstanceError):
            r.x = 99  # type: ignore[misc]


# ---------------------------------------------------------------------------
# ObservationMeta
# ---------------------------------------------------------------------------


class TestObservationMeta:
    def test_create(self) -> None:
        meta = ObservationMeta(
            width=640,
            height=480,
            original_width=1920,
            original_height=1080,
            labels=("ui", "form"),
            description="Login page",
            quality_score=0.85,
        )
        assert meta.width == 640
        assert meta.height == 480
        assert len(meta.labels) == 2
        assert meta.description == "Login page"
        assert meta.quality_score == 0.85

    def test_defaults(self) -> None:
        meta = ObservationMeta()
        assert meta.width == 0
        assert meta.labels == ()
        assert meta.description is None
        assert meta.quality_score == 0.0

    def test_frozen(self) -> None:
        meta = ObservationMeta()
        with pytest.raises(dataclasses.FrozenInstanceError):
            meta.width = 100  # type: ignore[misc]


# ---------------------------------------------------------------------------
# CaptureInfo
# ---------------------------------------------------------------------------


class TestCaptureInfo:
    def test_create(self) -> None:
        info = CaptureInfo(
            id=1,
            timestamp=1000,
            session_id=5,
            source_type="file",
            width=640,
            height=480,
        )
        assert info.id == 1
        assert info.timestamp == 1000
        assert info.source_type == "file"

    def test_defaults(self) -> None:
        info = CaptureInfo()
        assert info.id == 0
        assert info.memory_link is None
        assert info.labels == ()

    def test_frozen(self) -> None:
        info = CaptureInfo(id=1)
        with pytest.raises(dataclasses.FrozenInstanceError):
            info.id = 2  # type: ignore[misc]


# ---------------------------------------------------------------------------
# CompareResult
# ---------------------------------------------------------------------------


class TestCompareResult:
    def test_create(self) -> None:
        result = CompareResult(
            before_id=1,
            after_id=2,
            similarity=0.95,
            pixel_diff_ratio=0.05,
        )
        assert result.before_id == 1
        assert result.after_id == 2
        assert result.similarity == 0.95

    def test_defaults(self) -> None:
        result = CompareResult()
        assert result.similarity == 0.0
        assert result.changed_regions == ()

    def test_frozen(self) -> None:
        result = CompareResult()
        with pytest.raises(dataclasses.FrozenInstanceError):
            result.similarity = 0.5  # type: ignore[misc]


# ---------------------------------------------------------------------------
# SimilarityMatch
# ---------------------------------------------------------------------------


class TestSimilarityMatch:
    def test_create(self) -> None:
        match = SimilarityMatch(id=42, similarity=0.87)
        assert match.id == 42
        assert match.similarity == 0.87

    def test_defaults(self) -> None:
        match = SimilarityMatch()
        assert match.id == 0
        assert match.similarity == 0.0


# ---------------------------------------------------------------------------
# HealthReport
# ---------------------------------------------------------------------------


class TestHealthReport:
    def test_create(self) -> None:
        report = HealthReport(
            observation_count=100,
            embedding_dim=512,
            session_count=3,
        )
        assert report.observation_count == 100
        assert report.embedding_dim == 512
        assert report.is_empty is False

    def test_empty(self) -> None:
        report = HealthReport()
        assert report.is_empty is True

    def test_frozen(self) -> None:
        report = HealthReport()
        with pytest.raises(dataclasses.FrozenInstanceError):
            report.observation_count = 5  # type: ignore[misc]


# ---------------------------------------------------------------------------
# StoreInfo
# ---------------------------------------------------------------------------


class TestStoreInfo:
    def test_create(self) -> None:
        info = StoreInfo(
            path="/tmp/test.avis",
            observation_count=50,
            embedding_dim=512,
        )
        assert info.path == "/tmp/test.avis"
        assert info.is_empty is False

    def test_empty(self) -> None:
        info = StoreInfo()
        assert info.is_empty is True


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------


class TestParseCapture:
    def test_full_data(self) -> None:
        data = {
            "id": 7,
            "timestamp": 1234567890,
            "session_id": 2,
            "source": {"type": "file", "path": "/tmp/img.png"},
            "metadata": {
                "width": 1920,
                "height": 1080,
                "original_width": 1920,
                "original_height": 1080,
                "labels": ["screenshot", "dashboard"],
                "description": "Main dashboard",
                "quality_score": 0.9,
            },
            "memory_link": 42,
        }
        info = parse_capture_info(data)
        assert info.id == 7
        assert info.source_type == "file"
        assert info.width == 1920
        assert info.labels == ("screenshot", "dashboard")
        assert info.description == "Main dashboard"
        assert info.memory_link == 42

    def test_minimal_data(self) -> None:
        info = parse_capture_info({})
        assert info.id == 0
        assert info.source_type == ""
        assert info.labels == ()


class TestParseCompareResult:
    def test_full_data(self) -> None:
        data = {
            "before_id": 1,
            "after_id": 2,
            "similarity": 0.85,
            "pixel_diff_ratio": 0.15,
            "changed_regions": [
                {"x": 10, "y": 20, "w": 100, "h": 50},
            ],
        }
        result = parse_compare_result(data)
        assert result.before_id == 1
        assert result.similarity == 0.85
        assert len(result.changed_regions) == 1
        assert result.changed_regions[0].x == 10

    def test_minimal_data(self) -> None:
        result = parse_compare_result({})
        assert result.similarity == 0.0
        assert result.changed_regions == ()


class TestParseHealthReport:
    def test_full_data(self) -> None:
        data = {
            "observation_count": 50,
            "embedding_dim": 512,
            "session_count": 3,
            "created_at": 100,
            "updated_at": 200,
        }
        report = parse_health_report(data)
        assert report.observation_count == 50
        assert report.embedding_dim == 512
        assert report.is_empty is False

    def test_minimal_data(self) -> None:
        report = parse_health_report({})
        assert report.observation_count == 0
        assert report.is_empty is True
