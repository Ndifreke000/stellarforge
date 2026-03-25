# Implementation Plan: WASM Size Documentation

## Overview

Implement a shell script (`scripts/update-wasm-sizes.sh`) that builds all five Soroban contracts, measures raw and optimized WASM sizes, and updates the README with a formatted size table. Includes bats-core unit tests and property-based tests.

## Tasks

- [ ] 1. Create the script skeleton and contract registry
  - Create `scripts/update-wasm-sizes.sh` with shebang, `set -euo pipefail`, CLI argument parsing (`--dry-run`), and the static `CONTRACTS` array with all five contract names
  - Add the helper function that converts a contract name to its snake_case WASM binary path under `target/wasm32v1-none/release/`
  - Add the `wasm-opt` availability check at startup: if not on PATH, set a flag and print a one-time warning with install instructions
  - _Requirements: 1.1, 3.3_

- [ ] 2. Implement Stage 1 — build all contracts
  - [ ] 2.1 Implement the build loop
    - Iterate over `CONTRACTS`, run `stellar contract build` for each, capture stderr, and on non-zero exit print the contract name plus captured error output then exit with code 1
    - _Requirements: 1.1, 1.2, 1.3_

  - [ ]* 2.2 Write unit tests for build failure handling
    - Test that when the build command fails for a contract, the script exits non-zero and prints the contract name
    - _Requirements: 1.3_

- [ ] 3. Implement Stage 2 — measure raw sizes
  - [ ] 3.1 Implement raw size collection
    - After each successful build, check for the `.wasm` file at the expected path; if present read its byte count with `wc -c` (or `stat`); if absent record `N/A`
    - _Requirements: 2.1, 2.2, 2.3_

  - [ ]* 3.2 Write property test for raw size accuracy (Property 2)
    - `# Feature: wasm-size-documentation, Property 2: Raw size in table matches actual file size`
    - For 100 iterations: create a `.wasm` stub of a random byte size (1–500 000 bytes), run the size-collection logic, assert the table row contains exactly that byte count as a plain integer
    - **Property 2: Raw size in table matches actual file size**
    - **Validates: Requirements 2.1, 2.2**

  - [ ]* 3.3 Write unit test for missing binary
    - Test that when a `.wasm` file is absent, the table row shows `N/A` for raw size
    - _Requirements: 2.3_

- [ ] 4. Implement Stage 3 — optimize and measure
  - [ ] 4.1 Implement wasm-opt invocation and optimized size collection
    - For each contract where `wasm-opt` is available and the raw binary exists, run `wasm-opt -Oz <input> -o <input>` (in-place), then read the resulting byte count; on failure record `N/A` for that contract and print a warning; if `wasm-opt` is absent record `N/A` for all contracts
    - _Requirements: 3.1, 3.2, 3.3_

  - [ ]* 4.2 Write property test for optimized size accuracy (Property 3)
    - `# Feature: wasm-size-documentation, Property 3: Optimized size in table matches wasm-opt output`
    - For 100 iterations: create a `.wasm` stub, run a mock `wasm-opt` that writes a file of a different known size, assert the table row contains the post-optimization byte count
    - **Property 3: Optimized size in table matches wasm-opt output**
    - **Validates: Requirements 3.1, 3.2, 5.4**

  - [ ]* 4.3 Write unit test for wasm-opt absent
    - Test that when `wasm-opt` is not on PATH, all optimized size cells show `N/A`
    - _Requirements: 3.3_

- [ ] 5. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Implement Stage 4 — generate and inject README section
  - [ ] 6.1 Build the Markdown block
    - Construct the full section string: sentinel start marker, `## ⚙️ WASM Binary Sizes` heading, regeneration note, the size table with columns `Contract`, `WASM Size (bytes)`, `WASM Size (optimized)`, the `wasm-opt` installation instructions (brew, npm, direct binary), the exact `-Oz` command example, and the sentinel end marker
    - _Requirements: 4.1, 4.2, 4.3, 5.1, 5.2, 5.3_

  - [ ] 6.2 Inject section into README.md
    - Check that `README.md` exists (exit 1 if not); if the sentinel markers are already present use `awk` to replace the region between them; if absent append the full block to the end of the file; if `--dry-run` is set print the block to stdout instead of writing
    - _Requirements: 4.3, 4.4_

  - [ ]* 6.3 Write property test for README section structure (Property 4)
    - `# Feature: wasm-size-documentation, Property 4: README section has correct structure`
    - For 100 iterations: generate a random README with random pre-existing content, run the script, assert the output contains the `WASM Binary Sizes` heading, the three column headers, and exactly five data rows
    - **Property 4: README section has correct structure**
    - **Validates: Requirements 4.1, 4.2, 4.3**

  - [ ]* 6.4 Write property test for README content preservation (Property 5)
    - `# Feature: wasm-size-documentation, Property 5: Script does not destroy existing README content`
    - For 100 iterations: generate a README with random content outside the markers, run the script, assert all content outside the markers is unchanged (compare checksums of non-marker regions)
    - **Property 5: Script does not destroy existing README content**
    - **Validates: Requirements 4.4**

  - [ ]* 6.5 Write unit tests for README section content
    - Test that the generated section contains the `wasm-opt` installation instructions (validates 5.1)
    - Test that the generated section contains the `-Oz` flag and a file path pattern (validates 5.2)
    - _Requirements: 5.1, 5.2, 5.3_

- [ ] 7. Set up bats-core test infrastructure
  - Create `tests/wasm-sizes/` directory with a `test_helper.bash` providing shared fixtures: mock `stellar` and `wasm-opt` binaries on a temp PATH, functions to create stub `.wasm` files of a given byte size, and README fixture generators
  - Create `tests/wasm-sizes/unit.bats` and `tests/wasm-sizes/property.bats` as the test entry points
  - _Requirements: 1.1, 2.1, 3.1_

- [ ]* 8. Write property test for build output completeness (Property 1)
  - `# Feature: wasm-size-documentation, Property 1: Build produces all five binaries`
  - Generate a mock workspace with all five contracts; run the build stage with a mock `stellar` that creates stub `.wasm` files; assert all five expected `.wasm` paths exist
  - **Property 1: Build produces all five binaries**
  - **Validates: Requirements 1.1, 1.2**

- [ ] 9. Wire everything together and validate end-to-end
  - [ ] 9.1 Ensure the script is executable and the `scripts/` directory exists
    - `chmod +x scripts/update-wasm-sizes.sh`, verify the script runs from the workspace root with `--dry-run` and produces the expected Markdown block
    - _Requirements: 1.1, 4.1, 4.2, 4.3, 5.1, 5.2_

  - [ ]* 9.2 Write integration tests for dry-run mode
    - Test that `--dry-run` prints the generated section to stdout and does not modify `README.md`
    - _Requirements: 4.4_

- [ ] 10. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests use randomized fixture generation in a loop (100 iterations each) as the shell-level PBT equivalent
- Unit tests use bats-core `@test` blocks with `run` and assertion helpers
- The script must be idempotent: re-running replaces the section rather than appending
