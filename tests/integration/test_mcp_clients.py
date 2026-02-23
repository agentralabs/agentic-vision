"""
Phase 5: Python → Rust Integration Tests
Tests that Python can communicate with agentic-vision-mcp and agentic-memory-mcp
via JSON-RPC over stdin/stdout (MCP stdio transport).
"""
import tempfile
import subprocess
import json
import os
import tomllib
from pathlib import Path


def fresh_temp_path(suffix):
    """Create a temp path that does NOT exist yet (so servers create fresh files)."""
    with tempfile.NamedTemporaryFile(suffix=suffix, delete=True) as f:
        path = f.name
    # File is deleted on close with delete=True, path is now free
    return path


def expected_vision_mcp_version():
    """Resolve expected agentic-vision-mcp version from Cargo metadata."""
    repo_root = Path(__file__).resolve().parents[2]
    mcp_toml = repo_root / "crates" / "agentic-vision-mcp" / "Cargo.toml"
    with mcp_toml.open("rb") as f:
        mcp_cfg = tomllib.load(f)

    package = mcp_cfg.get("package", {})
    pkg_version = package.get("version")
    if isinstance(pkg_version, str) and pkg_version:
        return pkg_version

    # Workspace-inherited version.
    workspace_toml = repo_root / "Cargo.toml"
    with workspace_toml.open("rb") as f:
        ws_cfg = tomllib.load(f)
    return ws_cfg["workspace"]["package"]["version"]


def mcp_roundtrip(command, args, request):
    """Send a single JSON-RPC request via stdin pipe and capture the response."""
    input_data = json.dumps(request) + '\n'
    result = subprocess.run(
        [command] + args,
        input=input_data,
        capture_output=True,
        text=True,
        timeout=15
    )
    stdout = result.stdout.strip()
    if not stdout:
        raise RuntimeError(
            f"No stdout. Return code: {result.returncode}. stderr: {result.stderr[:500]}"
        )
    return json.loads(stdout)


def mcp_multi_roundtrip(command, args, messages):
    """Send multiple JSON-RPC messages and capture all responses."""
    input_data = '\n'.join(json.dumps(m) for m in messages) + '\n'
    result = subprocess.run(
        [command] + args,
        input=input_data,
        capture_output=True,
        text=True,
        timeout=15
    )
    stdout = result.stdout.strip()
    if not stdout:
        raise RuntimeError(
            f"No stdout. Return code: {result.returncode}. stderr: {result.stderr[:500]}"
        )
    lines = [l.strip() for l in stdout.split('\n') if l.strip()]
    return [json.loads(l) for l in lines]


def test_vision_client_basic():
    """Python can talk to agentic-vision-mcp"""
    vision_path = fresh_temp_path('.avis')

    init_req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    }

    response = mcp_roundtrip(
        './target/release/agentic-vision-mcp',
        ['serve', '--vision', vision_path],
        init_req
    )

    assert "result" in response, f"Expected 'result' in response, got: {response}"
    assert response["result"]["protocolVersion"] == "2024-11-05"
    assert response["result"]["serverInfo"]["name"] == "agentic-vision-mcp"
    assert response["result"]["serverInfo"]["version"] == expected_vision_mcp_version()

    # Check capabilities
    caps = response["result"]["capabilities"]
    assert "tools" in caps
    assert "resources" in caps
    assert "prompts" in caps
    assert "logging" in caps

    # Clean up
    if os.path.exists(vision_path):
        os.unlink(vision_path)
    print("✓ Vision client basic test passed")


def test_vision_tools_list():
    """Python can list tools from agentic-vision-mcp"""
    vision_path = fresh_temp_path('.avis')

    messages = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2024-11-05", "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }},
        {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    ]

    responses = mcp_multi_roundtrip(
        './target/release/agentic-vision-mcp',
        ['serve', '--vision', vision_path],
        messages
    )

    assert len(responses) >= 2, f"Expected at least 2 responses, got {len(responses)}"

    # First response is initialize
    assert "result" in responses[0]

    # Second response is tools/list
    tools_resp = responses[1]
    assert "result" in tools_resp
    tools = tools_resp["result"]["tools"]
    tool_names = [t["name"] for t in tools]

    expected_tools = [
        "vision_capture", "vision_compare", "vision_query",
        "vision_similar", "vision_diff", "vision_link",
    ]
    for expected in expected_tools:
        assert expected in tool_names, f"Missing tool: {expected}. Found: {tool_names}"

    if os.path.exists(vision_path):
        os.unlink(vision_path)
    print(f"✓ Vision tools list test passed ({len(tools)} tools: {tool_names})")


def test_memory_client_basic():
    """Python can talk to agentic-memory-mcp"""
    memory_path = fresh_temp_path('.amem')

    init_req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "1.0"}
        }
    }

    response = mcp_roundtrip(
        'agentic-memory-mcp',
        ['serve', '--memory', memory_path],
        init_req
    )

    assert "result" in response, f"Expected 'result' in response, got: {response}"
    assert response["result"]["protocolVersion"] == "2024-11-05"
    assert response["result"]["serverInfo"]["name"] == "agentic-memory-mcp"

    if os.path.exists(memory_path):
        os.unlink(memory_path)
    print("✓ Memory client basic test passed")


if __name__ == "__main__":
    test_vision_client_basic()
    test_vision_tools_list()
    test_memory_client_basic()
    print("\n✓ All integration tests PASSED")
