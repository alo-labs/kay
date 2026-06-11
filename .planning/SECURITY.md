# Security

Security review for the release/install update path changes found no new
high-risk issues in the touched surface.

Notes:
- Homebrew ownership checks now prefer `brew which-formula` before suggesting
  uninstall or upgrade commands.
- The installer and doctor guidance avoid assuming PATH ownership when the
  formula cannot be confirmed.
- No new credential, command-injection, or release-publish regressions were
  introduced by the patch set.
