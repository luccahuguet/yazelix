#!/usr/bin/env python3
"""Migrate br (beads_rust) JSONL to bd (gastownhall/beads) JSONL.

Usage:
    python3 scripts/migrate_br_to_bd.py .beads/issues.jsonl > .beads/migration_bd.jsonl

Then: bd import .beads/migration_bd.jsonl

Schema mapping:
  br field            -> bd field
  id                  -> id (kept as-is, yazelix-xxx prefix preserved)
  title               -> title
  description         -> description (bd supports this field in import)
  design              -> design
  acceptance_criteria -> acceptance_criteria
  notes               -> notes
  status              -> status (mapped: in_progress -> in_progress, open -> open)
  priority            -> priority (0-4, same in both)
  issue_type          -> issue_type (mapped: docs -> chore, question -> decision)
  created_at           -> created_at (br has nanosecond precision, bd uses seconds)
  created_by           -> created_by
  updated_at           -> updated_at
  labels               -> labels
  external_ref         -> external_ref
  close_reason         -> close_reason
  closed_at            -> closed_at
  dependencies         -> dependencies (type mapping: blocks -> blocks, parent-child -> parent-child)
  compaction_level     -> DROPPED (bd has ephemeral/wisp instead)
  original_size        -> DROPPED
  source_repo          -> DROPPED
  owner                -> owner (set to created_by if not present)
"""

import json
import sys
from datetime import datetime

STATUS_MAP = {
    "open": "open",
    "in_progress": "in_progress",
    "closed": "closed",
    "deferred": "deferred",
}

TYPE_MAP = {
    "task": "task",
    "bug": "bug",
    "feature": "feature",
    "epic": "epic",
    "chore": "chore",
    "decision": "decision",
    "docs": "chore",
    "question": "decision",
}

DEP_TYPE_MAP = {
    "blocks": "blocks",
    "parent-child": "parent-child",
    "related": "relates_to",
    "relates_to": "relates_to",
}


def truncate_timestamp(ts):
    """Truncate nanosecond timestamps to seconds."""
    if not ts:
        return ts
    if "." in ts:
        base, frac_and_z = ts.split(".", 1)
        frac = frac_and_z.rstrip("Z")
        if len(frac) > 6:
            frac = frac[:6]
        return f"{base}.{frac}Z"
    return ts


def migrate_issue(issue):
    out = {}

    out["id"] = issue["id"]
    out["title"] = issue.get("title", "")

    description = issue.get("description", "")
    if description:
        out["description"] = description

    design = issue.get("design", "")
    if design:
        out["design"] = design

    acceptance = issue.get("acceptance_criteria", "")
    if acceptance:
        out["acceptance_criteria"] = acceptance

    notes = issue.get("notes", "")
    if notes:
        out["notes"] = notes

    raw_status = issue.get("status", "open")
    out["status"] = STATUS_MAP.get(raw_status, raw_status)

    raw_priority = issue.get("priority", 2)
    out["priority"] = (
        raw_priority if isinstance(raw_priority, int) else int(raw_priority)
    )

    raw_type = issue.get("issue_type", "task")
    out["issue_type"] = TYPE_MAP.get(raw_type, raw_type)

    owner = issue.get("owner", issue.get("created_by", ""))
    if owner:
        out["owner"] = owner

    out["created_at"] = truncate_timestamp(issue.get("created_at", ""))
    out["created_by"] = issue.get("created_by", "")
    out["updated_at"] = truncate_timestamp(issue.get("updated_at", ""))

    if issue.get("closed_at"):
        out["closed_at"] = truncate_timestamp(issue["closed_at"])

    if issue.get("close_reason"):
        out["close_reason"] = issue["close_reason"]

    if issue.get("external_ref"):
        out["external_ref"] = issue["external_ref"]

    labels = issue.get("labels", [])
    if labels:
        out["labels"] = labels

    deps = issue.get("dependencies", [])
    if deps:
        mapped_deps = []
        for dep in deps:
            mapped_dep = {
                "issue_id": dep["issue_id"],
                "depends_on_id": dep["depends_on_id"],
                "type": DEP_TYPE_MAP.get(
                    dep.get("type", "blocks"), dep.get("type", "blocks")
                ),
            }
            if dep.get("created_at"):
                mapped_dep["created_at"] = truncate_timestamp(dep["created_at"])
            if dep.get("created_by"):
                mapped_dep["created_by"] = dep["created_by"]
            if dep.get("metadata"):
                mapped_dep["metadata"] = dep["metadata"]
            mapped_deps.append(mapped_dep)
        out["dependencies"] = mapped_deps

    out["dependency_count"] = len(deps)
    out["dependent_count"] = issue.get("dependent_count", 0)
    out["comment_count"] = issue.get("comment_count", 0)

    return out


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 migrate_br_to_bd.py <br_issues.jsonl>", file=sys.stderr)
        sys.exit(1)

    input_path = sys.argv[1]
    count = 0
    skipped = 0

    with open(input_path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            try:
                issue = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"Skipping malformed line: {e}", file=sys.stderr)
                skipped += 1
                continue

            migrated = migrate_issue(issue)
            print(json.dumps(migrated, ensure_ascii=False))
            count += 1

    print(f"\n# Migrated {count} issues, skipped {skipped}", file=sys.stderr)


if __name__ == "__main__":
    main()
