# Requirements Document

## Introduction

Soroban contracts are subject to WASM binary size limits enforced by the Stellar network. Developers deploying StellarForge contracts need visibility into the compiled size of each contract — both raw and optimized — so they can plan for size-sensitive deployments and apply optimizations before hitting network limits. This feature adds a WASM size table to the README and documents the optimization workflow using `wasm-opt`.

## Glossary

- **Build_Tool**: The `stellar contract build` CLI command that compiles Soroban contracts to WASM binaries targeting `wasm32v1-none`.
- **WASM_Binary**: The compiled `.wasm` output file produced by the Build_Tool for a given contract, located under `target/wasm32v1-none/release/`.
- **Raw_Size**: The byte size of the WASM_Binary produced directly by the Build_Tool without any post-processing.
- **Optimized_Size**: The byte size of the WASM_Binary after running `wasm-opt` with size-optimization flags.
- **wasm-opt**: A binary size and performance optimizer from the Binaryen toolchain, used to reduce WASM_Binary size after compilation.
- **README**: The top-level `README.md` file in the workspace root.
- **Size_Table**: A Markdown table in the README listing each contract alongside its Raw_Size and Optimized_Size.
- **Contract**: One of the five Soroban smart contracts in this workspace: `forge-governor`, `forge-multisig`, `forge-oracle`, `forge-stream`, `forge-vesting`.

---

## Requirements

### Requirement 1: Build All Contracts

**User Story:** As a contributor, I want all contracts built with the standard Soroban build command, so that the resulting WASM binaries reflect the actual deployment artifacts.

#### Acceptance Criteria

1. WHEN the size documentation workflow is executed, THE Build_Tool SHALL compile all five Contracts in the workspace using `stellar contract build`.
2. WHEN the Build_Tool completes successfully, THE Build_Tool SHALL produce one WASM_Binary per Contract under `target/wasm32v1-none/release/`.
3. IF the Build_Tool fails for any Contract, THEN THE Build_Tool SHALL report the failing Contract name and the error output before halting the workflow.

---

### Requirement 2: Record Raw WASM Sizes

**User Story:** As a developer, I want to know the unoptimized compiled size of each contract, so that I can understand the baseline binary footprint before any post-processing.

#### Acceptance Criteria

1. WHEN all WASM_Binaries have been produced, THE Size_Table SHALL record the Raw_Size in bytes for each Contract.
2. THE Size_Table SHALL display Raw_Size values as whole-number byte counts with no rounding or unit conversion.
3. IF a WASM_Binary for a Contract is not found at the expected path, THEN THE Size_Table SHALL mark that Contract's Raw_Size as `N/A` with a note indicating the build did not produce output.

---

### Requirement 3: Record Optimized WASM Sizes

**User Story:** As a developer, I want to know the post-optimization size of each contract, so that I can assess whether optimization brings the binary within acceptable deployment limits.

#### Acceptance Criteria

1. WHEN all WASM_Binaries have been produced, THE Size_Table SHALL record the Optimized_Size in bytes for each Contract after running `wasm-opt -Oz` on the WASM_Binary.
2. THE Size_Table SHALL display Optimized_Size values as whole-number byte counts with no rounding or unit conversion.
3. IF `wasm-opt` is not installed, THEN THE Size_Table SHALL mark each Contract's Optimized_Size as `N/A` and include a note directing the developer to install `wasm-opt`.

---

### Requirement 4: Add Size Table to README

**User Story:** As a developer evaluating StellarForge, I want a WASM size table in the README, so that I can quickly compare contract sizes without building the workspace myself.

#### Acceptance Criteria

1. THE README SHALL contain a Size_Table with the columns: `Contract`, `WASM Size (bytes)`, and `WASM Size (optimized)`.
2. THE Size_Table SHALL include one row per Contract, listing `forge-governor`, `forge-multisig`, `forge-oracle`, `forge-stream`, and `forge-vesting`.
3. THE Size_Table SHALL be placed in a dedicated section of the README titled `WASM Binary Sizes`.
4. WHEN the Size_Table is added to the README, THE README SHALL retain all existing content without modification.

---

### Requirement 5: Document wasm-opt Optimization Instructions

**User Story:** As a developer, I want step-by-step instructions for optimizing WASM binaries, so that I can reduce contract size before deployment without guessing the correct tooling or flags.

#### Acceptance Criteria

1. THE README SHALL include installation instructions for `wasm-opt` covering at least one supported package manager (e.g., `npm`, `brew`, or direct binary download).
2. THE README SHALL document the exact `wasm-opt` command used to produce the Optimized_Size values recorded in the Size_Table, including the `-Oz` flag and the input/output file paths.
3. THE README SHALL state the purpose of the `-Oz` flag in plain language within the optimization instructions section.
4. WHEN a developer follows the documented instructions, THE wasm-opt tool SHALL produce an output file that matches the Optimized_Size recorded in the Size_Table for the corresponding Contract.
