# deployment-security

## ADDED Requirements

### Requirement: The gateway image builds with the admin UI embedded
The gateway Docker image SHALL build the admin UI and make `admin-ui/dist` available in the Rust
build context before `cargo build`, so the compile-time `rust-embed` of the admin UI resolves.

#### Scenario: Image builds clean
- **WHEN** the gateway image is built
- **THEN** `admin-ui/dist` exists in the Rust build context and the gateway binary compiles.
