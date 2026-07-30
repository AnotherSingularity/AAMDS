# RC1 Tag Preservation

The annotated tag `aeon-air-defense-rc1` was created in the RC1 freeze
environment but the session's git proxy refused the tag push
(HTTP 403 on `refs/tags/*` — the same push run succeeded for
`refs/heads/claude/aeon-air-defense-layer-wico33`). Because an
annotated tag is an independent git object, cloning the branch does
**not** transfer the tag. These files preserve the exact tag object
so a sponsor-owned runner can publish it verbatim.

## Recorded object identity

| Field | Value |
|---|---|
| Tag object SHA-1 | `8bc2edac76afbb75ded176c2f39c717e39784297` |
| Points to commit | `d1b8414181bc164b426ef23e4f591ec5c3c5eeb7` |
| Tag name         | `aeon-air-defense-rc1` |
| Tagger           | `Claude <noreply@anthropic.com>` |

The tagger identity and timestamp are baked into the tag object;
re-creating the tag with `git tag -a` on a different machine or at a
different time produces a *different* tag-object hash even if the
target commit and message text are identical. That is why this
preservation flow uses `git bundle`, not `git tag -a`.

## Files

| File | Purpose |
|---|---|
| `aeon-air-defense-rc1-tag.bundle` | Complete git bundle containing the tag object. `git bundle verify` PASSES. |
| `aeon-air-defense-rc1-tag-record.txt` | Human-readable `git cat-file -p` output for audit / cross-check. |

## Sponsor-side publication

```
# On any runner with direct write access to AnotherSingularity/AAMDS:
git clone <repository-url> AAMDS
cd AAMDS
git fetch /path/to/aeon-air-defense-rc1-tag.bundle \
    refs/tags/aeon-air-defense-rc1:refs/tags/aeon-air-defense-rc1

# Verify the imported tag hits the RC1 commit exactly.
git rev-parse aeon-air-defense-rc1^{commit}
# Must equal:
# d1b8414181bc164b426ef23e4f591ec5c3c5eeb7

git rev-parse aeon-air-defense-rc1
# Must equal:
# 8bc2edac76afbb75ded176c2f39c717e39784297

# Publish the tag.
git push origin refs/tags/aeon-air-defense-rc1
git ls-remote --tags origin aeon-air-defense-rc1
```

If either `rev-parse` output does not match the values above, do
**not** push — the bundle was corrupted in transit and RC1 preservation
integrity has been lost.

## Alternative recovery from the pushed branch

The bundle above is the authoritative preservation path. If for any
reason it is unavailable but the branch is present, a sponsor can
observe the RC1 evidence at commit `d1b8414`
(release/AEON_AIR_DEFENSE_RC1_MANIFEST.json, RC1_CONTRACT_FREEZE.md,
docs/evidence/releases/aeon-air-defense-rc1/) and issue a *new* tag
against that commit. The new tag object will have a **different**
SHA-1 than `8bc2eda…` and MUST be given a different name
(e.g. `aeon-air-defense-rc1-republished`) so the two tag objects do
not collide in any downstream evidence store.
