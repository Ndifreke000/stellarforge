# Design Document: WASM Size Documentation

## Overview

This feature adds WASM binary size visibility to the StellarForge workspace by:

1. Building all five Soroban contracts with `stellar contract build`
2. Measuring raw WASM sizes from `target/wasm32v1-none/release/`
3. Running `wasm-opt -Oz` on each binary and measuring optimized sizes
4. Inserting a formatted size table into `README.md` under a new `WASM Binary Sizes` section
5. Documenting the `wasm-opt` installation and usage instructions in the same section

The workflow is implemented as a shell script (`scripts/update-wasm-sizes.sh`) that can be run manually by contributors and re-run whenever contracts change. The script is idempotent: it replaces the size table section on each run rather than appending.

---

## Architecture

The workflow is a single-pass shell script with four sequential stages:

```
┌─────────────────────────────────────────────────────────────┐
│  Stage 1: Build                                             │
│  stellar contract build  →  target/wasm32v1-none/release/   │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│  Stage 2: Measure Raw Sizes                                 │
│  stat / wc -c  →  raw_size[contract]                        │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│  Stage 3: Optimize & Measure                                │
│  wasm-opt -Oz  →  optimized_size[contract]                  │
└────────────────────┬────────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────────┐
│  Stage 4: Update README                                     │
│  sed / awk  →  README.md (section replaced in-place)        │
└─────────────────────────────────────────────────────────────┘
```

No external runtime dependencies beyond standard POSIX shell utilities (`stat`/`wc`, `sed`, `awk`), `stellar-cli`, and optionally `wasm-opt`.

---

## Components and Interfaces

### `scripts/update-wasm-sizes.sh`

The single entry point for the entire workflow.

**Responsibilities:**
- Invoke `stellar contract build` and capture exit codes per contract
- Read byte sizes of produced `.wasm` files
- Invoke `wasm-opt -Oz` on each binary (skip gracefully if not installed)
- Construct the Markdown size table string
- Replace the `WASM Binary Sizes` section in `README.md` using `awk`

**Interface (CLI):**
```
Usage: ./scripts/update-wasm-sizes.sh [--dry-run]

  --dry-run   Print the generated section to stdout instead of writing README.md
```

**Exit codes:**
- `0` — success
- `1` — one or more contracts failed to build

### README.md Section Markers

The script uses sentinel comments to locate and replace the section:

```markdown
<!-- WASM-SIZES-START -->
...generated content...
<!-- WASM-SIZES-END -->
```

These markers are inserted on first run and used as replacement targets on subsequent runs.

---

## Data Models

### Contract Registry

The script maintains a static ordered list of contract names and their corresponding WASM binary filenames:

```
CONTRACTS=(
  "forge-governor"
  "forge-multisig"
  "forge-oracle"
  "forge-stream"
  "forge-vesting"
)
```

WASM binary path pattern: `target/wasm32v1-none/release/<name with hyphens replaced by underscores>.wasm`

e.g. `forge-governor` → `target/wasm32v1-none/release/forge_governor.wasm`

### Size Record

For each contract, the script tracks:

| Field | Type | Description |
|---|---|---|
| `name` | string | Contract name (e.g. `forge-governor`) |
| `wasm_path` | string | Absolute path to the `.wasm` binary |
| `raw_size` | integer \| `"N/A"` | Byte count of the raw binary, or `N/A` if missing |
| `opt_size` | integer \| `"N/A"` | Byte count after `wasm-opt -Oz`, or `N/A` if wasm-opt absent |

### Generated Markdown Block

The script produces a block of this shape and splices it into README.md:

```markdown
<!-- WASM-SIZES-START -->
## ⚙️ WASM Binary Sizes

> Sizes are in bytes. Run `./scripts/update-wasm-sizes.sh` to regenerate.

| Contract | WASM Size (bytes) | WASM Size (optimized) |
| :--- | ---: | ---: |
| `forge-governor` | 12345 | 9876 |
| `forge-multisig` | 11000 | 8800 |
| `forge-oracle`   |  9500 | 7200 |
| `forge-stream`   | 13000 | 10100 |
| `forge-vesting`  | 14200 | 11300 |

### Optimizing with wasm-opt

`wasm-opt` is part of the [Binaryen](https://github.com/WebAssembly/binaryen) toolchain.
The `-Oz` flag instructs the optimizer to minimize binary size as aggressively as possible,
trading compilation time for the smallest possible output.

#### Install wasm-opt

```bash
# macOS (Homebrew)
brew install binaryen

# npm (cross-platform)
npm install -g binaryen

# Direct binary: https://github.com/WebAssembly/binaryen/releases
```

#### Run optimization

```bash
wasm-opt -Oz \
  target/wasm32v1-none/release/forge_governor.wasm \
  -o target/wasm32v1-none/release/forge_governor.wasm
```

Replace `forge_governor` with the snake_case name of the contract you want to optimize.
<!-- WASM-SIZES-END -->
```


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Build produces all five binaries

*For any* clean workspace where all five contracts compile successfully, running the script should result in exactly five `.wasm` files existing under `target/wasm32v1-none/release/` — one for each of `forge-governor`, `forge-multisig`, `forge-oracle`, `forge-stream`, and `forge-vesting`.

**Validates: Requirements 1.1, 1.2**

### Property 2: Raw size in table matches actual file size

*For any* `.wasm` binary produced by the build, the raw size value recorded in the README size table for that contract must equal the exact byte count of the file on disk (no rounding, no unit conversion).

**Validates: Requirements 2.1, 2.2**

### Property 3: Optimized size in table matches wasm-opt output

*For any* `.wasm` binary processed by `wasm-opt -Oz`, the optimized size value recorded in the README size table for that contract must equal the exact byte count of the file produced by `wasm-opt`.

**Validates: Requirements 3.1, 3.2, 5.4**

### Property 4: README section has correct structure

*For any* run of the script, the resulting `README.md` must contain a section headed `WASM Binary Sizes`, a Markdown table with columns `Contract`, `WASM Size (bytes)`, and `WASM Size (optimized)`, and exactly five data rows — one for each contract in the workspace.

**Validates: Requirements 4.1, 4.2, 4.3**

### Property 5: Script does not destroy existing README content

*For any* `README.md` with content outside the `<!-- WASM-SIZES-START -->` / `<!-- WASM-SIZES-END -->` markers, running the script must leave all content outside those markers byte-for-byte identical to the original.

**Validates: Requirements 4.4**

---

## Error Handling

| Scenario | Detection | Behavior |
|---|---|---|
| `stellar contract build` fails for a contract | Non-zero exit code from build command | Print contract name and captured stderr, exit script with code 1 |
| `.wasm` binary not found after successful build | File existence check | Record `N/A` in raw size column with inline note |
| `wasm-opt` not installed | `command -v wasm-opt` check at startup | Record `N/A` in optimized size column for all contracts; print one-time warning with install instructions |
| `wasm-opt` fails on a specific binary | Non-zero exit from wasm-opt | Record `N/A` for that contract's optimized size; print warning and continue |
| README.md not found | File existence check before write | Print error and exit with code 1 |
| README markers missing (first run) | grep for `WASM-SIZES-START` | Append the full section (including markers) to end of README |

---

## Testing Strategy

### Dual Testing Approach

Both unit tests and property-based tests are required. Unit tests cover specific examples and error conditions; property tests verify universal correctness across varied inputs.

### Unit Tests

Focus on:
- Specific example: after a mock build, the generated table string contains the expected column headers
- Specific example: the README section contains `wasm-opt` installation instructions (validates 5.1)
- Specific example: the README section contains the `-Oz` flag and a file path pattern (validates 5.2)
- Edge case: when a `.wasm` file is absent, the table row shows `N/A` for raw size (validates 2.3)
- Edge case: when `wasm-opt` is not on PATH, all optimized size cells show `N/A` (validates 3.3)
- Edge case: when a build fails, the script exits non-zero and prints the contract name (validates 1.3)

Because the script is a shell script, unit tests can be written using [bats-core](https://github.com/bats-core/bats-core) (Bash Automated Testing System), which provides `@test` blocks, `run` for capturing output, and assertion helpers.

### Property-Based Tests

Property-based testing library: **[bats-core](https://github.com/bats-core/bats-core) with randomized fixture generation** (shell-level PBT via generated inputs in a loop).

For shell scripts, true PBT libraries are uncommon; the equivalent is a parameterized test loop that generates random inputs (random file sizes, random existing README content) and asserts the properties hold across all of them. Each property test should run a minimum of **100 iterations**.

Each property test must be tagged with a comment in the format:
`# Feature: wasm-size-documentation, Property <N>: <property_text>`

**Property 1 test** — `# Feature: wasm-size-documentation, Property 1: Build produces all five binaries`
Generate a mock workspace with all five contracts. Run the build stage. Assert all five `.wasm` paths exist.

**Property 2 test** — `# Feature: wasm-size-documentation, Property 2: Raw size in table matches actual file size`
For each of 100 iterations: create a `.wasm` stub of a random byte size (1–500 000 bytes). Run the size-collection logic. Assert the table row for that contract contains exactly that byte count as a plain integer.

**Property 3 test** — `# Feature: wasm-size-documentation, Property 3: Optimized size in table matches wasm-opt output`
For each of 100 iterations: create a `.wasm` stub, run a mock `wasm-opt` that writes a file of a different known size. Assert the table row contains the post-optimization byte count.

**Property 4 test** — `# Feature: wasm-size-documentation, Property 4: README section has correct structure`
For each of 100 iterations: generate a random README with random pre-existing content. Run the script. Assert the output contains the `WASM Binary Sizes` heading, the three column headers, and exactly five data rows.

**Property 5 test** — `# Feature: wasm-size-documentation, Property 5: Script does not destroy existing README content`
For each of 100 iterations: generate a README with random content outside the markers. Run the script. Assert all content outside the markers is unchanged (compare checksums of non-marker regions).
