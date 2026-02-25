"""Tests for the agentic_vision.errors module.

Validates the error hierarchy and attributes.
"""

from __future__ import annotations

import pytest

from agentic_vision.errors import (
    AvisError,
    CaptureError,
    CaptureNotFoundError,
    EmbeddingError,
    LibraryNotFoundError,
    StorageError,
    ValidationError,
)


class TestAvisError:
    def test_base_error(self) -> None:
        err = AvisError("test error")
        assert str(err) == "test error"
        assert err.code == -1

    def test_custom_code(self) -> None:
        err = AvisError("fail", code=-42)
        assert err.code == -42

    def test_is_exception(self) -> None:
        assert issubclass(AvisError, Exception)


class TestCaptureError:
    def test_with_source(self) -> None:
        err = CaptureError("https://example.com")
        assert err.source == "https://example.com"
        assert "https://example.com" in str(err)
        assert err.code == -2

    def test_empty(self) -> None:
        err = CaptureError()
        assert err.source == ""

    def test_is_avis_error(self) -> None:
        assert issubclass(CaptureError, AvisError)


class TestStorageError:
    def test_with_path(self) -> None:
        err = StorageError("/tmp/store.avis")
        assert err.path == "/tmp/store.avis"
        assert "/tmp/store.avis" in str(err)
        assert err.code == -3

    def test_is_avis_error(self) -> None:
        assert issubclass(StorageError, AvisError)


class TestEmbeddingError:
    def test_default(self) -> None:
        err = EmbeddingError()
        assert "Embedding" in str(err)
        assert err.code == -4

    def test_custom(self) -> None:
        err = EmbeddingError("model not loaded")
        assert "model not loaded" in str(err)

    def test_is_avis_error(self) -> None:
        assert issubclass(EmbeddingError, AvisError)


class TestCaptureNotFoundError:
    def test_stores_id(self) -> None:
        err = CaptureNotFoundError(42)
        assert err.capture_id == 42
        assert "42" in str(err)
        assert err.code == -5

    def test_is_avis_error(self) -> None:
        assert issubclass(CaptureNotFoundError, AvisError)


class TestLibraryNotFoundError:
    def test_default(self) -> None:
        err = LibraryNotFoundError()
        assert "Native library not found" in str(err)
        assert err.searched == []
        assert err.code == -1

    def test_with_locations(self) -> None:
        err = LibraryNotFoundError(["/usr/lib", "/opt/lib"])
        assert "/usr/lib" in str(err)
        assert "/opt/lib" in str(err)
        assert err.searched == ["/usr/lib", "/opt/lib"]

    def test_is_avis_error(self) -> None:
        assert issubclass(LibraryNotFoundError, AvisError)


class TestValidationError:
    def test_default(self) -> None:
        err = ValidationError()
        assert "Validation" in str(err)
        assert err.code == -6

    def test_is_avis_error(self) -> None:
        assert issubclass(ValidationError, AvisError)

    def test_raise(self) -> None:
        with pytest.raises(AvisError):
            raise ValidationError("empty input")


class TestHierarchy:
    """All errors should be subclasses of both AvisError and Exception."""

    @pytest.mark.parametrize(
        "cls",
        [
            CaptureError,
            StorageError,
            EmbeddingError,
            CaptureNotFoundError,
            LibraryNotFoundError,
            ValidationError,
        ],
    )
    def test_is_subclass_of_avis_error(self, cls: type) -> None:
        assert issubclass(cls, AvisError)

    @pytest.mark.parametrize(
        "cls",
        [
            AvisError,
            CaptureError,
            StorageError,
            EmbeddingError,
            CaptureNotFoundError,
            LibraryNotFoundError,
            ValidationError,
        ],
    )
    def test_is_subclass_of_exception(self, cls: type) -> None:
        assert issubclass(cls, Exception)
