# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Version Numbering

This project uses a three-number versioning system (X.Y.Z):

X (Major): Breaking changes, major feature overhauls
Y (Minor): New features, significant improvements
Z (Patch): Bug fixes, minor improvements

Example: Version 1.2.3

1: Major version
2: Minor version
3: Patch version

When to increment:

Major (X): When making incompatible changes that might break existing functionality
Minor (Y): When adding functionality in a backward-compatible manner
Patch (Z): When making backward-compatible bug fixes

# Making Changelog Entries For New Changes in Development:

Add changes under the [Unreleased] section

Categorize them under appropriate headers:

Added for new features

Changed for changes in existing functionality

Deprecated for soon-to-be removed features

Removed for removed features

Fixed for bug fixes

Security for vulnerability fixes

Technical for technical changes/dependencies

Keep entries concise but descriptive

# When Releasing a Version:
Convert the [Unreleased] section to a version number with date (e.g., [1.0.0] - 2024-01-20)

Create a new empty [Unreleased] section at the top

# General Rules:
Newest changes always go at the top of the file

Each version should be in descending order (newest to oldest)

Group related changes under the same category

Use bullet points for each entry

# Development Workflow:
For Every Code Change:

ALWAYS add an entry to the [Unreleased] section in this changelog

Write clear, descriptive change notes

Categorize changes appropriately using the headers above

Commit changes with meaningful commit messages

For Version Releases:

Move [Unreleased] changes to a new version section with today's date

Update version number in ProjectSettings.asset (bundleVersion)

Create a git tag for the version

Create a new empty [Unreleased] section at the top

# Release Process:
When asked to make a release, follow these steps:

Review Changes:

Review all changes under [Unreleased]

Ensure all changes are properly categorized

Verify all changes are documented

Choose Version Number:

For new features: increment minor version (0.1.0 → 0.2.0)

For bug fixes: increment patch version (0.1.0 → 0.1.1)

For breaking changes: increment major version (0.1.0 → 1.0.0)

Update Files:

Move [Unreleased] changes to new version section with today's date

Update version in ProjectSettings.asset (bundleVersion)

Create new empty [Unreleased] section

Commit and Tag:

Commit all changes with message "release: Version X.Y.Z"

Create a git tag for the version (e.g., v0.2.0)

# [Unreleased]

### Added
- Thread-safe i18n implementation using Fluent
  - Support for English, German, Albanian, French, and Spanish
  - Async/await compatible with tokio
  - Language negotiation with fallback
  - Thread-safe resource management using Arc and RwLock
  - Type-safe language identifiers
  - Implemented FTL files for all supported languages
  - Common message categories: errors, navigation, user messages, tenant messages, actions, confirmations, success messages, form labels, and validation messages
  - Language detection middleware with support for:
    - URL query parameters (?lang=de)
    - Accept-Language header
    - Fallback to default language
    - Extension trait for easy access in request handlers
  - Concurrent memoization for improved performance
  - System status messages in all supported languages

### Changed
- Improved test assertion readability in error handling tests
- Enhanced thread safety in i18n implementation using intl-memoizer
- Enhanced health check system
  - Added i18n support for health status messages
  - Improved response structure with status messages
  - Unified health and readiness check response format
  - Integrated health routes with i18n middleware
  - Added timestamp to health responses
- Optimized language middleware implementation
  - Improved error handling in language negotiation
  - Enhanced type safety in middleware service implementation
  - Streamlined extension handling in test service
  - Better test assertions using assert_eq
- Removed ARM64/AArch64 architecture support from CI pipelines
  - Updated Docker multi-architecture builds to only target AMD64 and PPC64LE
  - Modified cross-compilation settings in GitHub Actions and GitLab CI
  - Adjusted QEMU emulation setup for remaining architectures
  - Updated platform-specific security scans and tests
  - Streamlined CI/CD pipeline configurations

### Fixed
- Fixed Clippy warnings for unnecessary borrows and unwraps
- Optimized Docker build to use distroless/cc for minimal runtime dependencies
- Suppressed dead code warnings for base components that will be used in future implementations
- Resolved dead code warnings for ErrorContext fields
- Removed unused imports and dead code
- Optimized middleware response handling
- Fixed test assertions for health checks
- Improved type safety in language middleware
- Streamlined error propagation in service implementations

### Technical
- Defined Minimum Supported Rust Version (MSRV) as 1.75 in Cargo.toml
- Added GitLab CI/CD pipeline configuration
  - Equivalent functionality to GitHub Actions workflows
  - Integrated security scanning with GitLab templates
  - Enhanced container scanning with Trivy
  - Automated test suite with coverage reporting
  - Cross-platform testing support
  - SBOM generation and validation
  - Image signing with Cosign
  - Dependency auditing and license compliance checks

### Added
- Comprehensive GitHub Actions test workflow
  - Automated test suite with unit and integration tests
  - Cross-platform testing (Ubuntu, macOS, Windows)
  - Code coverage reporting with cargo-tarpaulin
  - MSRV (Minimum Supported Rust Version) validation
  - Clippy and rustfmt checks
  - Security audits with cargo-audit
  - Documentation tests and builds
  - Binary size monitoring
  - Performance benchmarking with criterion
- Core dependencies for multi-tenant system
  - Axum web framework with full features
  - Sea-ORM for database operations
  - GraphQL support with async-graphql
  - Authentication and authorization libraries
  - Logging and metrics infrastructure
  - Error handling utilities
  - Development and testing utilities
- Comprehensive test suite implementation
  - API endpoint tests for health and readiness checks
  - Domain logic tests for tenant management
  - Error handling and context validation tests
  - Infrastructure tests for database and cache connections
  - Logging system tests with context tracking
  - Total test coverage: 24 unit tests across all components

### Security
- Enhanced Docker security pipeline
  - Added explicit permissions configuration for GitHub Actions
  - Improved SARIF report handling for security scans
  - Added write permissions for security events and pull requests
  - Enhanced access control for GitHub token usage
  - Integrated Docker Scout scanning with proper authentication
  - Added SARIF output for Docker Scout results
  - Improved security scanning results visualization in GitHub
  - Added automated Docker Hub authentication for security scans
