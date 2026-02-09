# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5](https://github.com/arcbox-labs/buildkit-client/compare/buildkit-client-v0.1.4...buildkit-client-v0.1.5) - 2026-02-09

### Added

- *(buildkit)* add secrets support and enhance session management
- *(build)* enhance proto file management with flexible fetching
- *(project)* add comprehensive test suite and documentation
- *(proto)* add automated proto file management and CI
- support local build by support session
- finish build public github repo

### Fixed

- *(ci)* resolve all clippy and test warnings for -D warnings
- *(ci)* resolve clippy ptr_arg warnings in diffcopy
- *(build)* remove 404 proto fetches and pin dependency refs
- *(session)* fix DiffCopy protocol to use Go FileMode format and implement followpaths filtering
- *(diffcopy)* fix diffcopy with context
- *(clippy)* fix few clippy issues

### Other

- vendor protos for offline builds
- address RustSec audit findings
- run registry-backed push tests
- apply cargo fmt to diffcopy test code
- add license file and author metadata
- apply cargo fmt and add diffcopy unit tests
- *(session)* extract DiffCopy protocol into separate module
- split filemode as a single crate
- split filemode as a single crate
- recover dockerignore integration test
- *(release)* bump version to `0.1.3`
- optimized the error handling in the BuildKit client library by migrating from anyhow to thiserror
- bump version to `0.1.2`
- add comment for empty
- *(grpc_tunnel)* document unused GetTokenAuthority function
- skip private repo text if pat_token not exist
- add bkit alias
- bump version to `0.1.1`
- *(project)* streamline project configuration and documentation
- *(build)* update proto file handling in build script
