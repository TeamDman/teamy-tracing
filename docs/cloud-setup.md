Current status: the selected Linux environment was exercised; see
[qualification](linux-qualification.md). The proposal below is historical and does
not describe additional setup required by this checkpoint.

# Proposed Linux validation environment

Status: environment proposal only. Source publication to the public MPL-2.0
TeamDman/teamy-tracing repository is approved. No cloud environment, credential,
network rule, or installation has been created or changed for this work.

Use a dedicated x86_64 Linux workspace for the independent library if the existing
Vox environment does not provide an isolated checkout and the following inputs.
The proposal does not assume access to, mutate, or take ownership of Vox's environment.

```text
/workspace/teamy-tracing/    # approved immutable local checkpoint, imported privately
/workspace/teamy-docs/       # read-only adjacent guidance
/workspace/artifacts/        # retained validation receipts and benchmark output
```

Suggested base: Ubuntu 24.04 with Rust 1.96.0, Cargo, Clippy, a C compiler/linker,
Git and CA certificates provisioned in the image. The formatter check uses the
already-qualified nightly-2026-05-24 rustfmt; provision it before offline execution.
Pin the actual image by digest when the environment is approved. No Dockerfile
with guessed digest or package versions is supplied as a reproducibility claim.

Import the verified published library checkpoint and mount a recorded teamy-docs
revision beside it.
Prepare Cargo caches for the root lockfile and the isolated comparison lockfile
during an explicitly approved setup stage. Then run checks with `--locked --offline`.
No credentials are needed by the library or tests; private source transport belongs
to the environment's approved setup, not scripts embedded in this repository.

The Facet pin has build scripts: on supported Linux x86_64 configurations, optional
Weavy/JIT stencil work can invoke the installed rustc to emit object files. Record
the effective Weavy feature/policy and resulting graph; do not silently modify
network or security controls to make a build pass. No external clang requirement
was established for those stencil scripts; a normal native C toolchain is needed
by dependencies such as blake3. The runtime serializer is standard/non-fast.

Run `sh scripts/check.sh` from the repository. Preserve Rust versions, source hashes,
commands, exit statuses, separate normal/build/dev graph evidence, and the benchmark
NDJSON in `/workspace/artifacts`. Run bounded tests without network access. Compare
performance within that host; Windows and cloud timing differences are not serializer
speedups. Linux qualifies platform portability and build-script behavior; it does
not replace Windows writer/concurrency tests or application-level acceptance.

Before creating this environment, resolve the source import/checkpoint,
image digest/toolchain provisioning and the approved cache-fetch
stage. No new environment is required merely to keep developing and testing locally.
