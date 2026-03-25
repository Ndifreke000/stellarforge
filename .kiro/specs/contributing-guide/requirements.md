# Requirements Document

## Introduction

This feature adds a `CONTRIBUTING.md` file to the root of the StellarForge repository. The guide gives new contributors a clear, self-contained reference for setting up the project locally, running tests, meeting code style requirements, and submitting pull requests. Without this file, contributors must piece together information from the README or ask maintainers directly, which slows onboarding and increases noise in the review process.

## Glossary

- **Contributor**: A developer who submits code, documentation, or other changes to the repository via a pull request.
- **Maintainer**: A project owner with merge rights who reviews and approves pull requests.
- **CONTRIBUTING.md**: The markdown file at the repository root that documents the contribution workflow.
- **Workspace**: The Cargo workspace defined in the root `Cargo.toml`, containing all five contracts.
- **PR**: Pull request — a GitHub mechanism for proposing and reviewing changes.
- **CI**: Continuous integration — automated checks that run on every PR.

## Requirements

### Requirement 1: Local Setup Instructions

**User Story:** As a new contributor, I want clear setup instructions, so that I can clone and build the project without guessing at prerequisites.

#### Acceptance Criteria

1. THE CONTRIBUTING.md SHALL list all required tools and their minimum versions (Rust edition 2021, `wasm32v1-none` target, `stellar-cli` v25.2.0 or higher).
2. THE CONTRIBUTING.md SHALL provide the exact shell commands needed to install missing targets and CLI tools.
3. THE CONTRIBUTING.md SHALL provide the exact command to clone the repository and build all workspace members (`cargo build --workspace`).
4. IF a prerequisite tool is missing, THE CONTRIBUTING.md SHALL direct the contributor to the relevant installation documentation.

### Requirement 2: Running Tests

**User Story:** As a contributor, I want to know how to run the test suite, so that I can verify my changes before opening a PR.

#### Acceptance Criteria

1. THE CONTRIBUTING.md SHALL document the command to run all tests across the workspace (`cargo test --workspace`).
2. THE CONTRIBUTING.md SHALL document the command to run tests for a single contract (`cargo test -p <contract-name>`).
3. THE CONTRIBUTING.md SHALL list all five contract package names (`forge-governor`, `forge-multisig`, `forge-oracle`, `forge-stream`, `forge-vesting`) as valid values for the `-p` flag.
4. WHEN a contributor runs the documented test commands, THE CONTRIBUTING.md SHALL set the expectation that all tests must pass before a PR is submitted.

### Requirement 3: Code Style Requirements

**User Story:** As a contributor, I want to know the code style rules, so that my PR is not rejected for formatting or lint issues.

#### Acceptance Criteria

1. THE CONTRIBUTING.md SHALL document the formatting command (`cargo fmt --all`) and state that it must pass with no changes.
2. THE CONTRIBUTING.md SHALL document the lint command (`cargo clippy --all-targets -- -D warnings`) and state that it must produce zero warnings.
3. THE CONTRIBUTING.md SHALL state that new public functions and types require `///` doc comments.
4. THE CONTRIBUTING.md SHALL state that no `unsafe` code is permitted in any contract.
5. THE CONTRIBUTING.md SHALL state that no external crate dependencies beyond `soroban-sdk` are permitted without prior discussion.

### Requirement 4: Pull Request Process

**User Story:** As a contributor, I want to understand the PR and review process, so that I know what to expect after submitting my changes.

#### Acceptance Criteria

1. THE CONTRIBUTING.md SHALL describe the expected PR workflow: fork the repository, create a feature branch, commit changes, and open a PR against the `main` branch.
2. THE CONTRIBUTING.md SHALL state that a PR description must summarise what changed and why.
3. THE CONTRIBUTING.md SHALL state that all CI checks (fmt, clippy, tests) must pass before a review is requested.
4. THE CONTRIBUTING.md SHALL state that at least one Maintainer approval is required before a PR is merged.
5. WHEN a PR introduces a new contract or public API, THE CONTRIBUTING.md SHALL require that the PR includes corresponding tests covering error paths and state transitions.
6. THE CONTRIBUTING.md SHALL state that commits should be squashed or kept logically atomic before merge.

### Requirement 5: Document Discoverability

**User Story:** As a maintainer, I want the CONTRIBUTING.md to be referenced from the README, so that contributors can find it easily.

#### Acceptance Criteria

1. THE CONTRIBUTING.md SHALL be placed at the repository root so that GitHub surfaces it automatically on the repository page and when opening a PR.
2. THE README.md SHALL replace its existing inline "Contributing" section with a link to `CONTRIBUTING.md`.
