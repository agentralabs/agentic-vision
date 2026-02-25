"""Error hierarchy for the agentic_vision package.

All exceptions raised by this library inherit from :class:`AvisError`,
making it easy to catch vision-specific errors in a single ``except`` block.
"""

from __future__ import annotations


class AvisError(Exception):
    """Base exception for all AgenticVision operations.

    Parameters
    ----------
    message:
        Human-readable error description.
    code:
        Numeric error code (mirrors FFI error codes). Defaults to ``-1``.
    """

    def __init__(self, message: str = "", *, code: int = -1) -> None:
        self.code = code
        super().__init__(message)


class CaptureError(AvisError):
    """Raised when an image capture operation fails.

    Parameters
    ----------
    source:
        Description of the capture source that failed.
    """

    def __init__(self, source: str = "", message: str = "") -> None:
        self.source = source
        msg = message or f"Capture failed: {source}" if source else "Capture failed"
        super().__init__(msg, code=-2)


class StorageError(AvisError):
    """Raised when a storage I/O operation fails.

    Parameters
    ----------
    path:
        Filesystem path that caused the error, if applicable.
    """

    def __init__(self, path: str = "", message: str = "") -> None:
        self.path = path
        msg = message or f"Storage error: {path}" if path else "Storage error"
        super().__init__(msg, code=-3)


class EmbeddingError(AvisError):
    """Raised when an embedding computation fails."""

    def __init__(self, message: str = "Embedding computation failed") -> None:
        super().__init__(message, code=-4)


class CaptureNotFoundError(AvisError):
    """Raised when a capture ID does not exist in the store.

    Parameters
    ----------
    capture_id:
        The capture identifier that was not found.
    """

    def __init__(self, capture_id: int = 0) -> None:
        self.capture_id = capture_id
        super().__init__(f"Capture not found: {capture_id}", code=-5)


class LibraryNotFoundError(AvisError):
    """Raised when the native shared library cannot be located.

    Parameters
    ----------
    searched:
        Filesystem paths that were searched.
    """

    def __init__(self, searched: list[str] | None = None) -> None:
        self.searched = searched or []
        paths = ", ".join(self.searched) if self.searched else "(none)"
        super().__init__(
            f"Native library not found. Searched: {paths}",
            code=-1,
        )


class ValidationError(AvisError):
    """Raised when input validation fails."""

    def __init__(self, message: str = "Validation failed") -> None:
        super().__init__(message, code=-6)
