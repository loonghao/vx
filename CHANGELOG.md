# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.27](https://github.com/loonghao/vx/compare/vx-v0.5.26...vx-v0.5.27) (2025-12-28)


### Features

* **extension:** complete phase 2 with error handling and 81 tests ([48cfbcd](https://github.com/loonghao/vx/commit/48cfbcdd8fca06fa2dbc622357b4eb7d2d4e44c6))
* **vx-project-analyzer:** implement RFC 0003 project analyzer ([efc2ca0](https://github.com/loonghao/vx/commit/efc2ca0b0b3235151e41c7b2b3ea1a77226765e3))


### Bug Fixes

* **vx-extension:** use std::io::Error::other for clippy ([9363106](https://github.com/loonghao/vx/commit/9363106bf6e8c1f3e690b6b179dbb54c187bf0d0))

## [0.5.26](https://github.com/loonghao/vx/compare/vx-v0.5.25...vx-v0.5.26) (2025-12-28)


### Features

* **extension:** implement vx extension system ([a23dccb](https://github.com/loonghao/vx/commit/a23dccbfd5d8e00d9eb8abdc6d73dd681bc18656))


### Bug Fixes

* add serial_test to prevent env var race conditions in tests ([49eb42b](https://github.com/loonghao/vx/commit/49eb42b231c6793cb2a44f6f495bf96267eacf38))
* **ci:** remove tests for non-existent commands and ignore RFC dead links ([da29e22](https://github.com/loonghao/vx/commit/da29e2261941f2b6d712958e083d9263fcf4a6f5))
* increase retry count and delay for network resilience ([1687749](https://github.com/loonghao/vx/commit/1687749e5075059a06dad02a720e18de23b64929))
* remove dead links and format code ([7a56b7e](https://github.com/loonghao/vx/commit/7a56b7e6b4264c626b5caca4f697950d6e8f4db0))
* resolve clippy warnings and doctest error ([7ebdf3e](https://github.com/loonghao/vx/commit/7ebdf3e729850015bc0fb7488bbab5ed42c5cc0c))
* resolve compilation errors and update documentation ([5cf23f2](https://github.com/loonghao/vx/commit/5cf23f24ee8c5c959b74cb61b5c138996d0c306f))
* resolve test failures and clippy warnings ([9b09ade](https://github.com/loonghao/vx/commit/9b09adee28147f3684aaf16f2a18e22a7549f707))
* **tests:** prevent parent directory config search in e2e tests ([84086af](https://github.com/loonghao/vx/commit/84086af1e94b476b557e0b566d169de5fd12e27b))

## [0.5.25](https://github.com/loonghao/vx/compare/vx-v0.5.24...vx-v0.5.25) (2025-12-27)


### Bug Fixes

* **ci:** fix Homebrew and Scoop publishing issues ([ab81dd6](https://github.com/loonghao/vx/commit/ab81dd6518e55e3ac979c5812175d9a46c17f475))

## [0.5.24](https://github.com/loonghao/vx/compare/vx-v0.5.23...vx-v0.5.24) (2025-12-27)


### Bug Fixes

* **ci:** only use releases with available assets ([e10afcd](https://github.com/loonghao/vx/commit/e10afcdb0fe89bbc23789a4c383322a464c49ab2))
* **ci:** replace Ash258/Scoop-GithubActions with custom script ([eb960c2](https://github.com/loonghao/vx/commit/eb960c23b966527135a0d8a277bf13bd4ee33325))
* **ci:** replace Justintime50/homebrew-releaser with custom script ([7aa8441](https://github.com/loonghao/vx/commit/7aa844135d657b6e02ca558e6de6be65a653d9e8))

## [0.5.23](https://github.com/loonghao/vx/compare/vx-v0.5.22...vx-v0.5.23) (2025-12-27)


### Documentation

* update Homebrew installation instructions ([1ff78b5](https://github.com/loonghao/vx/commit/1ff78b50d291880d3e7bfdac9e627f4a2869ac6e))

## [0.5.22](https://github.com/loonghao/vx/compare/vx-v0.5.21...vx-v0.5.22) (2025-12-27)


### Features

* **provider:** add release-please provider ([ee4c934](https://github.com/loonghao/vx/commit/ee4c93405736f4272e61c8ae5ce4831d8c65fbad))


### Documentation

* add plugin command documentation and update snapshots ([4af6248](https://github.com/loonghao/vx/commit/4af6248fbcb507aee2901cd30fdb1014144f14ca))

## [0.5.21](https://github.com/loonghao/vx/compare/vx-v0.5.20...vx-v0.5.21) (2025-12-27)


### Features

* vx.toml v2 configuration enhancement ([e55f621](https://github.com/loonghao/vx/commit/e55f621f3a924daaf64f3cb0bdef1a68c6e22e80))


### Bug Fixes

* **cli:** correct bool flag defaults for parallel and backup ([7f63f88](https://github.com/loonghao/vx/commit/7f63f88633dcf0fbedbf8b5dd6606858dc4e4d39))
* **clippy:** move generate_dockerfile before tests and remove duplicate if branches ([af2962b](https://github.com/loonghao/vx/commit/af2962be1accef980b90d6ef2fce28983a401528))
* **executor:** add platform check at execute() entry point ([4a20a18](https://github.com/loonghao/vx/commit/4a20a18fbeedac2c12cba33d1b0067c9fa9dc778))
* **executor:** check platform support before installing runtime ([96e0495](https://github.com/loonghao/vx/commit/96e049517960863956aafb4ebc043c945c7b0c02))
* pin backon to 1.4.0 for MSRV 1.83 compatibility ([74d65f5](https://github.com/loonghao/vx/commit/74d65f58e6635ba2f0e05012d6de2c93c811c92d))
* resolve clippy dead_code and should_implement_trait warnings ([26e3221](https://github.com/loonghao/vx/commit/26e322135f9e5584ed9cd31293e82126b74da6f1))
* resolve clippy warnings (redundant closure, single match, collapsible if, assertions) ([46e3e1f](https://github.com/loonghao/vx/commit/46e3e1f6cdc2e77971d11f04be7fc414db358a31))
* resolve clippy warnings and use PowerShell syntax for Windows tests ([2e84cc8](https://github.com/loonghao/vx/commit/2e84cc8060bcd9c9fcac55f35a18ece14d2da716))
* **spack:** restrict to Unix platforms only (Linux/macOS) ([ce27acf](https://github.com/loonghao/vx/commit/ce27acf48fe9014c5a7494b9e17d434e2d016f4b))
* **tests:** resolve unused imports, variables and private module access ([bbf5933](https://github.com/loonghao/vx/commit/bbf593361e740dc18aa8254d333674b9d75d3699))
* update help.md snapshot with new subcommands ([8ff8d56](https://github.com/loonghao/vx/commit/8ff8d569f7937b08f516c2314c06bce3754058a8))
* use valid TOML key name in script validation test ([3c1fdea](https://github.com/loonghao/vx/commit/3c1fdeac05bebe6177a19d1345623d20541fa947))


### Code Refactoring

* **runtime:** add check_platform_support() helper and platform utils ([75c61e3](https://github.com/loonghao/vx/commit/75c61e341e5c7e700167accc0a2d4ee054c0b3a9))
* split types.rs into modular types/ directory ([ea1a506](https://github.com/loonghao/vx/commit/ea1a506d5f925a84707153ca244871e134700bbc))

## [0.5.20](https://github.com/loonghao/vx/compare/vx-v0.5.19...vx-v0.5.20) (2025-12-26)


### Bug Fixes

* make test_generate_script_backslash_in_value Windows-only ([1ccc70e](https://github.com/loonghao/vx/commit/1ccc70e60154a18625d61c295f243f9d77db3844))
* use rez-style dynamic script generation for vx run ([135eb6d](https://github.com/loonghao/vx/commit/135eb6dda63329ba525267ded9a233d196d5e311))


### Code Refactoring

* use modern shells for script execution ([f667588](https://github.com/loonghao/vx/commit/f6675882e96eb763583fec737b4c9f5c158c9455))

## [0.5.19](https://github.com/loonghao/vx/compare/vx-v0.5.18...vx-v0.5.19) (2025-12-26)


### Features

* add vx env export command ([c23168c](https://github.com/loonghao/vx/commit/c23168ca143d3644ef0d0cb10d1241af131227e4))
* **installer:** add automatic retry with exponential backoff for downloads ([cea2949](https://github.com/loonghao/vx/commit/cea2949b64384e07f19c1239614fb93a1a5c458a))


### Bug Fixes

* isolate e2e env tests with separate workdir ([e1eb7a9](https://github.com/loonghao/vx/commit/e1eb7a92f9b8d8e85a3213e249135cae659e9182))
* rename from_str to parse to avoid clippy warning ([4ca6e00](https://github.com/loonghao/vx/commit/4ca6e00818f1dac9de6e374b6c22367b41e7bbee))


### Code Refactoring

* **installer:** use backon for retry logic with exponential backoff ([3460c33](https://github.com/loonghao/vx/commit/3460c3304742ac5b05b3de2ab1b17e15e50d022e))
* merge vx env export into vx dev --export ([a2c8422](https://github.com/loonghao/vx/commit/a2c8422a38270fff2b5e1cc7d1e62e7ed491a6a5))


### Documentation

* add vx env export documentation ([383df76](https://github.com/loonghao/vx/commit/383df76737e9eb24ab11eeef532c31a616f934a5))

## [0.5.18](https://github.com/loonghao/vx/compare/vx-v0.5.17...vx-v0.5.18) (2025-12-26)


### Bug Fixes

* add vx managed tools bin directory to PATH in GitHub Action ([067663d](https://github.com/loonghao/vx/commit/067663dfe26024f253699e95ff52e6598b5a97d2))

## [0.5.17](https://github.com/loonghao/vx/compare/vx-v0.5.16...vx-v0.5.17) (2025-12-26)


### Bug Fixes

* improve install scripts and GitHub Action reliability ([e87b9ac](https://github.com/loonghao/vx/commit/e87b9ac9a8c484acb7c2523828f464d7f63fe58f))

## [0.5.16](https://github.com/loonghao/vx/compare/vx-v0.5.15...vx-v0.5.16) (2025-12-26)


### Features

* add P0 cloud and container providers (Docker, AWS CLI, Azure CLI, gcloud) ([84f4940](https://github.com/loonghao/vx/commit/84f494036d5a8519b31f12338d7d89b0f808ccc4))
* add P1 providers (ninja, cmake, protoc, task, pre-commit) ([0ccf7eb](https://github.com/loonghao/vx/commit/0ccf7eb2b87ce3142e3fdf95bbafb736904945c7))
* add spack provider and unit tests for multiple providers ([03af547](https://github.com/loonghao/vx/commit/03af547cdda6fba70bb47ffdefe6e362bcccd53a))
* **list:** add --all flag to show unsupported platform tools ([3d1ad50](https://github.com/loonghao/vx/commit/3d1ad506ce249bfcc9d662af83b4cc6eef2cf2b5))
* **ollama:** add ollama provider for local LLM management ([ff5ed86](https://github.com/loonghao/vx/commit/ff5ed8637989066e9e971a6b88e4e454130f26cf))
* **provider:** add Chocolatey package manager provider ([4396fa2](https://github.com/loonghao/vx/commit/4396fa25b69fd6212d4167f4f12cde512c0bcbc4))
* **provider:** add git provider for version management ([f7881b4](https://github.com/loonghao/vx/commit/f7881b4d4afa2b104b8cfe481be9f4ac0c7b8831))


### Bug Fixes

* add rez-release to provider supports ([e6934ec](https://github.com/loonghao/vx/commit/e6934ec8e0d9adf6d81496b204df6cc7bd1cd971))
* address clippy warnings in dev_environment_tests ([68533b6](https://github.com/loonghao/vx/commit/68533b64c8d28e5f04478dd6caf53a9fcd66b69c))
* **docker:** update Docker Hub repository to longhal/vx ([f2212c6](https://github.com/loonghao/vx/commit/f2212c650ab29aafad989ff44f50726300fcf8c3))
* **docker:** use short version format for Docker tags ([650361f](https://github.com/loonghao/vx/commit/650361fee3a297583ed4b906ba3766ab2d1b3a53))
* **docs:** remove dead links in Chinese documentation ([7e33316](https://github.com/loonghao/vx/commit/7e33316addb4539eebc4341695c5082556ca7194))
* **list:** fix compilation errors for platform support check ([3bf0638](https://github.com/loonghao/vx/commit/3bf06389df595ee8f5d8f3745b9cbdeb8edd0b83))
* **list:** use helper function for platform support check ([2ea3e4c](https://github.com/loonghao/vx/commit/2ea3e4cd49554c2791fc4f92138042184e0b7678))
* resolve compilation errors in cloud providers ([c98b6be](https://github.com/loonghao/vx/commit/c98b6be0f10329f093af2fbf35f7806f97abd2ee))
* update release-please PR title pattern to include v prefix ([e757238](https://github.com/loonghao/vx/commit/e75723829f60fb92045c6241c993df8aa06d1fac))
* use array instead of vec for path_entries ([fba551d](https://github.com/loonghao/vx/commit/fba551d3b55aa340477dcb074458ab11873ff62e))
* use as_ref for trait method call ([8d4a53e](https://github.com/loonghao/vx/commit/8d4a53eb45a0ecdeb434cb8274e4581827f1bcda))
* use method reference instead of redundant closure ([d2d6b26](https://github.com/loonghao/vx/commit/d2d6b26ce9b1c2d7c8c73914977dd50c02febce4))


### Documentation

* add documentation for new tool providers ([07f058c](https://github.com/loonghao/vx/commit/07f058c2d16d575046ade915987474fbe8426bd7))
* add GitHub Action guide and fix CI issues ([bd49925](https://github.com/loonghao/vx/commit/bd499259ad3638ad937327e011d8843e60319003))
* **i18n:** add Chinese documentation ([8ba8e7e](https://github.com/loonghao/vx/commit/8ba8e7e1b0034c48a6d2f17bb0f6a0e67eebc052))
* update GitHub Action usage to use specific version tag ([aecef6f](https://github.com/loonghao/vx/commit/aecef6f7b2de52c023bf685f0ffe8aad506ce4bd))

## [0.5.15](https://github.com/loonghao/vx/compare/vx-v0.5.14...vx-v0.5.15) (2025-12-24)


### Bug Fixes

* **docs:** add .nojekyll file to fix GitHub Pages 404 errors ([4ddab93](https://github.com/loonghao/vx/commit/4ddab934751cdb000212183c63ca865bfb8a0ba6))

## [0.5.14](https://github.com/loonghao/vx/compare/vx-v0.5.13...vx-v0.5.14) (2025-12-24)


### Features

* add GitHub Action for easy CI/CD integration ([36f1061](https://github.com/loonghao/vx/commit/36f1061efd588ef5a3fc857af072ee2b53191006))


### Bug Fixes

* escape template syntax in docs for VitePress compatibility ([ebd54ef](https://github.com/loonghao/vx/commit/ebd54ef24882f12448506f3d5a9b68d66efb8d1a))
* use authenticated download in action and install script ([0ac69ec](https://github.com/loonghao/vx/commit/0ac69ecfc4d4b1966e9e62e59c8557dd2ad76644))


### Documentation

* update documentation with complete tool list and provider development guide ([55d7798](https://github.com/loonghao/vx/commit/55d779826faa562a2763c8ff835d54ae87785d0e))

## [0.5.13](https://github.com/loonghao/vx/compare/vx-v0.5.12...vx-v0.5.13) (2025-12-23)


### Features

* **list:** show bundled tools as installed when parent is installed ([78895bf](https://github.com/loonghao/vx/commit/78895bf8ee49657d0a3d470e06e236163f13ee35))


### Bug Fixes

* **java:** update test to expect Ecosystem::Custom instead of Unknown ([0405b6e](https://github.com/loonghao/vx/commit/0405b6e94d6fbf9c5544ff127dd05273b281fbf4))
* move clippy allow attribute to struct level for Rust 1.92 compatibility ([a59afae](https://github.com/loonghao/vx/commit/a59afaea82cd32a51246733aa9f9e37d37801fa2))
* **pnpm:** rename downloaded file to standard name in post_install ([eb4ae55](https://github.com/loonghao/vx/commit/eb4ae5589161b94ef38658bd119cab0f86772024))
* **pnpm:** use Platform parameter for executable path and download URL ([dea9e19](https://github.com/loonghao/vx/commit/dea9e190383c0be8e35a8e19d75a25fa22292a82))
* resolve CI issues and clean up old docs ([5b8c2df](https://github.com/loonghao/vx/commit/5b8c2df87155e47147c09ab39e5e9d5941380e05))
* resolve zig URL format and docs dead links ([fe7677e](https://github.com/loonghao/vx/commit/fe7677e6ee1ec56d3d1b85506b3db6926adedf22))


### Code Refactoring

* consolidate platform utilities and improve tests ([b7e783d](https://github.com/loonghao/vx/commit/b7e783dc612b8c5a9c69cbc2272f1ef2531608e5))

## [0.5.12](https://github.com/loonghao/vx/compare/vx-v0.5.11...vx-v0.5.12) (2025-12-22)

### Features

* add new providers (deno, helm, java, kubectl, rcedit, terraform, zig) ([176984b](https://github.com/loonghao/vx/commit/176984bf3f496dc9d5a7a7c49b9567587f5a7d77))

### Bug Fixes

* conditionally import BenchmarkId for cdn-acceleration feature ([61666af](https://github.com/loonghao/vx/commit/61666af67d37a40ac04af548d2be631b32b88ddb))
* correct pnpm executable path to bin/pnpm ([8883d1e](https://github.com/loonghao/vx/commit/8883d1ebf528568b82002bd4c2fe0a5e22a22072))
* update MSRV to 1.83.0 and modernize progress bars ([1ff77bf](https://github.com/loonghao/vx/commit/1ff77bf2c1c9fd4495a00d36f05a171bb0d1630a))
* use actual downloaded filename for pnpm executable path ([6b4e5fa](https://github.com/loonghao/vx/commit/6b4e5fa50715552ede91c03d3a277bd342899cfe))
* use Ecosystem::Unknown instead of Ecosystem::Other in provider tests ([ae9f6b6](https://github.com/loonghao/vx/commit/ae9f6b645fed71851fb62d6f64c8e9c198874159))

### Documentation

* fix dead links ([4870612](https://github.com/loonghao/vx/commit/4870612c6fe0f453c886431d498d4a4aa1792ae3))

## [0.5.11](https://github.com/loonghao/vx/compare/vx-v0.5.10...vx-v0.5.11) (2025-12-21)

### Features

* **cli:** add project development environment commands ([0ad51e4](https://github.com/loonghao/vx/commit/0ad51e4d119b27df2ae673a440544373917f5674))

## [0.5.10](https://github.com/loonghao/vx/compare/vx-v0.5.9...vx-v0.5.10) (2025-12-20)

### Bug Fixes

* correct CDN URL version extraction for repo@tag format ([8082580](https://github.com/loonghao/vx/commit/808258039cca0429894bac951f8f4cd28e5b4ecf))
* **deps:** update rust crate zip to v7 ([b0d135b](https://github.com/loonghao/vx/commit/b0d135b0f59cdb520c864a22a67029666e6dcec5))
* improve self-update tag format handling ([23eeb89](https://github.com/loonghao/vx/commit/23eeb89bdeedae55ff6e0812cd656ed43bac183e))

## [0.5.9](https://github.com/loonghao/vx/compare/vx-v0.5.8...vx-v0.5.9) (2025-12-19)

### Features

* add just command runner provider ([2c9dc21](https://github.com/loonghao/vx/commit/2c9dc21838acf9b326b05b783708996a282c746b))
* add rez provider ([4d342aa](https://github.com/loonghao/vx/commit/4d342aa7ae25e5db93b094ec1f486f64696cfc30))
* **vite:** add Vite provider ([c7c37bb](https://github.com/loonghao/vx/commit/c7c37bbf5f9ae5abbe7f92dcaa8ed4fdedb76c6a))

### Bug Fixes

* make go tests more robust for CI ([1708bdc](https://github.com/loonghao/vx/commit/1708bdcab44c39b4a8adbba4a964cf9ec02bcc6e))
* use derive macro for InstallMethod Default impl ([d7178b5](https://github.com/loonghao/vx/commit/d7178b5317da4c1b941035ae5003a508313813d4))

## [0.5.8](https://github.com/loonghao/vx/compare/vx-v0.5.7...vx-v0.5.8) (2025-12-18)

### Features

* **vscode:** add VSCode provider ([08d2178](https://github.com/loonghao/vx/commit/08d21781fb9d1c2b216f7426c26df17ffc1e03cc))

### Bug Fixes

* simplify release asset naming and fix installer download URLs ([5a079f2](https://github.com/loonghao/vx/commit/5a079f2a3d7e2aa35694ef4448ec82293d75c5f4))

## [0.5.7](https://github.com/loonghao/vx/compare/vx-v0.5.6...vx-v0.5.7) (2025-12-18)

### Bug Fixes

* **ci:** pin tracing-indicatif to 0.3.9 and remove RUST_BACKTRACE=1 ([6d4dc61](https://github.com/loonghao/vx/commit/6d4dc61f9abe43016449d096c8e761d789cd0373))

## [0.5.6](https://github.com/loonghao/vx/compare/vx-v0.5.5...vx-v0.5.6) (2025-12-18)

### Bug Fixes

* **ci:** use softprops/action-gh-release to upload release artifacts ([552b785](https://github.com/loonghao/vx/commit/552b7851ea6f18a02964f9d983ff3c18039d561c))

## [0.5.5](https://github.com/loonghao/vx/compare/vx-v0.5.4...vx-v0.5.5) (2025-12-17)

### Bug Fixes

* **ci:** remove --locked flag from CI builds to handle crates.io index updates ([3ed7974](https://github.com/loonghao/vx/commit/3ed79746eebcacedcb19a4c9358240abec5bb133))

## [0.5.4](https://github.com/loonghao/vx/compare/vx-v0.5.3...vx-v0.5.4) (2025-12-17)

### Bug Fixes

* **ci:** fix release workflow to properly build and upload artifacts ([9cd4bfb](https://github.com/loonghao/vx/commit/9cd4bfbdf5efc8a4af21187302c69edb09af70fb))

## [0.5.3](https://github.com/loonghao/vx/compare/vx-v0.5.2...vx-v0.5.3) (2025-12-17)

### Bug Fixes

* **ci:** escape changelog content with toJSON() in release workflow ([a21aa29](https://github.com/loonghao/vx/commit/a21aa295e58b7532707d63c0b7fd8af2ea8c5d14))

## [0.5.2](https://github.com/loonghao/vx/compare/vx-v0.5.1...vx-v0.5.2) (2025-12-16)

### Bug Fixes

* **ci:** remove --locked flag from release build to handle crates.io index updates ([fc4dea7](https://github.com/loonghao/vx/commit/fc4dea7e0f597278025735b6377cda947b2253dd))

## [0.5.1](https://github.com/loonghao/vx/compare/vx-v0.5.0...vx-v0.5.1) (2025-12-16)

### Bug Fixes

* **ci:** correctly extract version from tag name ([bb3f679](https://github.com/loonghao/vx/commit/bb3f67974a312d5c12ba2f36b7ad8c3a1a4b890c))
* **ci:** remove custom pull-request-title-pattern ([8f8c23f](https://github.com/loonghao/vx/commit/8f8c23f061b2cd901ba1c0d02ee643bf6fe7db3a))
* replace deprecated criterion::black_box with std::hint::black_box ([269888c](https://github.com/loonghao/vx/commit/269888c408f4b9c4cdea7dc5f65564e9eb5f0d7f))
* use workspace dependencies for internal crates ([8791c47](https://github.com/loonghao/vx/commit/8791c47005e26fc3d6d627ae242954bd9f66aeaf))

## [0.5.0](https://github.com/loonghao/vx/compare/vx-v0.4.1...vx-v0.5.0) (2025-12-16)

### ⚠ BREAKING CHANGES

* vx-shim is no longer supported, use shimexe-core instead
* Legacy commands have been removed. Use new standardized commands instead.
* Complete rewrite of release system
* Complete rewrite using GoReleaser's prebuilt builder

### Features

* add --debug flag for enabling debug logging ([40c65c3](https://github.com/loonghao/vx/commit/40c65c3436cedead6047709fddbff07d3b4f898b))
* add comprehensive environment variables to GoReleaser workflow ([3a343d8](https://github.com/loonghao/vx/commit/3a343d82c1c59626551fa7e75584d0f620f4189a))
* add comprehensive package manager distribution support ([1cc160b](https://github.com/loonghao/vx/commit/1cc160bb19cd8ec13747f0fa1c7bcdf8d621ecaf))
* add comprehensive testing and coverage infrastructure ([8c11933](https://github.com/loonghao/vx/commit/8c1193386cc932bc6a7b123ee56653c93188dadc))
* add comprehensive version caching system (inspired by uv) ([5d6cc21](https://github.com/loonghao/vx/commit/5d6cc21577b18b4cf7df0596ee85ccbe1eef4898))
* add cross-platform support for macOS and Windows builds ([5b2f8bd](https://github.com/loonghao/vx/commit/5b2f8bd0bf8407152834028b92a95a41b0c86982))
* add GoReleaser and CI/CD configuration ([465893d](https://github.com/loonghao/vx/commit/465893dcb45663bc54bad98e20acf325e7b8a31d))
* add multi-platform distribution support ([e1a1213](https://github.com/loonghao/vx/commit/e1a1213846b7f793c87eeedb8f67ab9041641008))
* add package managers workflow and documentation ([3be1904](https://github.com/loonghao/vx/commit/3be19047092f68301904ec34a86471c165f1725f))
* add runtime lifecycle hooks and global progress manager ([a2193ab](https://github.com/loonghao/vx/commit/a2193abd9ed1ee00f34739d6b5c72aefe9a828c7))
* add verbose logging control and fix environment isolation ([e52f5ce](https://github.com/loonghao/vx/commit/e52f5ce4fe7cf046123f281ebfa90135a02ed7f5))
* add version numbers to workspace dependencies and automated publishing ([63c90d6](https://github.com/loonghao/vx/commit/63c90d698108c37367d4bc6451dab3febfdc0d90))
* add virtual environment support and separate Rust toolchain ([4661f7b](https://github.com/loonghao/vx/commit/4661f7b1c9abcc8b35a0265ecb96274f494d481e))
* add Windows smart installer with multi-format support ([4f8b82a](https://github.com/loonghao/vx/commit/4f8b82a7ff35a74b1708175ef97399e6e73e7a56))
* add Windows-compatible publishing scripts and environment testing ([3a33c8e](https://github.com/loonghao/vx/commit/3a33c8e4c145338f66a1081dfeb8d582aeca0a9a))
* bump version to 0.1.4 for release automation testing ([ae3b4a8](https://github.com/loonghao/vx/commit/ae3b4a862c945ef520531f952296d4d4af620799))
* bump version to 0.2.2 to trigger new release ([4cd7ef4](https://github.com/loonghao/vx/commit/4cd7ef41d998ab7c1d4883d18379b7ad5a8bf8c8))
* **cli:** add command aliases and short options ([6549318](https://github.com/loonghao/vx/commit/654931852ff04151b22e1637ea29e746f093e065))
* **cli:** add friendly tool suggestions with fuzzy matching ([3c08e4d](https://github.com/loonghao/vx/commit/3c08e4d9fecff87ad0fffc39987747c97dd7843a))
* **cli:** add progress indicators and real tool test framework ([a86d9f3](https://github.com/loonghao/vx/commit/a86d9f31a3dd1c689d7ada31ce716c8147b19aca))
* complete vx project modular refactoring ([90a6008](https://github.com/loonghao/vx/commit/90a600897c4cf4865cd2ac12ddb79134e4a816c5))
* configure release-please for all crates ([c589e49](https://github.com/loonghao/vx/commit/c589e490aba60c6ed92605a86ca5089d9fa1caf9))
* enable multi-platform builds (Stage 2) ([fee8a5f](https://github.com/loonghao/vx/commit/fee8a5f9b1a57e36d31b3cf5d8fb247ba9d74f3f))
* enhance CI/CD automation and release process ([f37d6ad](https://github.com/loonghao/vx/commit/f37d6ade2236ad39b181a30db11c7750c9dfe02f))
* enhance package managers workflow to use GitHub release assets ([ba54be0](https://github.com/loonghao/vx/commit/ba54be01e7a03e979062c1e6fb7b667d47a57fb4))
* enhance release workflow to prevent duplicate releases ([4030216](https://github.com/loonghao/vx/commit/40302161a4bc21ec55be606bdd2996dd38f5cc65))
* fix compilation errors and add comprehensive test suite ([8678ae8](https://github.com/loonghao/vx/commit/8678ae8cd085d837b9a3e89aafbd90e149d7e3b7))
* implement auto-install and symlink virtual environments ([9e994d5](https://github.com/loonghao/vx/commit/9e994d56cd5b2b6424f13585a6930e3c9e734a88))
* implement bun tool and package manager support ([b528873](https://github.com/loonghao/vx/commit/b528873b893d570d63c5e53e7c8f42a91369fdd6))
* implement complete venv command functionality with VenvManager integration ([622f635](https://github.com/loonghao/vx/commit/622f635b9052a462427ea1588d374075274ec644))
* implement comprehensive build optimization and advanced CI/CD pipeline ([9bd501e](https://github.com/loonghao/vx/commit/9bd501e3e3cad77ed7a0225d24d629dba8334ef6))
* implement cross-platform shim functionality ([9973921](https://github.com/loonghao/vx/commit/997392185c3da18cd3aa694c06f0f76913251367))
* implement GoReleaser prebuilt builder for external binaries ([4658190](https://github.com/loonghao/vx/commit/4658190e13af61fe8b466e4f99f3b0a2c26dc2f7))
* implement modular plugin architecture with auto-installation ([17e7358](https://github.com/loonghao/vx/commit/17e735861c2a60a5c8f49dcc71fa3f5f569ec5c8))
* implement multi-channel self-update with CDN fallback ([7b1e5af](https://github.com/loonghao/vx/commit/7b1e5afd43fb9d1f52b259d70c07300d7728c219))
* implement multi-platform native builds with matrix strategy ([b615112](https://github.com/loonghao/vx/commit/b6151124a169b78a04152448a71f9b69423e6fad))
* implement npx and uvx support with environment isolation ([11a56e1](https://github.com/loonghao/vx/commit/11a56e1dc19aa726fe8dc2eb9f566c3829176ff3))
* implement proper release-please + GoReleaser workflow ([9446b78](https://github.com/loonghao/vx/commit/9446b78a0eac8da90ba5b7b494f9ebb139971dab))
* implement separate crates.io publishing workflow ([a485362](https://github.com/loonghao/vx/commit/a485362affca8417bcc9a23620ad08abd60c0efd))
* implement smart publishing system for crates.io ([df1921a](https://github.com/loonghao/vx/commit/df1921a44e8d436e10f9524b39ce9aba3ab99a58))
* implement unified path management and complete crate documentation ([#112](https://github.com/loonghao/vx/issues/112)) ([76d8e0a](https://github.com/loonghao/vx/commit/76d8e0ad1aabd72ef736fe92398d876c58976b53))
* implement universal package management ecosystem ([4d05d33](https://github.com/loonghao/vx/commit/4d05d33ecde85a4e2e0db46d77c9d9b0f854cdec))
* improve CI configuration based on shimexe best practices ([1258da5](https://github.com/loonghao/vx/commit/1258da5153b401c9d1bd8af94383c7323ba4f49e))
* improve CI workflows with winget support and enhanced platform coverage ([a44c19a](https://github.com/loonghao/vx/commit/a44c19acba5ef23570ad634d15a5c2c14ca0a628))
* improve distribution channels and solve GitHub API rate limits ([7f7942b](https://github.com/loonghao/vx/commit/7f7942b84ea21e712dd387af63f23f4262a4f3cb))
* improve install scripts with better platform detection and fallback ([a92a200](https://github.com/loonghao/vx/commit/a92a200938a1d563edba8c62e2f8e8d56d7042cb))
* improve release-please configuration with changelog integration ([095c2bf](https://github.com/loonghao/vx/commit/095c2bf2d0199e21fa34074cec46f9148a6b71d7))
* integrate shim technology for seamless tool version switching ([ce4ba74](https://github.com/loonghao/vx/commit/ce4ba74df90db9eedb1f9399f65af0bf68902b10))
* integrate trycmd for CLI snapshot E2E testing ([79292f9](https://github.com/loonghao/vx/commit/79292f9ccbfcb318d074f8c19c20d2e849120b6a))
* major refactor with modular architecture and PGO support ([dbf883d](https://github.com/loonghao/vx/commit/dbf883de84305986a348394bb7bef888179fb20b))
* merge PGO and tag-based release into unified GoReleaser workflow ([099fbf0](https://github.com/loonghao/vx/commit/099fbf087cf563e607e9242cbb8d4d28e2c522ee))
* modernize CI workflows with latest GitHub Actions and simplified logic ([fbf218c](https://github.com/loonghao/vx/commit/fbf218cbd7ca090614a1b69ad76363bffc77f1b4))
* optimize CI workflows for automated publishing and asset management ([b250d4b](https://github.com/loonghao/vx/commit/b250d4b2efe2afa982da7aebf5a32ce989b5d529))
* optimize CI workflows for efficiency ([59b1e53](https://github.com/loonghao/vx/commit/59b1e5306a7919365463ba1020ba905b4d8f4149))
* optimize core logic with shimexe-core integration and progress bars ([c240f53](https://github.com/loonghao/vx/commit/c240f53fcc2e8a3db43d2dba0c8f5a3166d9fb01))
* optimize GitHub Actions workflows for enhanced stability ([30cb0da](https://github.com/loonghao/vx/commit/30cb0dadd81f44cbca6b2c21004c371b515fb124))
* optimize package publishing order based on dependency hierarchy ([2b361f7](https://github.com/loonghao/vx/commit/2b361f77072c59ea32bee234368571ad862ce855))
* optimize release configuration for single vx package releases ([51481a0](https://github.com/loonghao/vx/commit/51481a09c6d6873f68290cdc61759c919db7ed1a))
* prepare comprehensive release automation testing ([bbf2b10](https://github.com/loonghao/vx/commit/bbf2b1011a522b53b557c3e58e1935f0abba813f))
* prepare for v0.1.2 release with enhanced automation ([f848c39](https://github.com/loonghao/vx/commit/f848c392fbe975d9120f4b6bf8429fa450fea507))
* redesign CLI with modern command structure and remove legacy commands ([974d8f5](https://github.com/loonghao/vx/commit/974d8f59dd63b28db24c721f6c83db360be61d9a))
* refactor vx-core architecture with closed-loop toolchain design ([9c819ee](https://github.com/loonghao/vx/commit/9c819ee3e5c99fe4e0773edc0c3a0b858c646b7f))
* remove vx-shim and improve GitHub API handling ([f5c47f8](https://github.com/loonghao/vx/commit/f5c47f8721b372caae74467f95503d18d2145aef))
* replace release-please with release-plz and fix package managers ([f4085d9](https://github.com/loonghao/vx/commit/f4085d9732c6a5b28741d1b3bbf6de4954f4f1f7))
* simplify release workflow based on shimexe best practices ([12d27c0](https://github.com/loonghao/vx/commit/12d27c0174ac5e5470eb339ec1c379cd0c0ed1df))
* simplify release workflow with modular scripts ([bdc4952](https://github.com/loonghao/vx/commit/bdc495242860da12d9e10555c76f04caa0eea319))
* simplify release-plz configuration based on shimexe best practices ([ea2d0ad](https://github.com/loonghao/vx/commit/ea2d0ad10f3a9c48c30cf75fe3699af6f1978833))
* unify all workspace versions to 0.1.36 ([7240bcd](https://github.com/loonghao/vx/commit/7240bcdd401d9dece4c5b8a3454574d8c0d17822))
* use GoReleaser extra_files best practice for pre-built binaries ([46d4b90](https://github.com/loonghao/vx/commit/46d4b90aadbbd84775de92426854fe1083f09aa0))
* use softprops/action-gh-release for reliable binary asset uploads ([0d229e6](https://github.com/loonghao/vx/commit/0d229e6ae3ec01a09cd915f356c616431c0ca663))

### Bug Fixes

* add better logging and verification for runtime installation ([7c3fbb2](https://github.com/loonghao/vx/commit/7c3fbb2dee0c0ad7cae26bc090b000ee0c333884))
* add config-file and manifest-file paths to release-please action ([f8f606c](https://github.com/loonghao/vx/commit/f8f606cde1b371ecba545fbf2d4b96fa00213959))
* add cross-compilation dependencies for ARM64 target ([e5caccf](https://github.com/loonghao/vx/commit/e5caccf1e156ce949669733057d267bd755108eb))
* add executable_relative_path for all providers and verification framework ([dcc57af](https://github.com/loonghao/vx/commit/dcc57afe3d018a9e8df3036b82c4d7c0736c9396))
* add executable_relative_path for custom archive layouts ([02ba430](https://github.com/loonghao/vx/commit/02ba430dbeabfabd208ef0839281311639b2a78f))
* add GITHUB_TOKEN support and improve API error handling ([406895c](https://github.com/loonghao/vx/commit/406895c424251bade1c916d4782002fa5a63f2ce))
* add missing dev-dependencies for integration tests ([088299d](https://github.com/loonghao/vx/commit/088299dfd1358652c98eec0cad4fb57255e75e6a))
* add remove alias for uninstall and expand CLI test coverage ([afe1e96](https://github.com/loonghao/vx/commit/afe1e963d262956359d9def01551912de365b855))
* add scope placeholder to release-please PR title patterns ([3eded91](https://github.com/loonghao/vx/commit/3eded91195e02ae427e4cfacf151f89896ec6b25))
* add skip-labeling to release-please action and restore config files ([6d14473](https://github.com/loonghao/vx/commit/6d14473e24d1c8720429ccf5dedc05bd9ec85de1))
* address clippy warnings and enhance pre-commit ([31f7d74](https://github.com/loonghao/vx/commit/31f7d7414b4d826817d55e2305373fa32c11ff32))
* address security audit findings ([b768bbb](https://github.com/loonghao/vx/commit/b768bbb7362a9e5c76723844d22eadb8c87247a7))
* align bootstrap-sha with actual v0.2.0 tag commit ([c8b37a8](https://github.com/loonghao/vx/commit/c8b37a880949229f33fa210bf95996b070237221))
* apply clippy suggestions for code quality improvements ([af44628](https://github.com/loonghao/vx/commit/af446287a8b96a1a3a05168b2255e7c651895549))
* bootstrap release-please with correct SHA and version ([b6d0c16](https://github.com/loonghao/vx/commit/b6d0c16b325d860ac5e20ddf8e44cacfdc2108af))
* **bun:** update default version to 1.3.4 ([28f7353](https://github.com/loonghao/vx/commit/28f73532d98e46fc13ba0fe559ef0c8d38a5b7a2))
* change release-please to simple type to avoid workspace scanning ([d0bcfe8](https://github.com/loonghao/vx/commit/d0bcfe84fceaef91f518ba331efa1d63a40532bb))
* check both store and tools directories for installed runtimes ([60d128f](https://github.com/loonghao/vx/commit/60d128f2ca1a8995037a9599594307ef96df4f5a))
* **ci:** add manual trigger support for rebuilding release binaries ([be5691b](https://github.com/loonghao/vx/commit/be5691b161522bdf1191fe0778c342f24f4a5fbd))
* **ci:** add pull-request-title-pattern to release-please config ([2ce7d95](https://github.com/loonghao/vx/commit/2ce7d95edc1c2cfdcaeab7ff27bb193a2186de07))
* **ci:** add x-release-please-version tag to Cargo.toml ([df703e9](https://github.com/loonghao/vx/commit/df703e95b66d6921fde7170c97d7d3ca5d75e79a))
* **ci:** change release-please type from rust to simple ([7a8d12a](https://github.com/loonghao/vx/commit/7a8d12a7d78512796860535fb3b348ca7faac7bd))
* **ci:** correct tool-tests matrix configuration ([c54fa65](https://github.com/loonghao/vx/commit/c54fa65d1b3aeeb553fe2adc1140e4b74c3b36da))
* **ci:** resolve cross-compilation issues in build-check job ([2f565a0](https://github.com/loonghao/vx/commit/2f565a09cf8c0de46313ff390683194178b58c60))
* **ci:** use correct rust-toolchain action name ([971822e](https://github.com/loonghao/vx/commit/971822e67c09c4f229d47bf52b7e19f94259dfa1))
* clean up artifacts directory to prevent git dirty state in GoReleaser ([004780c](https://github.com/loonghao/vx/commit/004780cb84646e48319ab9736ce8c37546221cf9))
* complete format! macro updates for Rust beta compatibility ([95dc7dd](https://github.com/loonghao/vx/commit/95dc7dd9a294eed54c8e3ea7c83bc32e67b1f58d))
* completely skip workspace packages in release-plz to avoid registry checks ([9b7e992](https://github.com/loonghao/vx/commit/9b7e99203c06dd32bbbb55d626d09e03f5fcf774))
* comprehensive release workflow solution ([145a065](https://github.com/loonghao/vx/commit/145a065480b16ecb273ae9a8a4fbaed6924f48c9))
* configure codecov to only warn instead of failing CI ([e284ac7](https://github.com/loonghao/vx/commit/e284ac7e726e17fad81817b067a6c08b01b49c08))
* configure release-plz for git-based versioning to resolve registry conflicts ([f3744f6](https://github.com/loonghao/vx/commit/f3744f601dbfb7f5576d34d16a91d0c34e4a6401))
* configure release-plz to handle workspace packages correctly ([30a167e](https://github.com/loonghao/vx/commit/30a167e2229f96bc8f8369b03968e820ce583636))
* configure release-plz to only create GitHub releases for main vx package ([248a5f3](https://github.com/loonghao/vx/commit/248a5f39a199a317ed18f87740d7dc119f6ae330))
* consolidate release workflows and fix CI issues ([9ebf166](https://github.com/loonghao/vx/commit/9ebf1667eddd3e0b992891918a46b64ed90592ff))
* correct file paths in GoReleaser workflow and add debug output ([0410d42](https://github.com/loonghao/vx/commit/0410d42e172ead4a0f959e8eb2cafe68f8e456c1))
* correct GoReleaser prebuilt binary paths ([033bdb6](https://github.com/loonghao/vx/commit/033bdb671db4afedeae85ee036cd65e0d8f7caf0))
* correct YAML syntax in release workflow ([281cd69](https://github.com/loonghao/vx/commit/281cd6925bc04fafcbf8617319c3293509936a4f))
* correct zip file creation path in create-archives script ([4d8abab](https://github.com/loonghao/vx/commit/4d8ababe212410862a5be2031bcab1622442e345))
* **deps:** update rust crate colored to v3 ([6932576](https://github.com/loonghao/vx/commit/6932576ab192ddaaa6bcefefbf2d4be7f498100e))
* **deps:** update rust crate dirs to v6 ([0418585](https://github.com/loonghao/vx/commit/0418585eb2c721a1ddbd2b4efdaeb05fee94ec71))
* **deps:** update rust crate dirs to v6 ([dfa0931](https://github.com/loonghao/vx/commit/dfa0931e36cca58b38ccebc3ce7390070ef5d275))
* **deps:** update rust crate libc to v0.2.173 ([0f31bdf](https://github.com/loonghao/vx/commit/0f31bdf31d74d5a1806f133e11f1faba8018655c))
* **deps:** update rust crate libc to v0.2.173 ([277664a](https://github.com/loonghao/vx/commit/277664a299e98c3a4a8c9396a70a96ad1bf0ab88))
* **deps:** update rust crate libc to v0.2.174 ([4a21daa](https://github.com/loonghao/vx/commit/4a21daaddf475dba8551e249d6efe25cc948b03c))
* **deps:** update rust crate nix to 0.30 ([90259b2](https://github.com/loonghao/vx/commit/90259b2cb8c1620e9a7ce54569a6ea021748e1ea))
* **deps:** update rust crate reqwest to 0.12 ([0a7b649](https://github.com/loonghao/vx/commit/0a7b6492cc2133b0220d85936f313b20f04047f1))
* **deps:** update rust crate reqwest to v0.12.20 ([7d6613a](https://github.com/loonghao/vx/commit/7d6613a3a58f6f65eed0eccd821d01af0c9e3e20))
* **deps:** update rust crate turbo-cdn to 0.5 ([e3d954f](https://github.com/loonghao/vx/commit/e3d954f1ad74fbb0b02d68a4b07118060bcdadfc))
* **deps:** update rust crate which to v8 ([f5ce820](https://github.com/loonghao/vx/commit/f5ce820f3755a3118bd0fe84de7e9c459f6804a2))
* **deps:** update rust crate which to v8 ([a9a7e21](https://github.com/loonghao/vx/commit/a9a7e210e4707646ba1efb8208647c1b7d845b16))
* **deps:** update rust crate which to v8 ([7d47d76](https://github.com/loonghao/vx/commit/7d47d76d7b739f11c8f586eb1cf3f46bc5826f80))
* **deps:** update rust crate zip to v4 ([d25a961](https://github.com/loonghao/vx/commit/d25a96193c7f2ddab1ae28479bd38e7bca4d4c41))
* **deps:** update rust crate zip to v4 ([fe20c3d](https://github.com/loonghao/vx/commit/fe20c3df76ec3faed8fb42caa8c181de466f344c))
* **deps:** update rust crate zip to v4.1.0 ([12f3ab8](https://github.com/loonghao/vx/commit/12f3ab8377e6804c42dcd332cab296698b4bdd48))
* **deps:** update rust crate zip to v4.1.0 ([8ac5678](https://github.com/loonghao/vx/commit/8ac567840e06f1c2376ee569458bd94179835dd5))
* **deps:** update rust crate zip to v6 ([b267cdd](https://github.com/loonghao/vx/commit/b267cdd2e5f7c0f90414078abc7792d1527b5dc2))
* disable nfpms and fix archive format deprecation in GoReleaser ([46dae87](https://github.com/loonghao/vx/commit/46dae8737088af3f1daec28ad563593143330f7a))
* enable release PR creation by setting release_always = true ([#79](https://github.com/loonghao/vx/issues/79)) ([d9aa11d](https://github.com/loonghao/vx/commit/d9aa11d8926f952b3f0787b30dd7365129b0e075))
* enhance CI permissions and configure release-please for PR-only mode ([f577185](https://github.com/loonghao/vx/commit/f5771854e1b93d8237200680d8b5935e77a7da18))
* execute .cmd/.bat files via cmd.exe on Windows ([c7240fe](https://github.com/loonghao/vx/commit/c7240fe110193ab8c7a7ffb93ab06a1352b84a74))
* fix clippy warnings in test code ([c3e919b](https://github.com/loonghao/vx/commit/c3e919bdce1c2cb7856abf30553c433a007060c8))
* fix install scripts platform naming and release workflow ([b5d3611](https://github.com/loonghao/vx/commit/b5d3611b3a8a54d8d7ecc62aba95505ed8baad5c))
* implement Default trait for ConsoleProgressReporter and enhance pre-commit ([f6724ab](https://github.com/loonghao/vx/commit/f6724ab95c9b3fc12a5fd470231a8604d6f341f1))
* implement release-please best practices for output handling ([8591fa3](https://github.com/loonghao/vx/commit/8591fa37f8e38f040ff8fc80108df0e8cbcae995))
* implement release-please best practices for output handling ([e0aeb6a](https://github.com/loonghao/vx/commit/e0aeb6a403e2b5636cea27577a2fd9b68fc87402))
* improve artifact path debugging and error handling ([578626c](https://github.com/loonghao/vx/commit/578626caff03a50bd11258894d5d2f105c8b8b78))
* improve CI checkout for fork PRs and optimize release workflows ([46b0671](https://github.com/loonghao/vx/commit/46b0671d588dff94b84422ba0cac7371f3cf7fb9))
* improve CI publishing with enhanced error handling and fallback ([8a6f693](https://github.com/loonghao/vx/commit/8a6f6936ce963ab73a5b5611c9f64f17f9e584d1))
* improve executable detection for archives with subdirectories ([3da4b69](https://github.com/loonghao/vx/commit/3da4b69f3298d045cef8d2d66bde4efbdf3b7647))
* improve release-plz commit detection configuration ([5e754ed](https://github.com/loonghao/vx/commit/5e754edf0ff66eddb7ce8e81cf878b90a2d94ae9))
* improve remove command error handling in force mode ([a5fe16b](https://github.com/loonghao/vx/commit/a5fe16b23a0648e934d44508a9448a7f2e694fb1))
* Installer script for powershell ([4e0f3e0](https://github.com/loonghao/vx/commit/4e0f3e021974a9f4b83094e50a2c215647466df3))
* make codecov upload optional to prevent CI failures ([cf23299](https://github.com/loonghao/vx/commit/cf23299867837359014f4efa14d7daada9fdaf12))
* move release-plz dry-run to CI and enhance token troubleshooting ([e17233a](https://github.com/loonghao/vx/commit/e17233aaf5942226ec684c5c6fdf1e857278b6b6))
* normalize line endings to CRLF on Windows ([718eb82](https://github.com/loonghao/vx/commit/718eb8224044de446c45715c1cd75b6dc0c7e9af))
* optimize release workflows to use conventional commits ([fbb65de](https://github.com/loonghao/vx/commit/fbb65def4ae7c13c364cb49aa1f9cb1731543142))
* optimize release-plz configuration to prevent duplicate CI triggers ([4f6a77d](https://github.com/loonghao/vx/commit/4f6a77d2af2b4d1b6d438f867eb77b1781897aa4))
* PathResolver now checks both store and tools directories ([1ab8bc4](https://github.com/loonghao/vx/commit/1ab8bc4077648cd8e4c177effbfa787ff3567ccc))
* PNPM download URL and list command store directory support ([62f8098](https://github.com/loonghao/vx/commit/62f8098c4607a099fc7096fac9ef13b2baf70a88))
* prevent tag-release-assets workflow from triggering on individual crate tags ([1a8b041](https://github.com/loonghao/vx/commit/1a8b0412ab283bd4f38e219cd4846980ba387542))
* provide download URLs for all tools on all platforms ([720fbda](https://github.com/loonghao/vx/commit/720fbdafe88513d6c0a10af51da9f0a9bbe6ea77))
* **providers:** update yarn and bun default versions ([decab6e](https://github.com/loonghao/vx/commit/decab6e2ecee09eb08d6c30685388762ad43a748))
* remove assert!(true) and add assertions_on_constants lint ([d89fa70](https://github.com/loonghao/vx/commit/d89fa701522058235120d89ff03505c8a0a1532d))
* remove async trait to fix CI compilation issues ([e0c2f29](https://github.com/loonghao/vx/commit/e0c2f294bb14e130ed23cf734826419f99a9edc8))
* remove branches filter from workflow_run trigger ([4eab1ac](https://github.com/loonghao/vx/commit/4eab1ac21042671e370ffb8314681c78a06dad4b))
* remove deprecated use command and fix binary installation ([3fcf253](https://github.com/loonghao/vx/commit/3fcf253745c504916a16d5e20c90b2cb67ca0a2c))
* remove FreeBSD target and add distributed release workflow ([d086b3b](https://github.com/loonghao/vx/commit/d086b3bf1a33388fd2f38a22e0e9dcdc927a9ed2))
* remove global RUSTFLAGS to fix macOS build failures ([1b2be02](https://github.com/loonghao/vx/commit/1b2be02ccfeecdc66889478e9e7f5bfcf1dd18d3))
* remove invalid --jobs=0 parameter from cargo build commands ([cc88089](https://github.com/loonghao/vx/commit/cc880893a5ceaf11b46fcc0abda99e85472c2ce7))
* remove invalid allow_dirty field from package section ([85ab45c](https://github.com/loonghao/vx/commit/85ab45c7334e7dcb416a1008dbdda478ace7b3c7))
* remove invalid bootstrap-sha from release-please config ([0519f6a](https://github.com/loonghao/vx/commit/0519f6a0ed0f15e2c20257c384a8b70eebe01c1c))
* remove invalid release_commits field from package section ([ac32960](https://github.com/loonghao/vx/commit/ac329608fe5b05a41d0aad60adb1e994f8f2c2c7))
* remove LLD linker configuration for macOS targets ([118c3a7](https://github.com/loonghao/vx/commit/118c3a7c953d0ef360f9ed4f0fd3179f759bd10a))
* remove unsupported path field from release-plz.toml and add dry-run step ([69f77ab](https://github.com/loonghao/vx/commit/69f77ab8b04cc3efa788564c005441233f0fb5c6))
* remove useless format! usage in venv command ([6ffe8ce](https://github.com/loonghao/vx/commit/6ffe8cecd3a0ef82066fafa6875f5e170f5ac751))
* replace problematic execute tests with safe unit tests ([73f4efd](https://github.com/loonghao/vx/commit/73f4efd1d61b441c70ae62d9b803030773192f53))
* reset release-please configuration to resolve CI issues ([178a415](https://github.com/loonghao/vx/commit/178a4152b554143358ea27b7417e04799807be2c))
* resolve 'jobs may not be 0' error with minimal configuration ([e65ee53](https://github.com/loonghao/vx/commit/e65ee5302fd9ab50a3b7c40cb333a30db18d30fa))
* resolve all clippy warnings across workspace ([8266713](https://github.com/loonghao/vx/commit/8266713f186b91cdd13db58146617629e4e8d4e0))
* resolve all clippy warnings and improve code quality ([585f266](https://github.com/loonghao/vx/commit/585f266c82390260b6f61fd94f04bfab87be8425))
* resolve all clippy warnings in test suite ([074e30b](https://github.com/loonghao/vx/commit/074e30b6a0e03914febc7f4d54b98238d6ad37ef))
* resolve build job conditions and GoReleaser dirty state ([560e87c](https://github.com/loonghao/vx/commit/560e87c68ad4f43aa359f6404476bad2c000945e))
* resolve CARGO_BUILD_JOBS and cross installation issues ([fa38fb5](https://github.com/loonghao/vx/commit/fa38fb515229183a5477f630a6ac8c844a3d5b9f))
* resolve CI issues and improve code quality ([465710e](https://github.com/loonghao/vx/commit/465710e39c8868466772321113976c72bd83e275))
* resolve CI issues and update documentation ([d11ba2c](https://github.com/loonghao/vx/commit/d11ba2cc11547eb94aa976c7586fbb860959e2ae))
* resolve CI publishing issues with release-plz ([6a10380](https://github.com/loonghao/vx/commit/6a103805c23b92f2830627b0154df417622294c5))
* resolve CI sccache and cargo-audit issues ([81c0cba](https://github.com/loonghao/vx/commit/81c0cbab841234b03ab6ed5459806086413fa3da))
* resolve CI shell syntax errors and remove test workflows ([85c8912](https://github.com/loonghao/vx/commit/85c8912b27f287fe1f91c636c81334653f9ec9f9))
* resolve CI test failures and binary conflicts ([fea5387](https://github.com/loonghao/vx/commit/fea538779428c95e879465a7d4a3e17510015c58))
* resolve clippy warnings and test failures ([a657530](https://github.com/loonghao/vx/commit/a657530fae44a7521be6d02a43c0140a4c995ddd))
* resolve clippy warnings for inline format args ([edc666c](https://github.com/loonghao/vx/commit/edc666cd2fee84e95905620377e8741c92ee419f))
* resolve clippy warnings in self-update module ([41a2570](https://github.com/loonghao/vx/commit/41a2570a4d86feaa0f6f0b7cd0bcda2c53416443))
* resolve compilation errors in config integration tests ([3994397](https://github.com/loonghao/vx/commit/3994397e38aba5be21a7aa4b7c7b4f483577fc6d))
* resolve coverage testing compilation errors and warnings ([c948e6a](https://github.com/loonghao/vx/commit/c948e6a601a92a727c8bb57a991a95019394d01b))
* resolve cross-compilation issues with proper cross tool ([b21daa0](https://github.com/loonghao/vx/commit/b21daa005ce3b250bac81cdba53bc6aa9166be67))
* resolve duplicate 'release' key in GoReleaser config ([8034f6d](https://github.com/loonghao/vx/commit/8034f6d2115021256aed01c4053204246a5f2ccf))
* resolve GitHub Actions release and installer issues ([1c3503c](https://github.com/loonghao/vx/commit/1c3503cd6f5c2604f7c9d8347211b2ceca682107))
* resolve GoReleaser and release-please workflow issues ([c20794e](https://github.com/loonghao/vx/commit/c20794e9e58f588c45c6485ca0726564b46746b4))
* resolve GoReleaser before hooks shell script parsing error ([a1019bd](https://github.com/loonghao/vx/commit/a1019bd663577f69a92cc9966df58d867e55f42d))
* resolve GoReleaser build parameter issues ([dc51e11](https://github.com/loonghao/vx/commit/dc51e1142af4edd42cdfddf62e457a5246e9031e))
* resolve GoReleaser configuration issues ([d89514d](https://github.com/loonghao/vx/commit/d89514d1dcde0a4ef9e4afc40ba3fd7200b89d29))
* resolve GoReleaser template function error ([87ec968](https://github.com/loonghao/vx/commit/87ec968f6d828bb11b10c022e94aa49cd0249ab0))
* resolve import errors and clippy warnings in tool packages ([f5b3247](https://github.com/loonghao/vx/commit/f5b32474df80af5dd3dc5df1a650a2b8ec77eaff))
* resolve lifetime errors in progress reporter and test issues ([cd40c63](https://github.com/loonghao/vx/commit/cd40c63814fbe45aaaaca28e334184e000f0c565))
* resolve Linux musl cross-compilation OpenSSL issues ([3abe5bb](https://github.com/loonghao/vx/commit/3abe5bbf7220237c82f6d3c73c8304b8e83583ce))
* resolve Mac test failures in tracing_setup module ([e0d428a](https://github.com/loonghao/vx/commit/e0d428a130cc032cdeb8c21a8ca6e22ee69ef9cc))
* resolve nfpm RPM package creation error ([fd44e03](https://github.com/loonghao/vx/commit/fd44e035f58eb8aaab5aed5ba8ae55606f65f347))
* resolve release-please configuration and workflow trigger issues ([5e1cd22](https://github.com/loonghao/vx/commit/5e1cd22d3fa512bf6a2b49dddd7b920699fae09f))
* resolve release-please configuration issues ([9717950](https://github.com/loonghao/vx/commit/9717950f7f28ae58c450272450f16c42a14b123d))
* resolve release-please tag configuration issues ([ca9e9b9](https://github.com/loonghao/vx/commit/ca9e9b98d361b18b165ce8aea29cf16ce75a9dcb))
* resolve release-please untagged PR issue ([391be7d](https://github.com/loonghao/vx/commit/391be7de14dcdd6c6bc257810c19d2e9af04c8a6))
* resolve release-please untagged PR issue by updating configuration ([8c13a25](https://github.com/loonghao/vx/commit/8c13a25dcdfa010b5b51cea581f22e8b07ee27a6))
* resolve release-plz configuration and dependency version issues ([0ff4d24](https://github.com/loonghao/vx/commit/0ff4d241afc7d2777e6e876953bb1f1c4b347268))
* resolve release-plz configuration and package manager issues ([9852e9c](https://github.com/loonghao/vx/commit/9852e9cef092ee13cd8c4db10efdf034dd93d676))
* resolve release-plz workspace dependency issues ([2fc3b83](https://github.com/loonghao/vx/commit/2fc3b832df71ff076724523a3c3eca7ae745e57e))
* resolve remaining clippy warnings in where_cmd.rs ([b45afe2](https://github.com/loonghao/vx/commit/b45afe282aed1da6d2982aaa7b6a2e558165aa3c))
* resolve ShimConfig args type mismatch and remove legacy format support ([dc18b27](https://github.com/loonghao/vx/commit/dc18b27f28e1e7c10458514c62eb08922f896946))
* resolve test failure and installation script issues ([7edbfc0](https://github.com/loonghao/vx/commit/7edbfc04ebb793cbc16cf10c7b96dc8055495ed7))
* resolve venv test failures and improve workspace publishing script ([f40b519](https://github.com/loonghao/vx/commit/f40b519f5d08e24df9a1075c87eb8daa6f89bcbd))
* resolve workflow test integration issues and add comprehensive test suite ([d2b7fb5](https://github.com/loonghao/vx/commit/d2b7fb5e8126d16b839b46b3cc4194d4cd7de7c5))
* restore proper release-please + GoReleaser separation ([73e2d00](https://github.com/loonghao/vx/commit/73e2d00793ac5a50b2641cb5ac81c2284df1755f))
* separate cross-compilation build from native testing ([3cbe273](https://github.com/loonghao/vx/commit/3cbe273fd2bfc4c4bc8f7b052463f2ecfdb49c6f))
* simplify build to use native cargo instead of cross ([0ac71a5](https://github.com/loonghao/vx/commit/0ac71a5ef28e60090ee2f62e4b6eb259f50ebb40))
* simplify CI build configuration for better reliability ([831b5ec](https://github.com/loonghao/vx/commit/831b5ec908499e37801692033fcf88ec5c4ea42b))
* simplify cross-platform build to Windows only for stability ([2be90fe](https://github.com/loonghao/vx/commit/2be90fe9a9e74877a7ac2ffe717fd95afaa24b67))
* simplify GoReleaser configuration for better stability ([d123532](https://github.com/loonghao/vx/commit/d1235326c9ce656c61173aa8aa5d94c94728efd3))
* simplify release-please configuration to resolve CI issues ([a3dbee6](https://github.com/loonghao/vx/commit/a3dbee66da63b9b05c9ea734b48c31ca2667c3ed))
* simplify release-plz configuration based on working reference ([ac38200](https://github.com/loonghao/vx/commit/ac382007e412a21d767bb6080274c90e98f65293))
* simplify release-plz.toml following shimexe best practices ([1e8728a](https://github.com/loonghao/vx/commit/1e8728a477d16a89f73293ef62b3d2556ba72393))
* skip UPX compression for macOS to resolve build issues ([8805d4c](https://github.com/loonghao/vx/commit/8805d4cf0ac2faf7eee6cc7761e95273212d0ff7))
* suppress dead_code warnings in test utilities ([c210346](https://github.com/loonghao/vx/commit/c210346ed964ec25435ff0f8d302b7cd1059fc0f))
* synchronize release-please version with existing v0.1.3 tag ([0446264](https://github.com/loonghao/vx/commit/044626404ceb4f2ae5d0ae8d15a23f10d0868e5b))
* synchronize version to 0.1.1 and remove incorrect v0.2.0 tag ([42704e4](https://github.com/loonghao/vx/commit/42704e4ac6a998fbef2abb3ad2816c38766119bd))
* temporarily disable ARM64 cross-compilation due to linker issues ([15a8a0a](https://github.com/loonghao/vx/commit/15a8a0a33bf6a569572fd25967aacd190bad85d8))
* **test:** make tool tests more robust for CI environments ([338cff6](https://github.com/loonghao/vx/commit/338cff6bff02b5051de9b10b8c06df60aca5ddc1))
* **tests:** fix version_cache offline test and invalid-command snapshot ([0ca5efe](https://github.com/loonghao/vx/commit/0ca5efe91640f85cfa358c3c1acf1af063a45dc6))
* **tests:** skip bun tests when bun is not installed ([4c417df](https://github.com/loonghao/vx/commit/4c417dfe6ea04d25b5254d5a7c78f13df68eb5af))
* **test:** use common module for vx binary lookup in e2e_tests ([6c6c30d](https://github.com/loonghao/vx/commit/6c6c30dbedf050e94cccfc5017a86b107120581c))
* **unix:** correct CommandExt::exec implementation ([8473793](https://github.com/loonghao/vx/commit/8473793e7c53c9783fdd22ab00df8b0fed722a1b))
* update all format! macros for Rust beta compatibility ([b6a4365](https://github.com/loonghao/vx/commit/b6a4365f3df0760b257bc7e44693aa08887a446a))
* update clean-cache-dry-run test to use flexible matching ([dbfb1cd](https://github.com/loonghao/vx/commit/dbfb1cd738085771402338e9eb273fd58856939b))
* update execute_tests.rs to use new function signatures ([abfc62b](https://github.com/loonghao/vx/commit/abfc62b83cc497f53c225a21bafbcf53f0c45ba9))
* update release-please workflow and clean up config files ([b6e8721](https://github.com/loonghao/vx/commit/b6e87218237bad64f69cd7e43cc31caae07793d0))
* update remaining format! macro in config.rs ([94aa7c8](https://github.com/loonghao/vx/commit/94aa7c8dac8a0d1f4407662e0039388e1b0f076f))
* update snapshot test and fix clippy warnings ([b1cc089](https://github.com/loonghao/vx/commit/b1cc0892713a4234921908af1265020ff919e8ed))
* use ... wildcard for list commands to ignore output order ([731aaf9](https://github.com/loonghao/vx/commit/731aaf9745974e8ecad27eaeb73bf91e12fe8844))
* use absolute path for vx binary in e2e tests ([d3fbbe8](https://github.com/loonghao/vx/commit/d3fbbe874a0f6a497a960234c43a37e5b90d01b8))
* use archive-only approach instead of prebuilt builder in GoReleaser ([58d48f6](https://github.com/loonghao/vx/commit/58d48f641215a1102f93b5842cb18bcc34354861))
* use correct GoReleaser envOrDefault function ([f92c27a](https://github.com/loonghao/vx/commit/f92c27a0f66670dce8638ff28d35e3ecb9f3e497))
* use correct release-plz action and resolve version sync issues ([486ea2b](https://github.com/loonghao/vx/commit/486ea2bc94cbed91010d3a9f2bf72bb69f65a93e))
* use extra_files instead of archives for direct binary uploads in GoReleaser ([f48f1cf](https://github.com/loonghao/vx/commit/f48f1cfa38217f0e0ac08b6bb4fdb692381240ed))
* use looser matching for list commands in trycmd tests ([b5491a1](https://github.com/loonghao/vx/commit/b5491a16e79cd285d74f0dbc8ede7710a7dab3b0))
* use manual GitHub CLI upload instead of GoReleaser extra_files for binary assets ([85662a6](https://github.com/loonghao/vx/commit/85662a635149827a1635233a534354e37a19f530))
* use native Rust toolchain for Windows cross-compilation ([41bce2b](https://github.com/loonghao/vx/commit/41bce2bcde6061f875326ff32525deb5955917b1))
* use release.extra_files for uploading pre-built binaries in GoReleaser ([520d2de](https://github.com/loonghao/vx/commit/520d2de7a35a5c45e937e28ed564479e943133ea))
* use Windows GNU target for better cross-compilation support ([b3e0c09](https://github.com/loonghao/vx/commit/b3e0c097995fa273d127bf102dbee171fc90e1c0))
* **uv:** correct download URL format ([6c84f4e](https://github.com/loonghao/vx/commit/6c84f4e22b57fd46f8899645030ab48ab2e608ee))
* **which:** fallback to system PATH when tool not found in vx-managed installations ([db91013](https://github.com/loonghao/vx/commit/db910136a8a24b52bc6493840b6e870e0fd4f549))
* **windows:** support .cmd files and fix uv archive structure ([4f38c84](https://github.com/loonghao/vx/commit/4f38c841113aae66fe2e04e3adb8156eeb2eec80))

### Code Refactoring

* add vx-sdk and cleanup deprecated code ([5f07376](https://github.com/loonghao/vx/commit/5f073762831fde0e1f74600675444309946b55ba))
* **ci:** use artifact sharing to avoid redundant builds ([2b3505e](https://github.com/loonghao/vx/commit/2b3505e505f631e33b9c11bbcf07411a382db224))
* migrate vx-cli to ProviderRegistry and remove legacy vx-tools ([dafa3a4](https://github.com/loonghao/vx/commit/dafa3a47618d57296a06f99b96329223d011c3b0))
* restructure tests to dedicated tests/ directories ([b1d4c93](https://github.com/loonghao/vx/commit/b1d4c9316273371bc881659ffa510176f7e6ea1b))
* simplify main package by reusing vx-cli main function ([7893190](https://github.com/loonghao/vx/commit/78931901ea98da330fa8d3e64e513a1c7c0d08e7))
* simplify release-plz.toml following shimexe best practices ([5bbe1c5](https://github.com/loonghao/vx/commit/5bbe1c50acf5145924516d9a1ce4f5ee480a75a8))

### Documentation

* add codecov setup guide ([a3199a1](https://github.com/loonghao/vx/commit/a3199a1e9f50ba8f4b0bb57e25e75167b7f446c8))
* add comprehensive implementation summary ([00c764e](https://github.com/loonghao/vx/commit/00c764e648e4a0444f3f1f1e24776b0293d364cb))
* add crates overview README and enhance vx-sdk documentation ([7b185e4](https://github.com/loonghao/vx/commit/7b185e44315daacc2da6eef04c46cf968c0cba67))
* add post-merge release guide ([8615dbe](https://github.com/loonghao/vx/commit/8615dbe16df0e3a01cff3a1e7f3108df2044edf3))
* add release automation note to README ([2c2aacb](https://github.com/loonghao/vx/commit/2c2aacb23beb27d4d0c979238d34eed613fbd60e))
* add testing guide and implementation summary ([a9761c0](https://github.com/loonghao/vx/commit/a9761c0b64eb9950c1b4ff10eac371c40f0f254d))
* update README installation instructions ([bc875ef](https://github.com/loonghao/vx/commit/bc875eff5f767d209e45b9da52dfc5a84866b8ac))
* update README with upcoming tool support ([a989762](https://github.com/loonghao/vx/commit/a989762401068767282b08de370ec13262450c04))

## [0.4.1](https://github.com/loonghao/vx/compare/v0.4.0...v0.4.1) - 2025-06-19

### Fixed

* prevent tag-release-assets workflow from triggering on individual crate tags
* configure release-plz to only create GitHub releases for main vx package

## [0.4.0](https://github.com/loonghao/vx/compare/v0.3.0...v0.4.0) - 2025-06-19

### Added

* implement unified path management and complete crate documentation ([#112](https://github.com/loonghao/vx/pull/112))

## [0.3.0](https://github.com/loonghao/vx/compare/v0.2.6...v0.3.0) - 2025-06-19

### Added

* fix compilation errors and add comprehensive test suite
* refactor vx-core architecture with closed-loop toolchain design
* complete vx project modular refactoring
* [**breaking**] remove vx-shim and improve GitHub API handling
* optimize core logic with shimexe-core integration and progress bars

### Fixed

* resolve release-plz configuration and dependency version issues
* *(deps)* update rust crate which to v8
* *(deps)* update rust crate dirs to v6
* resolve coverage testing compilation errors and warnings
* resolve Linux musl cross-compilation OpenSSL issues
* resolve import errors and clippy warnings in tool packages

### Other

* *(deps)* update codecov/codecov-action action to v5
* update README with upcoming tool support

## [0.2.6](https://github.com/loonghao/vx/compare/v0.2.5...v0.2.6) - 2025-06-18

### Added

* improve install scripts with better platform detection and fallback
* optimize release configuration for single vx package releases

### Other

* simplify release-plz.toml following shimexe best practices

## [0.2.5](https://github.com/loonghao/vx/compare/v0.2.4...v0.2.5) - 2025-06-18

### Fixed

* Installer script for powershell
* simplify release-plz.toml following shimexe best practices
* optimize release-plz configuration to prevent duplicate CI triggers
* improve CI checkout for fork PRs and optimize release workflows

## [0.2.4](https://github.com/loonghao/vx/compare/v0.2.3...v0.2.4) - 2025-06-17

### Added

* simplify release-plz configuration based on shimexe best practices
* simplify release workflow based on shimexe best practices
* improve CI configuration based on shimexe best practices

### Fixed

* separate cross-compilation build from native testing
* add cross-compilation dependencies for ARM64 target
* temporarily disable ARM64 cross-compilation due to linker issues
* use correct release-plz action and resolve version sync issues
* move release-plz dry-run to CI and enhance token troubleshooting

### Other

* update README installation instructions

## [Unreleased]

## [0.2.3](https://github.com/loonghao/vx/compare/v0.2.2...v0.2.3) - 2025-06-16

### 🐛 Bug Fixes

* remove invalid release_commits field from package section
* improve release-plz commit detection configuration

# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0](https://github.com/loonghao/vx/compare/v0.1.36...v0.2.0) - 2025-06-15

### Bug Fixes

* *(deps)* update rust crate zip to v4.1.0
* add missing dev-dependencies for integration tests
* remove deprecated use command and fix binary installation
* resolve venv test failures and improve workspace publishing script
* resolve release-plz workspace dependency issues
* configure release-plz to handle workspace packages correctly
* resolve release-plz configuration and package manager issues

### Features

* add Windows-compatible publishing scripts and environment testing
* unify all workspace versions to 0.1.36
* add version numbers to workspace dependencies and automated publishing
* implement separate crates.io publishing workflow

### Refactor

* simplify main package by reusing vx-cli main function
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.4 (2025-06-13)

### Features

* optimize GitHub Actions workflows for enhanced stability
* prepare comprehensive release automation testing
* test release-please integration after version sync

### Bug Fixes

* synchronize release-please version with existing v0.1.3 tag
* resolve GoReleaser build parameter issues

## 0.1.2 (2025-06-13)

### Bug Fixes

* resolve release-please untagged PR issue by updating configuration ([8c13a25](https://github.com/loonghao/vx/commit/8c13a25dcdfa010b5b51cea581f22e8b07ee27a6))
* synchronize version to 0.1.1 and remove incorrect v0.2.0 tag ([42704e4](https://github.com/loonghao/vx/commit/42704e4ac6a998fbef2abb3ad2816c38766119bd))
* add scope placeholder to release-please PR title patterns ([3eded91](https://github.com/loonghao/vx/commit/3eded91195e02ae427e4cfacf151f89896ec6b25))

## 0.1.1 (2025-06-11)

## What's Changed

* fix: resolve GoReleaser and release-please workflow issues by @loonghao in <https://github.com/loonghao/vx/pull/31>
* fix: enhance CI permissions and configure release-please for PR-only mode by @loonghao in <https://github.com/loonghao/vx/pull/33>
* fix: resolve CI shell syntax errors and remove test workflows by @loonghao in <https://github.com/loonghao/vx/pull/34>
* fix: implement release-please best practices for output handling by @loonghao in <https://github.com/loonghao/vx/pull/35>

**Full Changelog**: <https://github.com/loonghao/vx/compare/v0.1.0...v0.1.1>

## 0.1.0 (2025-06-11)

## What's Changed

* chore: Configure Renovate by @renovate in <https://github.com/loonghao/vx/pull/1>
* fix(deps): update rust crate dirs to v6 by @renovate in <https://github.com/loonghao/vx/pull/3>
* fix(deps): update rust crate reqwest to 0.12 by @renovate in <https://github.com/loonghao/vx/pull/2>
* feat: Add GoReleaser CI/CD and improve CLI user experience by @loonghao in <https://github.com/loonghao/vx/pull/5>
* fix(deps): update rust crate reqwest to v0.12.20 by @renovate in <https://github.com/loonghao/vx/pull/9>
* fix(deps): update rust crate which to v8 by @renovate in <https://github.com/loonghao/vx/pull/6>
* chore(deps): update dependency go to 1.24 by @renovate in <https://github.com/loonghao/vx/pull/19>
* fix(deps): update rust crate zip to v4 - autoclosed by @renovate in <https://github.com/loonghao/vx/pull/7>
* chore(deps): update goreleaser/goreleaser-action action to v6 by @renovate in <https://github.com/loonghao/vx/pull/20>
* fix: resolve CI release-please configuration issues by @loonghao in <https://github.com/loonghao/vx/pull/21>

## New Contributors

* @renovate made their first contribution in <https://github.com/loonghao/vx/pull/1>
* @loonghao made their first contribution in <https://github.com/loonghao/vx/pull/5>

**Full Changelog**: <https://github.com/loonghao/vx/commits/vx-v0.1.0>

## [Unreleased]

### Features

* **Virtual Environment Support**: Added `vx venv` command for creating and managing isolated development environments
  * `vx venv create <name>` - Create new virtual environment with specific tool versions
  * `vx venv activate <name>` - Generate activation script for shell integration
  * `vx venv list` - List all virtual environments
  * `vx venv remove <name>` - Remove virtual environment
  * `vx venv current` - Show current active environment
* **Rust Toolchain Separation**: Split Rust tool into separate `cargo` and `rustc` tools
  * `vx cargo` - Rust package manager and build tool
  * `vx rustc` - Rust compiler
* **Environment Isolation Improvements**: Enhanced tool execution to better support isolated environments
* Initial implementation of vx - Universal Development Tool Manager
* Support for UV (Python package manager)
* Support for Node.js and npm
* Support for Go toolchain
* Support for Rust and Cargo
* Plugin architecture for extensibility
* Multi-platform support (Linux, macOS, Windows, FreeBSD)
* Automatic tool installation and version management
* Project-specific configuration support

### Documentation

* Comprehensive README with installation instructions
* Chinese translation (README_zh.md)
* Plugin documentation and examples

### Build System

* GoReleaser configuration for multi-platform releases
* GitHub Actions CI/CD pipeline
* Docker image support
* Package manager integration (Homebrew, Scoop)

## [0.1.0] - 2025-01-09

### Features

* Initial release of vx
* Basic plugin system
* Core tool support (UV, Node.js, Go, Rust)
* Command-line interface
* Configuration management

[Unreleased]: https://github.com/loonghao/vx/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/loonghao/vx/releases/tag/v0.1.0
