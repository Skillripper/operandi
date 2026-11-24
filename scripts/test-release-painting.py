#!/usr/bin/env python3
"""Test the real release scripts through their Cargo invocation, without packaging."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent


def test_release_painting():
    scratch = ROOT / "development"
    scratch.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch) as temp:
        root = Path(temp)
        scripts = root / "scripts"
        scripts.mkdir()
        for name in ("build-release.sh", "release-macos.sh"):
            shutil.copyfile(ROOT / "scripts" / name, scripts / name)

        tools = root / "tools"
        tools.mkdir()

        def stub(path, body):
            path.write_text("#!/bin/sh\n" + body)
            path.chmod(0o755)

        stub(tools / "uname", 'printf "%s\\n" "$TEST_PLATFORM"\n')
        stub(tools / "git", 'case "$1" in rev-parse) echo hardening-test;; rev-list) echo 1;; esac\n')
        stub(tools / "security", 'printf \'"%s"\\n\' "$OPERANDI_SIGN_IDENTITY"\n')
        stub(tools / "cargo", 'printf "%s\\n" "$@" > "$TEST_CARGO_ARGS"\nexit 73\n')
        stub(tools / "cmake", "exit 0\n")
        stub(scripts / "build-cycles.sh", "exit 0\n")
        stub(scripts / "build-usd.sh", "exit 0\n")
        manifest = root / "crates" / "operandi-app" / "Cargo.toml"
        manifest.parent.mkdir(parents=True)
        shutil.copyfile(ROOT / "crates" / "operandi-app" / "Cargo.toml", manifest)
        args_path = root / "cargo-args"
        env = dict(
            os.environ,
            PATH=f"{tools}{os.pathsep}{os.environ['PATH']}",
            CARGO_BIN=str(tools / "cargo"),
            TEST_CARGO_ARGS=str(args_path),
            DEVELOPER_DIR=str(root),
            OPERANDI_RELEASE_OUTPUT=str(root / "out"),
            OPERANDI_SIGN_IDENTITY="Test Signer",
            OPERANDI_CYCLES_LIB_DIR="",
            OPERANDI_CYCLES_OUT="",
        )
        for platform, script, usd_values in (
            ("Linux", "build-release.sh", ("0",)),
            ("MINGW64_NT", "build-release.sh", ("0",)),
            ("Darwin", "release-macos.sh", ("0", "1")),
        ):
            for usd in usd_values:
                for painting in (None, "0", "1"):
                    run_env = dict(env, TEST_PLATFORM=platform, OPERANDI_USD=usd)
                    run_env.pop("OPERANDI_PAINTING", None)
                    if painting is not None:
                        run_env["OPERANDI_PAINTING"] = painting
                    result = subprocess.run(
                        ["sh", scripts / script], env=run_env, capture_output=True, text=True
                    )
                    assert result.returncode == 73, result.stdout + result.stderr
                    actual = args_path.read_text().splitlines()
                    features = ["cycles"]
                    if painting != "0":
                        features.append("painting")
                    if usd == "1":
                        features.append("usd")
                    expected = [
                        "build", "--locked", "--release", "-p", "operandi-app",
                        "--features", ",".join(features),
                    ]
                    if painting == "0":
                        expected.append("--no-default-features")
                    assert actual == expected, (platform, usd, painting, actual)

                    graph = subprocess.check_output(
                        ["cargo", "tree", actual[1], *actual[3:], "-e", "normal", "--prefix", "none"],
                        cwd=ROOT,
                        text=True,
                    )
                    has_painting = any(line.startswith("operandi-paint v") for line in graph.splitlines())
                    assert has_painting == (painting != "0"), (platform, usd, painting)


if __name__ == "__main__":
    test_release_painting()
