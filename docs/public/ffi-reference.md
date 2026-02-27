---
status: stable
---

# FFI Reference

AgenticVision exposes a C-compatible FFI layer through the `agentic-vision-ffi` crate. This enables integration from any language that supports C function calls (Python ctypes, Node.js ffi-napi, Ruby FFI, Go cgo, etc.).

## Shared Library

Build the shared library:

```bash
cargo build --release -p agentic-vision-ffi
# Output: target/release/libagentic_vision_ffi.{so,dylib,dll}
```

## Functions

### `agentic_vision_ffi_version`

Return the crate version string.

```c
const char* agentic_vision_ffi_version(void);
```

**Returns:** Static version string (e.g., `"0.3.0"`). Caller must NOT free.

## Current Status

The FFI crate currently provides a minimal facade exposing the version string. This is the foundation for future expansion. Planned additions include:

- `avis_open` / `avis_create` -- open or create a `.avis` file with opaque handle
- `avis_close` -- close and free a vision handle
- `avis_capture` -- capture an image from file path
- `avis_query` -- search observations by filters (JSON input/output)
- `avis_similar` -- find similar captures by embedding
- `avis_compare` -- compare two captures
- `avis_diff` -- pixel-level diff between captures
- `avis_stats` -- get store statistics as JSON
- `avis_free_string` -- free strings returned by other FFI functions
- `avis_last_error` -- get the last error message

## Example: Python ctypes

```python
import ctypes

lib = ctypes.CDLL("libagentic_vision_ffi.dylib")

lib.agentic_vision_ffi_version.restype = ctypes.c_char_p
lib.agentic_vision_ffi_version.argtypes = []

version = lib.agentic_vision_ffi_version()
print(f"AgenticVision FFI v{version.decode()}")
```

## Thread Safety

The version function is inherently thread-safe (returns a static string). Future handle-based functions will be thread-safe when called with different handles. Concurrent access to the same handle from multiple threads will require external synchronization.
