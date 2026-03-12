#!/usr/bin/env python3
"""Build a minimal static HTML site from tracked Markdown files."""

from __future__ import annotations

import argparse
import html
import os
import re
import shutil
import subprocess
from pathlib import Path

from jinja2 import Template
from markdown_it import MarkdownIt

SITE_CSS_PATH = "_static/site.css"

HTML_TEMPLATE = Template(
    """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{{ title }}</title>
  <link rel="stylesheet" href="{{ stylesheet_href }}">
</head>
<body>
  <header class="site-header">
    <div class="site-header__inner">
      <a class="site-title" href="{{ home_href }}">TopClaw Docs</a>
      <nav class="site-nav">
        <a href="{{ docs_hub_href }}">Docs Hub</a>
        <a href="{{ toc_href }}">Summary</a>
        <a href="{{ repo_readme_href }}">README</a>
      </nav>
    </div>
  </header>
  <main class="page-shell">
    <aside class="page-meta">
      <p class="page-meta__label">Source</p>
      <p><code>{{ source_path }}</code></p>
      <p><a href="{{ github_source_href }}">View on GitHub</a></p>
    </aside>
    <article class="page-content markdown-body">
      {{ body|safe }}
    </article>
  </main>
</body>
</html>
"""
)

SITE_CSS = """\
:root {
  color-scheme: light;
  --bg: #f7f4ed;
  --panel: #fffdf8;
  --text: #1f1f1a;
  --muted: #5a5a4f;
  --border: #d8d3c4;
  --accent: #0c6d62;
  --code-bg: #f1ede3;
  --link: #045c8c;
}

* { box-sizing: border-box; }
body {
  margin: 0;
  background: linear-gradient(180deg, #f8f2e4 0%, var(--bg) 100%);
  color: var(--text);
  font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua", serif;
  line-height: 1.6;
}
.site-header {
  position: sticky;
  top: 0;
  border-bottom: 1px solid var(--border);
  background: rgba(255, 253, 248, 0.94);
  backdrop-filter: blur(12px);
}
.site-header__inner,
.page-shell {
  width: min(1120px, calc(100% - 32px));
  margin: 0 auto;
}
.site-header__inner {
  display: flex;
  gap: 16px;
  align-items: center;
  justify-content: space-between;
  padding: 14px 0;
}
.site-title,
.site-nav a,
a {
  color: var(--link);
  text-decoration: none;
}
.site-title { font-weight: 700; }
.site-nav {
  display: flex;
  gap: 18px;
  flex-wrap: wrap;
}
.page-shell {
  display: grid;
  grid-template-columns: minmax(180px, 220px) minmax(0, 1fr);
  gap: 24px;
  padding: 28px 0 48px;
}
.page-meta {
  align-self: start;
  position: sticky;
  top: 72px;
  padding: 18px;
  border: 1px solid var(--border);
  border-radius: 14px;
  background: rgba(255, 253, 248, 0.9);
}
.page-meta__label {
  margin-top: 0;
  font-size: 0.82rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
}
.page-content {
  min-width: 0;
  padding: 28px;
  border: 1px solid var(--border);
  border-radius: 18px;
  background: var(--panel);
  box-shadow: 0 18px 50px rgba(56, 45, 18, 0.06);
}
.markdown-body h1,
.markdown-body h2,
.markdown-body h3 {
  line-height: 1.2;
}
.markdown-body code {
  padding: 0.15rem 0.35rem;
  border-radius: 6px;
  background: var(--code-bg);
  font-family: "SFMono-Regular", Consolas, "Liberation Mono", monospace;
}
.markdown-body pre {
  overflow-x: auto;
  padding: 16px;
  border-radius: 12px;
  background: #1b1f23;
  color: #f4f7fb;
}
.markdown-body pre code {
  padding: 0;
  background: transparent;
  color: inherit;
}
.markdown-body blockquote {
  margin-left: 0;
  padding-left: 16px;
  border-left: 4px solid var(--border);
  color: var(--muted);
}
.markdown-body table {
  width: 100%;
  border-collapse: collapse;
}
.markdown-body th,
.markdown-body td {
  padding: 10px 12px;
  border: 1px solid var(--border);
  vertical-align: top;
}
.markdown-body img {
  max-width: 100%;
  height: auto;
}
@media (max-width: 860px) {
  .page-shell {
    grid-template-columns: 1fr;
  }
  .page-meta {
    position: static;
  }
}
"""


def tracked_markdown_files(repo_root: Path) -> list[Path]:
    try:
        proc = subprocess.run(
            ["git", "ls-files", "*.md"],
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=True,
        )
        candidates = [Path(line) for line in proc.stdout.splitlines()]
    except subprocess.CalledProcessError:
        candidates = [
            path.relative_to(repo_root)
            for path in repo_root.rglob("*.md")
            if not any(part in {"node_modules", "target", ".venv", "venv", ".git"} for part in path.parts)
        ]

    paths: list[Path] = []
    for rel in candidates:
        if any(part in {"node_modules", "target", ".venv", "venv"} for part in rel.parts):
            continue
        if len(rel.parts) == 1 or rel.parts[:1] == ("docs",) or rel == Path(".github/workflows/main-branch-flow.md"):
            paths.append(rel)
    return sorted(set(paths))


def tracked_asset_files(repo_root: Path) -> list[Path]:
    try:
        proc = subprocess.run(
            ["git", "ls-files"],
            cwd=repo_root,
            text=True,
            capture_output=True,
            check=True,
        )
        candidates = [Path(line) for line in proc.stdout.splitlines()]
    except subprocess.CalledProcessError:
        candidates = [
            path.relative_to(repo_root)
            for path in repo_root.rglob("*")
            if path.is_file() and not any(part in {"node_modules", "target", ".venv", "venv", ".git"} for part in path.parts)
        ]

    assets: list[Path] = []
    for rel in candidates:
        if rel.suffix.lower() == ".md":
            continue
        if any(part in {"node_modules", "target", ".venv", "venv"} for part in rel.parts):
            continue
        if rel.parts and rel.parts[0] == "docs":
            assets.append(rel)
    return sorted(set(assets))


def html_path_for_markdown(rel_path: Path) -> Path:
    return rel_path.with_suffix(".html")


def extra_output_paths(rel_path: Path) -> list[Path]:
    extras: list[Path] = []
    if rel_path.name == "README.md":
        if len(rel_path.parts) == 1:
            extras.append(Path("index.html"))
        else:
            extras.append(rel_path.parent / "index.html")
    return extras


def first_heading(markdown_text: str, fallback: str) -> str:
    for line in markdown_text.splitlines():
        if line.startswith("# "):
            return line[2:].strip()
    return fallback


def rewrite_local_links(
    rendered_html: str,
    source_rel: Path,
    repo_root: Path,
    md_targets: set[Path],
    asset_targets: set[Path],
) -> str:
    def replace_attr(pattern: str, attr: str, html_text: str) -> str:
        def repl(match: re.Match[str]) -> str:
            value = match.group(1)
            updated = rewrite_target(value, source_rel, repo_root, md_targets, asset_targets)
            return f'{attr}="{html.escape(updated, quote=True)}"'

        return re.sub(pattern, repl, html_text)

    rewritten = replace_attr(r'href="([^"]+)"', "href", rendered_html)
    rewritten = replace_attr(r'src="([^"]+)"', "src", rewritten)
    return rewritten


def rewrite_target(
    target: str,
    source_rel: Path,
    repo_root: Path,
    md_targets: set[Path],
    asset_targets: set[Path],
) -> str:
    if not target or target.startswith(("#", "http://", "https://", "mailto:", "tel:")):
        return target
    if target.startswith("/"):
        return target

    target_part, hash_part = split_anchor(target)
    candidate_path = (repo_root / source_rel.parent / target_part).resolve()
    try:
        candidate = candidate_path.relative_to(repo_root)
    except ValueError:
        return target
    if candidate in md_targets:
        resolved = html_path_for_markdown(candidate)
        return relative_href(html_path_for_markdown(source_rel), resolved, hash_part)
    if candidate in asset_targets:
        return relative_href(html_path_for_markdown(source_rel), candidate, hash_part)
    return target


def split_anchor(target: str) -> tuple[str, str]:
    if "#" in target:
        base, frag = target.split("#", 1)
        return base, f"#{frag}"
    return target, ""


def relative_href(from_html: Path, to_path: Path, hash_part: str = "") -> str:
    rel = os.path.relpath(to_path, start=from_html.parent)
    return f"{rel}{hash_part}"


def render_markdown(md: MarkdownIt, markdown_text: str) -> str:
    return md.render(markdown_text)


def ensure_parent(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="Build a static HTML site from repository Markdown files.")
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--output-dir", default="site")
    parser.add_argument(
        "--github-source-base",
        default="https://github.com/topway-ai/topclaw/blob/main",
        help="Base URL used for source links in generated pages.",
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    output_dir = (repo_root / args.output_dir).resolve() if not Path(args.output_dir).is_absolute() else Path(args.output_dir).resolve()

    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    markdown_files = tracked_markdown_files(repo_root)
    asset_files = tracked_asset_files(repo_root)
    md_targets = set(markdown_files)
    asset_targets = set(asset_files)

    md = (
        MarkdownIt("commonmark", {"html": True, "linkify": True, "typographer": False})
        .enable("table")
        .enable("strikethrough")
    )

    stylesheet_output = output_dir / SITE_CSS_PATH
    ensure_parent(stylesheet_output)
    stylesheet_output.write_text(SITE_CSS, encoding="utf-8")

    for rel_asset in asset_files:
        src = repo_root / rel_asset
        dst = output_dir / rel_asset
        ensure_parent(dst)
        shutil.copy2(src, dst)

    for rel_md in markdown_files:
        src = repo_root / rel_md
        markdown_text = src.read_text(encoding="utf-8")
        title = first_heading(markdown_text, rel_md.stem)
        body = rewrite_local_links(render_markdown(md, markdown_text), rel_md, repo_root, md_targets, asset_targets)
        html_output = html_path_for_markdown(rel_md)
        stylesheet_href = relative_href(html_output, Path(SITE_CSS_PATH))
        page = HTML_TEMPLATE.render(
            title=title,
            stylesheet_href=stylesheet_href,
            home_href=relative_href(html_output, Path("index.html")),
            docs_hub_href=relative_href(html_output, Path("docs/README.html")),
            toc_href=relative_href(html_output, Path("docs/SUMMARY.html")),
            repo_readme_href=relative_href(html_output, Path("README.html")),
            source_path=str(rel_md),
            github_source_href=f"{args.github_source_base}/{rel_md.as_posix()}",
            body=body,
        )

        dst = output_dir / html_output
        ensure_parent(dst)
        dst.write_text(page, encoding="utf-8")

        for extra in extra_output_paths(rel_md):
            extra_dst = output_dir / extra
            ensure_parent(extra_dst)
            extra_dst.write_text(page, encoding="utf-8")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
