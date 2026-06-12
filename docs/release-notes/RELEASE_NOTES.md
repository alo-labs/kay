## @alo-labs/kay v0.9.24

This patch completes the release cleanup after v0.9.23 by normalizing the
Silver Bullet doc-scheme checklist and publishing the follow-up package
metadata bump. There are no runtime changes beyond the release metadata update.

### Changes

- Docs: normalize doc-scheme section checklist statuses to exact `updated`
  tokens so the completion gate accepts the current-session release checklist.
- Docs: refresh release/install audit guidance in slash-command and DevOps
  workflow docs.
- Release: publish the v0.9.24 package metadata bump and verify the local
  `kay` command upgrades to the latest standalone package.

### Install

```bash
npm install -g @alo-labs/kay@latest
kay
```

Compare: https://github.com/alo-labs/kay/compare/v0.9.23...v0.9.24
