# ACA Motion Studio — ALEJANDRO Creative Academy derivative

Local source fork of [EffectCraft](https://github.com/storytold/effectcraft) at `79f695760109d548791558833b7bb1afc4f79308`.
This is an independent ACA derivative, not an official ArtCraft distribution or endorsement.

Display branding and the actual default egui palette use purple surfaces and gold controls.
Native document formats, engine/command IDs, extensions, semantic meters/errors, and saved documents remain upstream-compatible.
MIT/Apache license texts, NOTICE, contributor attribution and dependency/asset notices remain.
The upstream ArtCraft trademark artwork was excluded; ACA branding is original code-drawn type/geometry.

Browser build: cargo xtask web.
Use Rust's wasm32-unknown-unknown target and wasm-bindgen CLI exactly 0.2.129.
Source edits are implemented; successful browser compilation and interactive behavior must be recorded separately.
Do not publish the upstream installers as rebranded Academy binaries or claim upstream features are tested by this palette change.

## Academy build pipeline

The fork-only `Academy WASM build` workflow is manual (`workflow_dispatch`) and uses two sequential jobs on standard public Ubuntu runners. It builds this modified source with Rust1.95 and wasm-bindgen0.2.129; it performs no signing, deployment, or desktop release matrix and reads no user secrets. Browser verification and upstream full-suite CI remain separate gates.

Source: https://github.com/mypurpletimemachine2023/alejandro-motion-studio

## WebGL2 baseline follow-up

The initial source revision compiled successfully in manual run 41264338108.
This follow-up adds explicit `?webgl` presentation-backend selection, matching
the supported Film/Sound web shells. `?noworkers&nogpuworkers&frameworkers=0`
can additionally disable background GPU worker paths for device diagnosis.
The default presentation backend remains unchanged. The follow-up build and
actual target-browser rendering must be verified separately; compilation alone
does not certify editor interactions, media codecs, or audio.
