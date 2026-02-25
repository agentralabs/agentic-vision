"""Low-level ctypes bindings for ``libagentic_vision``.

This module loads the shared library and declares the C function signatures
exactly as exported by the Rust ``agentic-vision`` crate.

.. note::

   The Vision core crate does not yet expose a C FFI surface.  This module
   provides the *binding scaffold* so that once the Rust FFI functions are
   added, the Python layer is ready to call them.  Until then, importing
   this module will raise :class:`~agentic_vision.errors.LibraryNotFoundError`
   unless the library exists on disk.
"""

from __future__ import annotations

import ctypes
import ctypes.util
import os
import platform
import sys
from pathlib import Path
from typing import Optional

from .errors import AvisError, LibraryNotFoundError

# ---------------------------------------------------------------------------
# Error codes (mirrored from the Rust crate — defined here for forward compat)
# ---------------------------------------------------------------------------

AVIS_OK: int = 0
AVIS_ERR_IO: int = -1
AVIS_ERR_CAPTURE: int = -2
AVIS_ERR_STORAGE: int = -3
AVIS_ERR_EMBEDDING: int = -4
AVIS_ERR_NOT_FOUND: int = -5
AVIS_ERR_NULL_PTR: int = -6

_ERROR_MESSAGES: dict[int, str] = {
    AVIS_ERR_IO: "A filesystem I/O operation failed",
    AVIS_ERR_CAPTURE: "An image capture operation failed",
    AVIS_ERR_STORAGE: "A storage operation failed",
    AVIS_ERR_EMBEDDING: "An embedding computation failed",
    AVIS_ERR_NOT_FOUND: "The requested capture was not found",
    AVIS_ERR_NULL_PTR: "A required pointer argument was null",
}

# ---------------------------------------------------------------------------
# Library loading
# ---------------------------------------------------------------------------


def _lib_filename() -> str:
    """Return the platform-specific shared library filename."""
    system = platform.system()
    if system == "Darwin":
        return "libagentic_vision.dylib"
    elif system == "Windows":
        return "agentic_vision.dll"
    else:
        return "libagentic_vision.so"


def _find_library() -> str:
    """Locate the native shared library.

    Search order:

    1. ``AGENTIC_VISION_LIB`` environment variable (explicit path).
    2. ``../target/release/`` relative to this package (development build).
    3. ``../target/debug/`` relative to this package (development build).
    4. System library search path via :func:`ctypes.util.find_library`.
    """
    # 1. Explicit env var.
    env_path = os.environ.get("AGENTIC_VISION_LIB")
    if env_path and os.path.isfile(env_path):
        return env_path

    lib_name = _lib_filename()

    # 2-3. Relative to the repository root.
    repo_root = Path(__file__).resolve().parent.parent.parent.parent
    for profile in ("release", "debug"):
        candidate = repo_root / "target" / profile / lib_name
        if candidate.is_file():
            return str(candidate)

    # 4. System search path.
    found = ctypes.util.find_library("agentic_vision")
    if found:
        return found

    raise LibraryNotFoundError(
        [str(repo_root / "target" / p / lib_name) for p in ("release", "debug")]
    )


def _load_library() -> ctypes.CDLL:
    """Load the shared library and declare all C function signatures."""
    lib = ctypes.CDLL(_find_library())
    # Placeholder: add function signatures here once the Rust FFI is ready.
    return lib


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _check(rc: int) -> None:
    """Raise :class:`AvisError` if *rc* is not ``AVIS_OK``."""
    if rc != AVIS_OK:
        msg = _ERROR_MESSAGES.get(rc, f"Unknown FFI error code {rc}")
        raise AvisError(msg, code=rc)
