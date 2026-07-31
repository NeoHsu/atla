#!/usr/bin/env python3
"""Run cargo-chef cook without replacing files in the source worktree."""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=ROOT,
        help="repository root (defaults to this script's parent repository)",
    )
    parser.add_argument(
        "--recipe-path",
        type=Path,
        default=Path(".mise-recipe.json"),
        help="recipe path, relative to the repository root by default",
    )
    parser.add_argument(
        "--target-dir",
        type=Path,
        help="Cargo target directory (defaults to CARGO_TARGET_DIR or <repo>/target)",
    )
    return parser.parse_args()


def resolve_from(root: Path, path: Path) -> Path:
    return path.resolve() if path.is_absolute() else (root / path).resolve()


def main() -> int:
    args = parse_args()
    try:
        root = args.repo_root.resolve(strict=True)
    except OSError as error:
        sys.stderr.write(f"cargo-chef cook failed: invalid repository root: {error}\n")
        return 2

    recipe = resolve_from(root, args.recipe_path)
    if not recipe.is_file():
        sys.stderr.write(
            f"cargo-chef cook failed: recipe not found at {recipe}; "
            "run cargo chef prepare first\n"
        )
        return 2

    configured_target = args.target_dir
    if configured_target is None:
        configured_target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
    target_dir = resolve_from(root, configured_target)

    cargo = os.environ.get("CARGO", "cargo")
    with tempfile.TemporaryDirectory(prefix="atla-cargo-chef-") as temporary:
        workspace = Path(temporary)
        isolated_recipe = workspace / "recipe.json"
        shutil.copy2(recipe, isolated_recipe)
        command = [
            cargo,
            "chef",
            "cook",
            "--workspace",
            "--recipe-path",
            str(isolated_recipe),
            "--target-dir",
            str(target_dir),
        ]
        try:
            return subprocess.run(command, cwd=workspace, check=False).returncode
        except OSError as error:
            sys.stderr.write(f"cargo-chef cook failed: {error}\n")
            return 127


if __name__ == "__main__":
    raise SystemExit(main())
