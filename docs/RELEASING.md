# Workstart release packaging

Workstart produces two x86-64 desktop packages:

- an RPM for Fedora 44;
- a per-user NSIS installer for Windows 10 and 11.

The repository contains unsigned development packaging commands. It does not
contain certificates, private keys, passphrases, signing-service credentials,
or placeholder signing identities.

## Icon assets

`src-tauri/icons/icon.png` is the original 1254×1254 transparent RGBA source.
Do not resize or optimize it in place. Regenerate the checked-in derivatives
with ImageMagick 7:

```sh
pnpm icons:generate
pnpm validate:packaging
```

The generator creates 32, 128, 256, and 512 pixel RGBA PNGs for Linux and a
multi-resolution ICO containing 16, 24, 32, 48, 64, 128, and 256 pixel frames
for Windows.

## Preflight verification

Run these checks on the release commit before building either package:

```sh
pnpm install --frozen-lockfile
pnpm test
pnpm check
pnpm build
pnpm validate:packaging
cargo test --manifest-path src-tauri/Cargo.toml
pnpm build:debug
```

`build:debug` deliberately passes `--no-bundle`; it validates the merged Tauri
configuration and application build without producing an installer.

## Unsigned development packages

Build each package on its target operating system. These commands do not sign,
install, or publish anything:

```sh
# Fedora 44 x86-64
pnpm package:fedora:unsigned

# Windows 10/11 x86-64, from a Developer Command Prompt
pnpm package:windows:unsigned
```

The Fedora configuration creates only an RPM and installs the desktop entry in
the `Office` and `Utility` categories. The Windows configuration creates only
an NSIS installer, installs for the current user, creates a Workstart Start
menu entry, and downloads the WebView2 bootstrapper when required. It does not
request administrator access and does not create an MSI.

## Windows release signing prerequisite

A public Windows release requires an Authenticode code-signing certificate
whose private key is available to the trusted Windows release host (or through
an approved hardware-backed signing service). Before the release build:

1. Import or connect the certificate outside the repository and record its
   certificate thumbprint in the release system's secret store.
2. Create `src-tauri/tauri.signing.conf.json`, which is ignored by Git, as a
   local Tauri override containing `certificateThumbprint`, `digestAlgorithm:
   "sha256"`, and the certificate provider's RFC 3161 timestamp URL. Do not
   edit the checked-in config with secret or machine-specific values.
3. Pass that override with `--config src-tauri/tauri.signing.conf.json` while
   building the NSIS target. Tauri signs the application binary and installer
   using the Windows signing tools.
4. Verify both signatures on a clean Windows 10 and Windows 11 system with
   `Get-AuthenticodeSignature` or `signtool verify /pa`.

Treat certificate files, hardware-token PINs, cloud signing credentials, and
timestamp-service credentials as release secrets. Never add them to Git or CI
logs.

## RPM release signing prerequisite

A public Fedora release requires a dedicated RPM GPG signing subkey held
outside the repository. Build the unsigned RPM first, sign the finished
artifact with the release host's `rpmsign --addsign` workflow, and verify it
with `rpm --checksig`. The RPM signing key ID and passphrase belong in the
release system's protected configuration, not in Tauri config or this
repository.

## Release checklist

- [ ] The release commit is tagged with the same version as `package.json`,
  `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- [ ] All preflight verification commands pass on a clean checkout.
- [ ] Icon derivatives were regenerated and packaging validation passes.
- [ ] The RPM was built on Fedora 44 x86-64 and its signature verifies.
- [ ] The NSIS installer was built on Windows x86-64 and both Authenticode
  signatures verify.
- [ ] RPM contents, dependencies, desktop entry, and application icon were
  inspected before release.
- [ ] The per-user NSIS install, Start menu launch, WebView2 bootstrap path,
  upgrade, and uninstall were smoke-tested without elevation on Windows 10 and
  Windows 11.
- [ ] SHA-256 checksums were generated from the final signed artifacts and
  independently verified.
- [ ] No signing material, local overrides, machine paths, or credentials are
  present in the commit or packaged files.
- [ ] Only the signed artifacts and checksum file are attached to the intended
  release after all checks pass.
