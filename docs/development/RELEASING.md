# Releasing Xunlie

Xunlie releases are built by GitHub Actions from a SemVer tag that must be treated as immutable. The
release workflow publishes two archives:

- `xunlie-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `xunlie-vX.Y.Z-x86_64-pc-windows-msvc.zip`

Each archive contains the `xunlie` executable, `LICENSE`, and `README.md`. The release also includes
`SHA256SUMS`, generated release notes, and GitHub build-provenance attestations for both archives.

## Preconditions

1. The release commit is merged into `main`, and all required checks are green.
2. The version under `[workspace.package]` in `Cargo.toml` is the intended SemVer version.
3. Internal path-dependency versions and `Cargo.lock` agree with that workspace version.
4. User-visible changes are represented by merged pull requests, because GitHub generates release
   notes from those pull requests.
5. `cargo xtask quality` passes from a clean checkout.

The workflow rejects a tag when its version differs from `Cargo.toml`, its name is not
`vMAJOR.MINOR.PATCH` SemVer, or the tagged commit is not reachable from `origin/main`.

## Publish a release

Create the tag only after the version bump has landed on `main`. Prefer a signed annotated tag:

```console
git switch main
git pull --ff-only origin main
cargo xtask quality
git tag -s v0.1.0 -m "Xunlie v0.1.0"
git push origin v0.1.0
```

Pushing the tag triggers `.github/workflows/release.yml`. It builds and smoke-tests the Linux and
Windows binaries, packages them, verifies their checksums, records provenance, and only then creates
the GitHub release. Do not create the GitHub release manually or move an existing release tag.

If signing is unavailable, an annotated tag (`git tag -a`) still preserves release metadata, but it
does not provide author cryptographic verification. Repository tag protection or a tag ruleset should
restrict `v*` tag creation to maintainers.

## Verify downloaded artifacts

Download all assets into one directory and verify their SHA-256 digests:

```console
gh release download v0.1.0 --repo somefirenoodles/xunlie
sha256sum --check SHA256SUMS
```

On PowerShell, verify an individual asset against the corresponding line in `SHA256SUMS`:

```powershell
(Get-FileHash -Algorithm SHA256 .\xunlie-v0.1.0-x86_64-pc-windows-msvc.zip).Hash.ToLowerInvariant()
Get-Content .\SHA256SUMS
```

Verify its GitHub-hosted provenance with GitHub CLI:

```console
gh attestation verify xunlie-v0.1.0-x86_64-unknown-linux-gnu.tar.gz \
  --repo somefirenoodles/xunlie \
  --signer-workflow somefirenoodles/xunlie/.github/workflows/release.yml
```

The attestation proves that GitHub Actions in this repository produced the archived bytes. It does
not make the executable reproducible and does not replace checksum verification or code review.

## Failure and recovery

- If a build, smoke test, checksum, or attestation step fails, no GitHub release is created. Fix the
  cause on `main`, bump the version if needed, and publish a new tag.
- If infrastructure fails before release creation, re-running the failed workflow is safe: all assets
  are rebuilt from the tagged source.
- If a release was already published, do not overwrite its assets or retarget its tag. Publish a patch
  release instead. Deleting a published release does not erase already downloaded artifacts.

## Current trust boundary

The release archives are native binaries built on GitHub-hosted `ubuntu-24.04` and `windows-2025`
runners. The workflow pins every referenced Action to a full commit SHA, uses the locked Cargo graph,
grants write permissions only to the final publish job, and does not persist checkout credentials.
Reproducible builds, code signing, and an independent release-builder comparison remain future
hardening work.
