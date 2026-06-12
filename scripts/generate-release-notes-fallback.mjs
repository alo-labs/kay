#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import fs from "node:fs";

const versionInput = process.env.NEW_VERSION || process.argv[2] || "";
const version = versionInput.replace(/^v/, "");

if (!version) {
  console.error("NEW_VERSION or argv[2] is required");
  process.exit(1);
}

const currentTag = `v${version}`;
const repo = process.env.GITHUB_REPOSITORY || "alo-labs/kay";
const outputPath =
  process.env.RELEASE_NOTES_PATH || "docs/release-notes/RELEASE_NOTES.md";

function git(args, fallback = "") {
  try {
    return execFileSync("git", args, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return fallback;
  }
}

function previousTag() {
  const envTag = (process.env.PREV_TAG || "").trim();
  if (envTag) {
    return envTag;
  }
  return git(["tag", "--sort=-v:refname"])
    .split(/\r?\n/)
    .map((tag) => tag.trim())
    .find((tag) => tag && tag !== currentTag);
}

function releaseRange(prevTag) {
  if (prevTag) {
    return `${prevTag}..HEAD`;
  }
  const root = git(["rev-list", "--max-parents=0", "HEAD"]);
  return root ? `${root}..HEAD` : "HEAD";
}

function commits(range) {
  const raw = git(
    ["log", "--no-merges", "--format=%H%x00%h%x00%s", "--abbrev=8", range],
    "",
  );
  return raw
    .split(/\r?\n/)
    .map((line) => line.split("\0"))
    .filter((parts) => parts.length === 3)
    .map(([hash, shortHash, subject]) => ({
      hash,
      shortHash,
      subject,
      normalized: normalizeSubject(subject),
      files: filesForCommit(hash),
    }))
    .filter((commit) => !isMechanicalPreviousReleaseNote(commit.subject));
}

function filesForCommit(hash) {
  return git(["diff-tree", "--no-commit-id", "--name-only", "-r", hash], "")
    .split(/\r?\n/)
    .map((file) => file.trim())
    .filter(Boolean);
}

function changedFiles(range) {
  return git(["diff", "--name-only", range], "")
    .split(/\r?\n/)
    .map((file) => file.trim())
    .filter(Boolean);
}

function normalizeSubject(subject) {
  return subject
    .replace(/\s*\[skip ci\]\s*/gi, "")
    .replace(/^[a-z]+(?:\([^)]+\))?!?:\s*/i, "")
    .replace(/\.$/, "")
    .trim();
}

function isMechanicalPreviousReleaseNote(subject) {
  return /^docs\(release\): refresh v\d+\.\d+\.\d+ notes/i.test(subject);
}

function categoryForCommit(commit, files) {
  const subject = commit.subject.toLowerCase();
  const touched = files.join("\n");

  if (
    subject.includes("(silver)") ||
    subject.includes("silver") ||
    /(^|\/)silver-bullet\.md$|(^|\/)\.silver-bullet\.json$|docs\/doc-scheme|docs\/task-doc-checklist/.test(
      touched,
    )
  ) {
    return "Silver Bullet Governance";
  }

  if (
    subject.startsWith("feat") ||
    subject.includes("feature") ||
    subject.includes("add ")
  ) {
    return "Features";
  }

  if (
    subject.startsWith("fix") ||
    subject.includes("bug") ||
    subject.includes("repair") ||
    subject.includes("restore")
  ) {
    return "Bug Fixes";
  }

  if (
    /(^|\/)(release\.yml|package\.json)$|^codex-cli\/package\.json$|^scripts\/install\//.test(
      touched,
    ) ||
    subject.includes("release") ||
    subject.includes("package")
  ) {
    return "Release And Packaging";
  }

  if (/^docs\//.test(touched) || subject.startsWith("docs")) {
    return "Documentation";
  }

  if (/test|nextest|pre-release|build-fast/i.test(touched + "\n" + subject)) {
    return "Testing And Verification";
  }

  return "Maintenance";
}

function buildCategories(commitList, files) {
  const categories = new Map();
  for (const commit of commitList) {
    const category = categoryForCommit(commit, commit.files || []);
    if (!categories.has(category)) {
      categories.set(category, []);
    }
    categories.get(category).push(commit);
  }

  if (files.includes("codex-cli/package.json")) {
    const releaseCategory = "Release And Packaging";
    if (!categories.has(releaseCategory)) {
      categories.set(releaseCategory, []);
    }
    const hasPackageCommit = categories
      .get(releaseCategory)
      .some((commit) => /^\d+\.\d+\.\d+$/.test(commit.normalized || ""));
    if (!hasPackageCommit) {
      categories.get(releaseCategory).push({
        shortHash: currentTag,
        normalized: `Publish package metadata for @alo-labs/kay ${version}`,
      });
    }
  }

  return categories;
}

function bulletFor(commit) {
  const text =
    /^\d+\.\d+\.\d+$/.test(commit.normalized || "")
      ? `Publish package metadata for @alo-labs/kay ${commit.normalized}`
      : commit.normalized || commit.subject || "Update release contents";
  return `- ${text.charAt(0).toUpperCase()}${text.slice(1)} (${commit.shortHash}).`;
}

function intro(categories) {
  const names = [...categories.keys()];
  if (names.length === 0) {
    return "This release contains release maintenance updates.";
  }
  const readable = names
    .map((name) => name.toLowerCase())
    .join(", ")
    .replace(/, ([^,]*)$/, ", and $1");
  return `This release includes ${readable} updates derived from the changes since the previous tag.`;
}

function render() {
  const prevTag = previousTag();
  const range = releaseRange(prevTag);
  const fileList = changedFiles(range);
  const commitList = commits(range);
  const categories = buildCategories(commitList, fileList);
  const lines = [`## @alo-labs/kay v${version}`, "", intro(categories), ""];

  for (const [category, categoryCommits] of categories.entries()) {
    const unique = [];
    const seen = new Set();
    for (const commit of categoryCommits) {
      const key = `${commit.shortHash}:${commit.normalized}`;
      if (seen.has(key)) {
        continue;
      }
      seen.add(key);
      unique.push(commit);
    }
    if (unique.length === 0) {
      continue;
    }
    lines.push(`### ${category}`, "");
    lines.push(...unique.map(bulletFor), "");
  }

  lines.push("### Install", "");
  lines.push("```bash");
  lines.push("npm install -g @alo-labs/kay@latest");
  lines.push("kay");
  lines.push("```");

  if (prevTag) {
    lines.push("");
    lines.push(`Compare: https://github.com/${repo}/compare/${prevTag}...${currentTag}`);
  }

  return `${lines.join("\n")}\n`;
}

fs.mkdirSync(outputPath.replace(/\/[^/]+$/, ""), { recursive: true });
fs.writeFileSync(outputPath, render());
