
# .avis File Format Specification

Version: 1 (`avis-v1`)

## Overview

An `.avis` file stores persistent visual memory as a binary file with a fixed-size header followed by a JSON payload. The header provides quick access to summary metadata (observation count, embedding dimension, timestamps) without parsing the payload. The payload contains the full set of visual observations serialized as JSON, including CLIP embeddings, JPEG thumbnails, capture sources, and per-observation metadata.

Default file locations:

- **Project-local:** `.avis/vision.avis` (relative to working directory)
- **User-global:** `~/.agentic-vision/vision.avis`
- **Multi-tenant:** `{data-dir}/{user-id}.avis`

## Structure

```
+---------------------------------------------+
|  HEADER                  64 bytes            |
+---------------------------------------------+
|  PAYLOAD                 variable length     |  JSON-serialized observation store
+---------------------------------------------+
```

All multi-byte integers in the header are **little-endian**.

### Header Layout (64 bytes)

```
Offset  Size    Field               Description
------  ------  ------------------  ------------------------------------
 0      4       magic               Magic bytes: 0x41564953 ("AVIS" in ASCII)
 4      2       version             Format version (currently 1)
 6      2       flags               Reserved flags (currently 0)
 8      8       observation_count   Number of observations in the store
16      4       embedding_dim       Embedding vector dimension (default 512)
20      4       session_count       Total number of sessions recorded
24      8       created_at          Store creation timestamp (Unix seconds)
32      8       updated_at          Last update timestamp (Unix seconds)
40      8       payload_length      Byte length of the JSON payload
48      16      (reserved)          Zero-padded, reserved for future use
```

### Payload

The payload immediately follows the header and is exactly `payload_length` bytes of UTF-8 JSON. Its structure is:

```json
{
    "observations": [ ... ],
    "embedding_dim": 512,
    "next_id": 42,
    "session_count": 3,
    "created_at": 1708345678,
    "updated_at": 1708345900
}
```

## Fields

### Header

| Field | Type | Offset | Size | Description |
|:---|:---|:---|:---|:---|
| `magic` | `u32` | 0 | 4 | Must be `0x41564953` (`"AVIS"` in big-endian ASCII, stored little-endian as bytes `53 49 56 41`). |
| `version` | `u16` | 4 | 2 | Format version number. Currently `1`. |
| `flags` | `u16` | 6 | 2 | Reserved. Must be `0`. |
| `observation_count` | `u64` | 8 | 8 | Number of `VisualObservation` records in the payload. |
| `embedding_dim` | `u32` | 16 | 4 | Dimensionality of embedding vectors. Default `512` (CLIP ViT-B/32). |
| `session_count` | `u32` | 20 | 4 | Total sessions recorded in this store. |
| `created_at` | `u64` | 24 | 8 | Timestamp of store creation in seconds since Unix epoch. |
| `updated_at` | `u64` | 32 | 8 | Timestamp of the most recent modification in seconds since Unix epoch. |
| `payload_length` | `u64` | 40 | 8 | Byte length of the JSON payload that follows the header. |
| _(reserved)_ | `[u8; 16]` | 48 | 16 | Zero-filled. Reserved for future header fields. |

### Payload (Top Level)

| Field | Type | Description |
|:---|:---|:---|
| `observations` | `array` | Array of `VisualObservation` objects. |
| `embedding_dim` | `u32` | Embedding dimension (mirrors header). |
| `next_id` | `u64` | Next observation ID to assign. |
| `session_count` | `u32` | Total session count (mirrors header). |
| `created_at` | `u64` | Store creation timestamp in seconds (mirrors header). |
| `updated_at` | `u64` | Last update timestamp in seconds (mirrors header). |

### VisualObservation

| Field | Type | Description |
|:---|:---|:---|
| `id` | `u64` | Unique observation identifier, monotonically increasing from 1. |
| `timestamp` | `u64` | Capture timestamp in seconds since Unix epoch. |
| `session_id` | `u32` | Session during which this observation was captured. |
| `source` | `CaptureSource` | How the image was captured (see below). |
| `embedding` | `array<f32>` | CLIP embedding vector. Length equals `embedding_dim` (default 512). L2-normalized. |
| `thumbnail` | `array<u8>` | JPEG-encoded thumbnail bytes (max 512x512, quality 85). Stored as a JSON array of unsigned byte values. |
| `metadata` | `ObservationMeta` | Metadata about the captured image. |
| `memory_link` | `u64?` | Optional ID of a linked AgenticMemory node. `null` if unlinked. |

### CaptureSource

Tagged union (JSON: `{ "type": "<variant>", ... }`):

| Variant | Fields | Description |
|:---|:---|:---|
| `file` | `path: string` | Captured from a local file path. |
| `base64` | `mime: string` | Decoded from base64 data with the given MIME type. |
| `screenshot` | `region: Rect?` | Screen capture. Optional region for partial capture. |
| `clipboard` | _(none)_ | Captured from the system clipboard. |

### ObservationMeta

| Field | Type | Description |
|:---|:---|:---|
| `width` | `u32` | Thumbnail width in pixels. |
| `height` | `u32` | Thumbnail height in pixels. |
| `original_width` | `u32` | Original image width before thumbnailing. |
| `original_height` | `u32` | Original image height before thumbnailing. |
| `labels` | `array<string>` | User-supplied labels for the observation. |
| `description` | `string?` | Optional human-readable description. |
| `quality_score` | `f32` | Signal quality score in `[0.0, 1.0]`. Computed from resolution, label count, description presence, and model availability. |

### Rect

| Field | Type | Description |
|:---|:---|:---|
| `x` | `u32` | X coordinate (pixels from left). |
| `y` | `u32` | Y coordinate (pixels from top). |
| `w` | `u32` | Width in pixels. |
| `h` | `u32` | Height in pixels. |

## Embedding Vector

Each observation carries a 512-dimensional `f32` embedding vector generated by CLIP ViT-B/32 via ONNX Runtime.

### Preprocessing Pipeline

```
input image (any size, any format)
    |
    v
resize to 224x224 (Lanczos3)
    |
    v
convert to RGB
    |
    v
normalize per-channel:
    mean = [0.48145466, 0.45782750, 0.40821073]
    std  = [0.26862954, 0.26130258, 0.27577711]
    |
    v
NCHW tensor [1, 3, 224, 224]
    |
    v
ONNX inference (clip-vit-base-patch32-visual.onnx)
    |
    v
512-dimensional f32 vector
    |
    v
L2 normalize
    |
    v
stored in observation.embedding
```

When no ONNX model is available, the engine runs in **fallback mode** and produces a zero vector (`[0.0; 512]`). Fallback embeddings are valid but yield zero cosine similarity against all other vectors.

The model file is expected at `~/.agentic-vision/models/clip-vit-base-patch32-visual.onnx`.

## Thumbnail Generation

Thumbnails are generated from the original image and stored inline as JPEG bytes:

1. If either dimension exceeds 512 pixels, resize preserving aspect ratio (Lanczos3) so the largest dimension is 512.
2. Convert to RGB.
3. Encode as JPEG with quality 85.

The resulting bytes are stored in the `thumbnail` field as a JSON array of unsigned integers (0--255). Thumbnails serve as the basis for pixel-level visual diffs between observations.

## Quality Score

The quality score is a weighted blend in `[0.0, 1.0]`:

```
quality = 0.35 * resolution_score
        + 0.20 * label_score
        + 0.20 * description_score
        + 0.25 * model_score
```

| Component | Calculation | Range |
|:---|:---|:---|
| `resolution_score` | `clamp(pixels / (1280 * 720), 0, 1)` | 0.0 -- 1.0 |
| `label_score` | `clamp(label_count / 6, 0, 1)` | 0.0 -- 1.0 |
| `description_score` | `1.0` if description present, else `0.0` | 0.0 or 1.0 |
| `model_score` | `1.0` if CLIP model loaded, else `0.35` | 0.35 or 1.0 |

## Metadata Sanitization

Before storage, labels and descriptions are sanitized to prevent accidental leakage of sensitive data:

| Pattern | Replacement |
|:---|:---|
| Email addresses (contains `@` and `.`) | `[redacted-email]` |
| API keys / secrets (starts with `sk-` and length >= 12, or hex strings >= 32 chars, or contains `token=` and length >= 16) | `[redacted-secret]` |
| Local file paths (starts with `/Users/`, `/home/`, `C:\`, or `D:\`) | `[redacted-path]` |

## Session Lifecycle

Sessions are monotonically numbered starting from 1. Each `.avis` file tracks the total `session_count` in both the header and payload.

### Auto-Save

The `VisionSessionManager` tracks a dirty flag and an auto-save interval (default 30 seconds). When a capture or mutation occurs and the interval has elapsed since the last save, the store is flushed to disk automatically. On session end or manager drop, a final save is always attempted.

### Storage Budget

A configurable storage budget prevents unbounded growth over a 20-year horizon:

| Environment Variable | Default | Description |
|:---|:---|:---|
| `CORTEX_STORAGE_BUDGET_MODE` | `auto-rollup` | `auto-rollup`, `warn`, or `off`. |
| `CORTEX_STORAGE_BUDGET_BYTES` | `2147483648` (2 GiB) | Maximum file size in bytes. |
| `CORTEX_STORAGE_BUDGET_HORIZON_YEARS` | `20` | Projection horizon for growth estimation. |
| `CORTEX_STORAGE_BUDGET_TARGET_FRACTION` | `0.85` | Target size after pruning (fraction of max). |

When `auto-rollup` mode detects the file has exceeded (or is projected to exceed) the budget, it prunes observations from completed sessions, preferring unlinked and low-quality-score observations with the oldest timestamps.

### Temporal Chain

Within a session, captures and observation notes are linked in a temporal chain. Each new capture or note records a `(prev_id, next_id)` edge to the previous item, forming a chronological sequence. Observation notes use an ID offset of `10,000,000` to avoid collisions with capture IDs.

## Security Properties

### No Encryption

The `.avis` format stores all data in plaintext. There is no encryption layer. This is intentional for version 1 -- the file is designed for local, single-user visual memory. Encryption is planned for future versions.

### Metadata Sanitization

All user-supplied text (labels, descriptions) passes through the sanitization pipeline described above before being written to disk. This prevents accidental persistence of email addresses, API keys, and local filesystem paths.

### Drop-on-Close Safety

The `VisionSessionManager` implements Rust's `Drop` trait. If the manager holds unsaved changes when it goes out of scope, it attempts a final save. This ensures data is not lost on unexpected shutdown, though it cannot guarantee success if the process is forcefully killed.

### Payload Integrity

The header's `observation_count`, `embedding_dim`, `session_count`, `created_at`, and `updated_at` fields mirror values inside the JSON payload. On read, the payload values are authoritative; the header values exist for fast inspection without full deserialization. No checksum or authentication tag is currently included -- integrity verification is planned for version 2.

### File Writes

Writes use `std::fs::File::create`, which truncates and rewrites the entire file. This is not atomic in the filesystem sense (no rename-over-temp-file pattern). A crash during write could leave a corrupt file. Atomic writes are planned for version 2.

## Version History

| Version | Format | Description |
|:---|:---|:---|
| 1 | `avis-v1` | Initial release. 64-byte LE header + JSON payload. CLIP ViT-B/32 embeddings (512-dim). JPEG thumbnails. No encryption. No checksum. |
