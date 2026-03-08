#!/usr/bin/env python3
"""
Quick validation script for skills.
"""

import re
import sys
from pathlib import Path

import yaml


ALLOWED_PROPERTIES = {"name", "description", "license", "allowed-tools", "metadata"}


def validate_skill(skill_path):
    skill_path = Path(skill_path)
    skill_md = skill_path / "SKILL.md"
    if not skill_md.exists():
        return False, "SKILL.md not found"

    content = skill_md.read_text()
    if not content.startswith("---"):
        return False, "No YAML frontmatter found"

    match = re.match(r"^---\n(.*?)\n---", content, re.DOTALL)
    if not match:
        return False, "Invalid frontmatter format"

    try:
        frontmatter = yaml.safe_load(match.group(1))
    except yaml.YAMLError as err:
        return False, f"Invalid YAML in frontmatter: {err}"

    if not isinstance(frontmatter, dict):
        return False, "Frontmatter must be a YAML dictionary"

    unexpected_keys = set(frontmatter.keys()) - ALLOWED_PROPERTIES
    if unexpected_keys:
        return (
            False,
            "Unexpected key(s) in SKILL.md frontmatter: "
            + ", ".join(sorted(unexpected_keys)),
        )

    name = frontmatter.get("name")
    description = frontmatter.get("description")
    if not isinstance(name, str) or not name.strip():
        return False, "Missing or invalid 'name' in frontmatter"
    if not isinstance(description, str) or not description.strip():
        return False, "Missing or invalid 'description' in frontmatter"

    normalized_name = name.strip()
    if not re.match(r"^[a-z0-9-]+$", normalized_name):
        return False, "Name must be hyphen-case"
    if normalized_name.startswith("-") or normalized_name.endswith("-") or "--" in normalized_name:
        return False, "Name cannot start/end with hyphen or contain consecutive hyphens"
    if len(normalized_name) > 64:
        return False, "Name exceeds the 64 character limit"

    normalized_description = description.strip()
    if "<" in normalized_description or ">" in normalized_description:
        return False, "Description cannot contain angle brackets"
    if len(normalized_description) > 1024:
        return False, "Description exceeds the 1024 character limit"

    return True, "Skill is valid"


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: quick_validate.py <skill_directory>")
        sys.exit(1)

    valid, message = validate_skill(sys.argv[1])
    print(message)
    sys.exit(0 if valid else 1)
