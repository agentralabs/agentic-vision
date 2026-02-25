"""Tests for package-level imports and metadata.

These tests verify that the public API surface is correctly exported
and that the package metadata (version, __all__) is valid.
"""

from __future__ import annotations


class TestImports:
    def test_import_package(self) -> None:
        import agentic_vision

        assert hasattr(agentic_vision, "__version__")

    def test_version_is_semver(self) -> None:
        from agentic_vision import __version__

        parts = __version__.split(".")
        assert len(parts) == 3
        for part in parts:
            assert part.isdigit()

    def test_avis_error_importable(self) -> None:
        from agentic_vision import AvisError

        assert issubclass(AvisError, Exception)

    def test_capture_info_importable(self) -> None:
        from agentic_vision import CaptureInfo

        assert CaptureInfo is not None

    def test_capture_source_type_importable(self) -> None:
        from agentic_vision import CaptureSourceType

        assert CaptureSourceType.FILE == "file"

    def test_models_importable(self) -> None:
        from agentic_vision import (
            CaptureInfo,
            CompareResult,
            HealthReport,
            ObservationMeta,
            Rect,
            SimilarityMatch,
            StoreInfo,
        )

        assert CaptureInfo is not None
        assert CompareResult is not None
        assert HealthReport is not None
        assert ObservationMeta is not None
        assert Rect is not None
        assert SimilarityMatch is not None
        assert StoreInfo is not None

    def test_errors_importable(self) -> None:
        from agentic_vision import (
            AvisError,
            CaptureError,
            CaptureNotFoundError,
            EmbeddingError,
            LibraryNotFoundError,
            StorageError,
            ValidationError,
        )

        assert AvisError is not None
        assert CaptureError is not None
        assert CaptureNotFoundError is not None
        assert EmbeddingError is not None
        assert LibraryNotFoundError is not None
        assert StorageError is not None
        assert ValidationError is not None

    def test_parse_helpers_importable(self) -> None:
        from agentic_vision import (
            parse_capture_info,
            parse_compare_result,
            parse_health_report,
        )

        assert callable(parse_capture_info)
        assert callable(parse_compare_result)
        assert callable(parse_health_report)

    def test_vision_graph_importable(self) -> None:
        from agentic_vision import VisionGraph

        assert VisionGraph is not None

    def test_convenience_functions_importable(self) -> None:
        from agentic_vision import capture, compare, diff, health, query, similar

        assert callable(capture)
        assert callable(compare)
        assert callable(diff)
        assert callable(health)
        assert callable(query)
        assert callable(similar)


class TestAll:
    def test_all_items_are_importable(self) -> None:
        import agentic_vision

        for name in agentic_vision.__all__:
            assert hasattr(agentic_vision, name), f"{name} in __all__ but not importable"

    def test_all_is_nonempty(self) -> None:
        import agentic_vision

        assert len(agentic_vision.__all__) > 15
