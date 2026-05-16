import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, beforeEach } from "@jest/globals";

const originalKayHome = process.env.KAY_HOME;
let currentKayHome: string | undefined;

beforeEach(async () => {
  currentKayHome = await fs.mkdtemp(path.join(os.tmpdir(), "kay-sdk-test-"));
  process.env.KAY_HOME = currentKayHome;
});

afterEach(async () => {
  const kayHomeToDelete = currentKayHome;
  currentKayHome = undefined;

  if (originalKayHome === undefined) {
    delete process.env.KAY_HOME;
  } else {
    process.env.KAY_HOME = originalKayHome;
  }

  if (kayHomeToDelete) {
    await fs.rm(kayHomeToDelete, { recursive: true, force: true });
  }
});
