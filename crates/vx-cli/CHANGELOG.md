# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Changelog

All notable changes to this project will be documented in this file.


## [0.2.0](https://github.com/loonghao/vx/compare/vx-cli-v0.1.36...vx-cli-v0.2.0) - 2025-06-15

### Bug Fixes

- remove deprecated use command and fix binary installation
- resolve venv test failures and improve workspace publishing script
- remove useless format! usage in venv command
- improve remove command error handling in force mode
- resolve CI issues and update documentation
- implement release-please best practices for output handling

### Features

- unify all workspace versions to 0.1.36
- add version numbers to workspace dependencies and automated publishing
- implement complete venv command functionality with VenvManager integration
- implement npx and uvx support with environment isolation

### Refactor

- simplify main package by reusing vx-cli main function
