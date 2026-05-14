import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, readFile, rm, symlink, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';

test('runPostinstall resolves in dry-run mode', async () => {
  const { runPostinstall } = await import('../postinstall.js');
  process.env.CODE_POSTINSTALL_DRY_RUN = '1';
  try {
    const result = await runPostinstall({ invokedByRuntime: true, skipGlobalAlias: true });
    assert.ok(result && result.skipped === true);
  } finally {
    delete process.env.CODE_POSTINSTALL_DRY_RUN;
    delete process.env.CODE_RUNTIME_POSTINSTALL;
  }
});

test('writeWrapperFile replaces npm bin symlink without overwriting target', async () => {
  const { writeWrapperFile } = await import('../postinstall.js');
  const dir = await mkdtemp(path.join(tmpdir(), 'kay-postinstall-'));
  try {
    const target = path.join(dir, 'coder.js');
    const shim = path.join(dir, 'code');
    await writeFile(target, '#!/usr/bin/env node\nconsole.log("coder")\n');
    await symlink(target, shim);

    writeWrapperFile(shim, '#!/bin/sh\nexec "$(dirname "$0")/kay" "$@"\n', false);

    assert.equal(await readFile(target, 'utf8'), '#!/usr/bin/env node\nconsole.log("coder")\n');
    assert.match(await readFile(shim, 'utf8'), /exec "\$\(dirname "\$0"\)\/kay"/);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});
