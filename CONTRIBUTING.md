# Contributing to DAGR

Thank you for your interest in contributing to **DAGR**! ⚡

---

## 📜 Licensing & Contributor Terms

DAGR is licensed under the **Apache License 2.0**.

By submitting a pull request, code modification, documentation change, or other contribution to this repository, you agree to the following terms:

1. **Licensing of Contributions:** You agree that all contributions you submit are licensed under the Apache License 2.0, as specified in [`LICENSE`](./LICENSE).
2. **Originality:** You represent that you are the sole author of the contribution and have the legal right to submit it under these terms.

---

## 🛠️ Development & Testing Guidelines

### Prerequisites
- **Rust Toolchain:** Stable 2021 edition (`rustup update stable`)
- **Formatting & Linting:** `rustfmt` and `clippy`

### Running the Test Suite
Before opening a pull request, ensure all tests and clippy checks pass with zero warnings:

```bash
# 1. Format code
cargo fmt --all

# 2. Run Clippy checks
cargo clippy --workspace --all-targets -- -D warnings

# 3. Run full workspace unit & integration tests
cargo test --workspace
```

---

## 🤝 Code of Conduct

Please maintain a collaborative, respectful, and high-standard engineering environment.
