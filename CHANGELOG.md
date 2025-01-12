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
  - Concurrent memoization for improved performance
  - System status messages in all supported languages
- Pre-commit hook for detecting hardcoded strings
- Clippy rules for hardcoded string detection
- Translation completeness checker script
- Added walkdir dependency for translation checking
- Multi-environment Docker setup with separate Dockerfiles for development, production, and testing
- Development environment with hot-reloading and development tools
- Test environment with testing tools and coverage support
- Production environment with security scanning and SBOM generation
- Added Codecov integration for code coverage reporting
- Configuration system with environment-specific settings
- Support for different environment configurations (dev/prod/test)
- Config service for centralized configuration management
- Configuration system implementation
  - Environment-specific configuration files (dev/prod/test)
  - Centralized settings management with type-safe access
  - Fail-fast error handling for configuration issues
  - Environment variable overrides with APP__ prefix
  - Default values for all configuration options
- Enhanced configuration system with template handling
  - Automatic creation of environment-specific config files from templates
  - Separate handling for dev, prod, and test environments
  - Robust fallback mechanism for missing configurations
  - Comprehensive test coverage for all configuration scenarios
- Tenant middleware for multi-tenant support
  - Tenant detection via HTTP header (X-Tenant-ID)
  - Tenant detection via domain name
  - Automatic tenant context injection
  - Comprehensive test coverage
  - Error handling for invalid/inactive tenants

### Changed
- Enhanced Docker configuration management
  - Standardized environment variable usage across all Docker configurations
  - Added DEFAULT_LANGUAGE support in all compose files
  - Improved port configuration handling in Dockerfiles
  - Enhanced Redis configuration with proper URL and binding
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
- Enhanced string literal handling in development workflow
- Improved i18n validation process
- Modified translation check script to exclude src/common/i18n.rs from verification
- Updated hardcoded string check script to exclude i18n configuration file
- Fixed regex patterns in hardcoded string detection to prevent invalid operator errors
- Improved i18n test infrastructure
  - Removed file system operations from tests
  - Introduced in-memory test resource provider
  - Enhanced test isolation and reliability
  - Simplified test setup and teardown
- Improved configuration system robustness
  - Enhanced error handling for missing templates
  - Better logging with appropriate warning levels
  - Cleaner separation of environment-specific settings
  - Type-safe configuration value handling
- Enhanced configuration system
  - Moved configuration files to /config directory
  - Improved default value handling to only apply when values are missing
  - Standardized configuration file naming (config.{env}.toml)
  - Separated test configuration handling to use fixed file
  - Enhanced configuration loading priority (env vars > config file > defaults)
  - Improved error messages for configuration loading failures
- Completely redesigned configuration system
  - Separated test and production configuration handling using conditional compilation
  - Simplified configuration loading logic
- Completely redesigned configuration testing system
  - Introduced mock file system for configuration tests
  - Separated test and production file operations
  - Added serial test execution to prevent environment variable conflicts
  - Improved test isolation and reliability
  - Eliminated flaky tests through proper mocking
- Updated dependencies
  - sea-orm from 1.1.3 to 1.1.4
  - config from 0.15.4 to 0.15.5
  - Added serial_test 3.0.0 for improved test isolation
- Enhanced Clippy configuration
  - Added comprehensive type-safe configuration for tracing macros
  - Configured proper ignore rules for interior mutability
  - Adjusted complexity thresholds for better maintainability
  - Enhanced documentation requirements and valid identifiers
  - Improved error handling safety rules

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
- Fixed container structure test configuration to use correct schema for environment variables
- Added required dependencies for PPC64LE cross-compilation
  - Installed bindgen-cli and AWS-LC system dependencies
  - Set up proper environment variables for cross-compilation
  - Added proper sysroot configuration for PPC64LE builds
- Fixed Docker multi-architecture build process
  - Modified build strategy to handle platform-specific builds separately
  - Updated image tagging scheme for better architecture identification
  - Adjusted security scanning and testing for platform-specific images
  - Fixed manifest list handling in GitHub Actions
- Removed redundant doc test step from CI configurations
  - Documentation tests are already included in the main test suite
  - Simplified test steps in GitHub Actions and GitLab CI
- Enhanced PPC64LE cross-compilation support
  - Added libc6-dev-ppc64el-cross for required system headers
  - Improved bindgen configuration with correct target triple
  - Set up proper pkg-config environment for cross-compilation
  - Added RUST_TARGET_PATH configuration
  - Configured correct linker for PPC64LE target
  - Added ELFv2 ABI support for PPC64LE builds
  - Disabled AWS-LC ASM optimizations for PPC64LE
  - Added proper target CPU configuration
- Removed unused LanguageExt trait and its implementation in favor of direct Extension extraction
- Marked unused but important functions with #[allow(dead_code)]
  - setup_request_span in logging module
  - validate method in Tenant implementation
  - TenantContext and its methods
- Removed unused imports across the codebase
  - Removed http::Request from health module main imports
  - Removed LanguageExt trait import
  - Removed TcpListener from health tests
- Added #[allow(clippy::disallowed_methods)] attributes for tracing macros
  - Applied to logging functions using tracing macros
  - Applied to main function for server startup logging
  - Applied to database connection error logging
  - Maintains code quality while allowing necessary logging functionality
- Added missing build dependencies for aws-lc-sys
  - Added cmake, clang, and LLVM development packages
  - Added cross-compilation support for PPC64LE
  - Fixed build environment variables for cross-compilation
  - Optimized multi-architecture build process
- Streamlined CI/CD pipelines
  - Simplified GitHub Actions workflow
  - Enhanced GitLab CI configuration
  - Improved build caching strategy
  - Optimized security scanning process
- Prevent deletion of production locales directory during test execution by using a separate test-specific directory
- Resolved Clippy warnings in configuration system
  - Added appropriate allow attributes for tracing macros
  - Improved test setup with proper cleanup
  - Enhanced error handling in configuration loading
  - Fixed unwrap usage in test environment
- Improved error handling in tenant middleware
  - Replaced unwrap() calls with proper error handling
  - Added centralized error response creation
  - Enhanced code maintainability and safety
- Completely redesigned configuration testing system
  - Introduced mock file system for configuration tests
  - Separated test and production file operations
  - Added serial test execution to prevent environment variable conflicts
  - Improved test isolation and reliability
  - Eliminated flaky tests through proper mocking
- Improved configuration system robustness
  - Separated test and production configuration handling using conditional compilation
  - Enhanced error handling for missing templates
  - Better logging with appropriate warning levels
  - Cleaner separation of environment-specific settings
  - Type-safe configuration value handling
- Resolved Clippy configuration issues
  - Fixed duplicate configuration entries
  - Corrected ignore-interior-mutability configuration
  - Added proper type-safe configuration for tracing macros
  - Enhanced error handling rules for unwrap and expect

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

## [0.1.0] - 2024-01-01

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
