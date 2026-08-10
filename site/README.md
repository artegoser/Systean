# Systean site

SvelteKit frontend for the Systean Rust engine.

The frontend does not read `language/*.toml` directly. `pnpm dev`, `pnpm check`, and `pnpm build` first compile `crates/systean-wasm` through `scripts/build-wasm.sh`, then the browser initializes the generated WebAssembly module through `$lib/engine`.

```bash
pnpm check
pnpm build
pnpm dev
```

Generated WASM bindings live in `src/lib/wasm/pkg/` and are intentionally ignored by Git.

pnpm build-script approvals are committed in `pnpm-workspace.yaml`. The site explicitly allows `esbuild` and `@tailwindcss/oxide`, so a fresh checkout does not depend on machine-local `pnpm approve-builds` state.

The site declares Node.js `24.x` in `package.json`, matching the current Vercel runtime requirement rather than relying on a deprecated Node 20 project setting.
