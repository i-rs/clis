#!/usr/bin/env python3
"""Test the i-rs-mcp server via stdio transport (newline-delimited JSON)."""

import json
import subprocess
import sys
import os
import threading

MCP_BIN = os.path.join(os.path.dirname(__file__), "..", "..", "target", "debug", "i-rs-mcp")


def stderr_reader(proc):
    """Read stderr in background and print to console."""
    for line in iter(proc.stderr.readline, b''):
        sys.stdout.write(f"  [stderr] {line.decode(errors='replace').rstrip()}\n")


def send(proc, msg):
    """Send a JSON-RPC message (newline-delimited)."""
    line = json.dumps(msg, ensure_ascii=False) + "\n"
    proc.stdin.write(line.encode())
    proc.stdin.flush()


def recv(proc, timeout=10):
    """Read a JSON-RPC response (one line = one message)."""
    # Use select + readline for reliability
    import select
    r, _, _ = select.select([proc.stdout], [], [], timeout)
    if not r:
        return None
    raw = proc.stdout.readline()
    if not raw:
        return None
    return json.loads(raw.decode(errors='replace').strip())


def assert_ok(resp, step):
    """Assert response contains a 'result' key (not an error)."""
    assert resp is not None, f"{step}: 没有收到响应"
    assert "result" in resp, f"{step}: 响应中没有 result: {resp.get('error', resp)}"


def test():
    print("=" * 60)
    print("i-rs-mcp 功能测试")
    print("=" * 60)

    proc = subprocess.Popen(
        [MCP_BIN],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    # Read stderr in background
    t = threading.Thread(target=stderr_reader, args=(proc,), daemon=True)
    t.start()

    try:
        # 1. 初始化
        print("\n[1/5] 初始化...")
        send(proc, {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0"},
            },
        })
        resp = recv(proc)
        assert_ok(resp, "初始化")
        info = resp["result"]["serverInfo"]
        print(f"  ✅ 服务器 {info['name']} v{info['version']}")

        # 2. initialized 通知
        print("\n[2/5] 发送 initialized 通知...")
        send(proc, {"jsonrpc": "2.0", "method": "notifications/initialized"})

        # 3. 列出工具
        print("\n[3/5] 列出工具...")
        send(proc, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
        resp = recv(proc)
        assert_ok(resp, "tools/list")
        tools = resp["result"]["tools"]
        tool_names = [t["name"] for t in tools]
        print(f"  ✅ 共 {len(tools)} 个工具")
        print(f"  示例: {', '.join(tool_names[:6])}...")
        assert len(tools) == 260, f"期望 260 个工具, 实际 {len(tools)}"

        # 4. 调用 weight_list
        print("\n[4/5] 调用 weight_list...")
        send(proc, {
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {"name": "weight_list", "arguments": {}},
        })
        resp = recv(proc)
        assert_ok(resp, "weight_list")
        result = resp["result"]
        is_err = result.get("isError", False)
        if is_err:
            print(f"  ⚠️ 返回错误: {result['content'][0]['text'][:200]}")
        else:
            print(f"  ✅ weight_list 调用成功")

        # 5. 新增 weight 条目
        print("\n[5/5] 新增 weight 条目...")
        send(proc, {
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "weight_add",
                "arguments": {"weight": 75.0, "date": "2026-05-19"},
            },
        })
        resp = recv(proc)
        assert_ok(resp, "weight_add")
        result = resp["result"]
        is_err = result.get("isError", False)
        if is_err:
            print(f"  ⚠️ 返回错误: {result['content'][0]['text'][:200]}")
        else:
            print(f"  ✅ weight_add 调用成功")
            # 验证返回的数据包含 id
            text_data = result["content"][0].get("text", "")
            if text_data:
                parsed = json.loads(text_data)
                print(f"      id: {parsed.get('id', 'N/A')[:8]}..., weight: {parsed.get('weight', 'N/A')}")

        # 6. 删除刚新增的条目
        print("\n[6/6] 清理: 删除 weight 条目...")
        resp_id = json.loads(result["content"][0]["text"]).get("id", "")
        send(proc, {
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "weight_delete",
                "arguments": {"id": resp_id},
            },
        })
        resp = recv(proc)
        assert_ok(resp, "weight_delete")
        print(f"  ✅ weight_delete 成功")

        print("\n" + "=" * 60)
        print("🎉 全部测试通过！")
        print("=" * 60)

    except Exception as e:
        print(f"\n❌ 测试失败: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
    finally:
        proc.terminate()
        proc.wait(timeout=3)


if __name__ == "__main__":
    test()

