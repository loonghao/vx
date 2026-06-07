# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.9.17](https://github.com/loonghao/vx/compare/v0.9.16...v0.9.17) (2026-06-07)


### Features

* **ai:** add vx ai headroom CLI skeleton with bridge runner ([#962](https://github.com/loonghao/vx/issues/962)) ([8fafbf4](https://github.com/loonghao/vx/commit/8fafbf4a2dfec71d03d650ad1b50fc1e5cb8ce22))
* **headroom:** implement MCP config setup for AI agents ([4e1a355](https://github.com/loonghao/vx/commit/4e1a355704f6afdc31144e16090f67880c716d01))
* **headroom:** implement proxy lifecycle management ([6a15d7e](https://github.com/loonghao/vx/commit/6a15d7e7665858504d277d209f8c4e0e75a55885))
* migrate hugo provider to github_smart_provider (PIP-558) ([b449dcb](https://github.com/loonghao/vx/commit/b449dcb2835300a88e4d481d46b9549b28ca4e18))
* **providers:** add sentry-cli provider (getsentry/sentry-cli) ([c967a03](https://github.com/loonghao/vx/commit/c967a03ece7baccdde12943fae40a252cab93a2a))
* **providers:** use vx-org/mirrors as stable download source ([7db5c8a](https://github.com/loonghao/vx/commit/7db5c8ab3f5d61afb1bc1084355cd6e4a1fa24a5))
* **providers:** use vx-org/mirrors as stable download source for all providers ([b5dfca4](https://github.com/loonghao/vx/commit/b5dfca4755f68fa4ff473f6b8d362eead0b26a36))
* **providers:** use vx-org/mirrors as stable download source for all providers ([b5dfca4](https://github.com/loonghao/vx/commit/b5dfca4755f68fa4ff473f6b8d362eead0b26a36))


### Bug Fixes

* **headroom:** add libc dependency for unix builds ([3fad63e](https://github.com/loonghao/vx/commit/3fad63e782fc9228458dadbb40afea8b2ba41130))
* **headroom:** reject invalid JSON configs, surface write errors ([842a3d5](https://github.com/loonghao/vx/commit/842a3d552f300f74f9551737b66961c601a622ca))


### Documentation

* add smart provider decision tree to creating-provider guide ([#961](https://github.com/loonghao/vx/issues/961)) ([b71b1e6](https://github.com/loonghao/vx/commit/b71b1e6a99f1270233d10dedb4ac5ad08bfab706))

## [0.9.16](https://github.com/loonghao/vx/compare/v0.9.15...v0.9.16) (2026-06-05)


### Features

* **installer:** auto-verify sidecar checksum files from GitHub releases ([6cbdc9a](https://github.com/loonghao/vx/commit/6cbdc9ac89ae50d75b9138b4357b7027ba13f62f))
* **starlark:** implement github_smart_provider MVP per RFC 0041 ([f274d6b](https://github.com/loonghao/vx/commit/f274d6b161644c915f025e59e340246128528615))


### Documentation

* add RFC 0041 for github_smart_provider asset scoring ([871bde6](https://github.com/loonghao/vx/commit/871bde65f73abe0c7a287931722f775428b8b246))
* add RFC 0041 for github_smart_provider asset scoring ([1b7918b](https://github.com/loonghao/vx/commit/1b7918bfa6ba45338b37652243329ad075d605b4))

## [0.9.15](https://github.com/loonghao/vx/compare/v0.9.14...v0.9.15) (2026-06-05)


### Bug Fixes

* **runtime:** replace zip extract() with entry-by-entry extraction to prevent truncation ([ff3c418](https://github.com/loonghao/vx/commit/ff3c4189ccf9cf925290607d9708dfc575f63940))

## [0.9.14](https://github.com/loonghao/vx/compare/v0.9.13...v0.9.14) (2026-06-05)


### Features

* add codegraph provider for colbymchenry/codegraph releases ([66d7483](https://github.com/loonghao/vx/commit/66d74835e4ffb26ea89c53ab6995717d03cd4b57))


### Documentation

* update version references to v0.9.13 (2026-06-03) ([92e04d5](https://github.com/loonghao/vx/commit/92e04d5f31f1624ee86869489c84170ebaf0026d))

## [0.9.13](https://github.com/loonghao/vx/compare/v0.9.12...v0.9.13) (2026-06-02)


### Features

* support legacy python and ai skills checks ([e8a60c5](https://github.com/loonghao/vx/commit/e8a60c52b5a246c745b49ea0a9e2d34d845cba3b))

## [0.9.12](https://github.com/loonghao/vx/compare/v0.9.11...v0.9.12) (2026-05-31)


### Bug Fixes

* **deps:** update rust crate windows-sys-73dcd821b1037cfd to 0.61 ([9eb4fe3](https://github.com/loonghao/vx/commit/9eb4fe31f745f8d4b163bd31457deebc827abf89))


### Documentation

* fix typo in GEMINI.md (swtich → switch) ([60b3c17](https://github.com/loonghao/vx/commit/60b3c177251f5a2e90eb333e8c881aef6193bac7))
* update outdated documentation references and add metrics-analysis ([3d52403](https://github.com/loonghao/vx/commit/3d524039a7d66a7b39f038c8698f6f0bd7bfa000))
* update version references and documentation indexes (2026-05-30) ([d7ce068](https://github.com/loonghao/vx/commit/d7ce0680187a89d2b3e8de277ed4a7c648e438c2))

## [0.9.11](https://github.com/loonghao/vx/compare/v0.9.10...v0.9.11) (2026-05-28)


### Bug Fixes

* pass through cargo install path ([bdd4dc2](https://github.com/loonghao/vx/commit/bdd4dc2baab37ee25c0fbe52fd4797416a380ada))
* skip cargo external source installs ([22cafd5](https://github.com/loonghao/vx/commit/22cafd5c425e7894aefa50ab167bbecb014b06b4))

## [0.9.10](https://github.com/loonghao/vx/compare/v0.9.9...v0.9.10) (2026-05-28)


### Bug Fixes

* **metrics:** improve statistics display and align CLI help with contract ([a4f7156](https://github.com/loonghao/vx/commit/a4f71563a7698639eb0e0b9c354d4942f09e0e3e))

## [0.9.9](https://github.com/loonghao/vx/compare/v0.9.8...v0.9.9) (2026-05-27)


### Bug Fixes

* keep metrics trace collection under rust log ([362800e](https://github.com/loonghao/vx/commit/362800ef30bb3810e488f0c26cfc0a1daf6c2d55))

## [0.9.8](https://github.com/loonghao/vx/compare/v0.9.7...v0.9.8) (2026-05-27)


### Features

* add 11 new providers (mise, gitleaks, biome, lazydocker, k9s, gping, watchexec, duf, trippy, sd, actionlint) ([a6abcab](https://github.com/loonghao/vx/commit/a6abcab83d78bc2d04d9d8dc6df0437ad0aef8ba))
* add 7 high-priority developer tool providers (lazygit, delta, hyperfine, zoxide, atuin, chezmoi, eza) ([3ec51ef](https://github.com/loonghao/vx/commit/3ec51efb29084be123c450f46b34b90e51ab2be4))
* add 7 new providers (tealdeer, dust, xh, bottom, trivy, zellij, dive) ([5bc63ab](https://github.com/loonghao/vx/commit/5bc63ab4b608bee61bfb9140cb0aa63c13ffe162))
* add age and sops providers, update project analyzer frameworks ([498a161](https://github.com/loonghao/vx/commit/498a161c067bfc1d1460fa8a20b6eed0890faf1e))
* add build cache providers (sccache, ccache, buildcache) ([c518f90](https://github.com/loonghao/vx/commit/c518f90862058a8b43cf79e98e380b15b8d6c7e7))
* add dynamic package_prefixes from provider.star metadata ([6d8a197](https://github.com/loonghao/vx/commit/6d8a1973ea2aac611cffd7de547b9d71acc74530))
* add FilterLevel enum (Light/Normal/Aggressive) for compact output ([#804](https://github.com/loonghao/vx/issues/804)) ([677e31d](https://github.com/loonghao/vx/commit/677e31dbc2f3a3180f70e2b4f3be9e42aaa42e6a))
* add helix and yazi providers ([b62985b](https://github.com/loonghao/vx/commit/b62985b6d19d11c6c971d64fea8f8f2e10fcb8c2))
* add llvm/conan/xmake/wix providers and enhance msvc/msbuild for C++/C# build automation ([adb9720](https://github.com/loonghao/vx/commit/adb97206b2dee7eb4b8281ebae5022d9e6f1c582))
* add new oneshot runner ecosystems and update CLI help/docs ([fcccbdd](https://github.com/loonghao/vx/commit/fcccbddd6faf7a72cab081b2e3dae18f418df74a))
* add Nx and Turborepo cache providers, fix CI sccache issue ([30954da](https://github.com/loonghao/vx/commit/30954da54563a41f7aa21824de36485784a632d3))
* add python 3.7 and wasm runtimes ([19b9a56](https://github.com/loonghao/vx/commit/19b9a56c82e5cb6d2bc7b0da2f222c643c76def4))
* add rust wasm providers ([4924679](https://github.com/loonghao/vx/commit/4924679f903ccc5dc92cadd8302ce8ad644ce53f))
* add self-update check with non-blocking notification ([8429303](https://github.com/loonghao/vx/commit/84293034ef46fdfbd6f07fb52575f447c45cf20a))
* add starlark provider support with bash, curl, meson, openssl, pre-commit, release-please, rez, systemctl, xcodebuild providers ([cd50d2f](https://github.com/loonghao/vx/commit/cd50d2fb07ea554e759f831fa3d6bfd491fa1e8a))
* add vx skill for ClawHub publication ([#638](https://github.com/loonghao/vx/issues/638)) ([42e2dc7](https://github.com/loonghao/vx/commit/42e2dc7d44cbab95121e2435f83524e7ae68105f))
* add vx-output-filter crate for compact subprocess output ([#802](https://github.com/loonghao/vx/issues/802)) ([6a6369c](https://github.com/loonghao/vx/commit/6a6369cfd91b2916c1e2685611cb2a865153a066))
* add well-known Python version fallback for python-build-standalone ([3d81387](https://github.com/loonghao/vx/commit/3d81387085e8c5abbd1d5094b52407efc2dccb6c))
* add witr provider (137 providers total) ([e6ec476](https://github.com/loonghao/vx/commit/e6ec476b321717d9887e4a5b069c03cc19c6f9b1))
* add worktrunk provider (132 tools total) ([#839](https://github.com/loonghao/vx/issues/839)) ([708f4ee](https://github.com/loonghao/vx/commit/708f4ee62b26007ad241eb277f7d9326885eb896))
* **ai:** implement RFC 0035 AI integration optimization ([595f77f](https://github.com/loonghao/vx/commit/595f77f059e4587ceb550b310454840f10417711))
* **auto-improve:** squash merge auto-improve branch ([bbddae5](https://github.com/loonghao/vx/commit/bbddae562447a85fdc6b52137d005a2e690b175e))
* bridge global install commands to vx package shim workflow ([9df5c6c](https://github.com/loonghao/vx/commit/9df5c6c9170c19357d9f5d02c71fe2413840935c))
* **build:** add rust-lld linker for faster builds (RFC 0032 Phase 1) ([7187439](https://github.com/loonghao/vx/commit/7187439a784eb18bebf2b64d67a28303879cc7e1))
* **build:** add vx-runtime-core and vx-runtime-archive to workspace dependencies ([555d405](https://github.com/loonghao/vx/commit/555d4054108e58428c581f58bc3d2c69c4083a58))
* **build:** create vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([dc0e646](https://github.com/loonghao/vx/commit/dc0e646421e449d9c6f964a2374fa3b1c95f9797))
* **build:** integrate vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([7077ba2](https://github.com/loonghao/vx/commit/7077ba273e8ec65d21cad44e9d1582546fff26d3))
* change non-TTY default output from JSON to Toon, disable CDN by default ([c698843](https://github.com/loonghao/vx/commit/c6988430cd54c53b807e44c8efbd45d38539fe3e))
* **cli:** add update channel support (stable, beta, dev) ([84c8220](https://github.com/loonghao/vx/commit/84c8220010e852f6bf63b36576c2e86d49d03681))
* **cli:** Agent DX improvements for AI agents ([1e5ac9d](https://github.com/loonghao/vx/commit/1e5ac9db00d3501b8f8ba124800ef99751679869))
* **cli:** bridge global install commands to vx package shims ([c3ff97d](https://github.com/loonghao/vx/commit/c3ff97dec4d429f961c3ed3f7d4584bec8a99e1a))
* **cli:** enable direct global command shims in vx bin dir ([b8d03e9](https://github.com/loonghao/vx/commit/b8d03e905227de3327d34ee2a2aada7d4008295e))
* **cli:** overhaul vx add/remove with format-preserving edits ([d5bd786](https://github.com/loonghao/vx/commit/d5bd7869e2ca80f100f2ad13786cd795a281eaa8))
* **ecosystem_aliases:** route ecosystem:package to dedicated provider binary ([db0b0cd](https://github.com/loonghao/vx/commit/db0b0cd627fae159d1ca1dc372446264033d835f))
* **hooks:** upgrade cargo-hakari pre-commit hook to auto-fix mode ([3f30652](https://github.com/loonghao/vx/commit/3f3065274840def26a2c796ed3460728f141a694))
* land provider and CLI improvements ([2c8e2fd](https://github.com/loonghao/vx/commit/2c8e2fdc74f72ee10d17bad9cf2b88f769a99115))
* **list:** sort tool list alphabetically (a-z) ([9f18585](https://github.com/loonghao/vx/commit/9f18585ec4c38d25ed6a05c5dc4a624a89543c24))
* propagate explicit version from bundled runtime to parent dependency ([#766](https://github.com/loonghao/vx/issues/766)) ([788a0d2](https://github.com/loonghao/vx/commit/788a0d209969080105c8ff30920e8cb2e402fc04))
* **providers:** add cargo-audit provider ([6f1b1b9](https://github.com/loonghao/vx/commit/6f1b1b98e48b17eaebb30c9425f3d2ff1c8c3c70))
* **providers:** add cargo-nextest and cargo-deny providers ([fa4b184](https://github.com/loonghao/vx/commit/fa4b184968aaf528de4bbe5b2403f001cbe0a803))
* **providers:** add conda provider with micromamba, conda and mamba ([c7e6013](https://github.com/loonghao/vx/commit/c7e60139cafed699786db1cc346ce93a6d599afe))
* **providers:** add conda provider with micromamba, conda and mamba ([aaeba19](https://github.com/loonghao/vx/commit/aaeba19b029d006b6db29ddf4db5f3e8b0bfaed6)), closes [#389](https://github.com/loonghao/vx/issues/389)
* **providers:** add grpcurl provider + update provider count to 114 ([356dc63](https://github.com/loonghao/vx/commit/356dc63088b922e02469bf84a34e47b62a0a3482))
* **providers:** add kind and k3d providers ([7b9ac2f](https://github.com/loonghao/vx/commit/7b9ac2f506856d2c8fa4e5c901190efb6eb44a95))
* **providers:** add tokei provider + triage stale issues ([88a8b0e](https://github.com/loonghao/vx/commit/88a8b0e6c27acb39132d733ee920a6423fc2f871))
* **resolver:** implement version priority with vx.lock support ([9fe8714](https://github.com/loonghao/vx/commit/9fe8714c1663fbedb9818381e85d89e0939b2e9a))
* **rfc-0037:** implement ProviderHandle unified facade for CLI commands ([cb2b65a](https://github.com/loonghao/vx/commit/cb2b65af1bca2b1a5d2b92ec21016bafb079dae4))
* **rfc-0040:** implement version_info() for toolchain version indirection ([126fbb1](https://github.com/loonghao/vx/commit/126fbb1d22b1c3b184a92dc511a8c3bc77e31ba1))
* **routing:** prefer dedicated provider over cargo install for ecosystem:package ([392ccd3](https://github.com/loonghao/vx/commit/392ccd3a8403179ccc36ac732efe25c1c0533963))
* **skills:** add vx-agent-workflow skill for token-efficient command execution ([86cd6bd](https://github.com/loonghao/vx/commit/86cd6bd79d31eea0f2602c218da88706b9cdf91e))
* **starlark:** add github.star stdlib + jj provider.star migration ([1d8802f](https://github.com/loonghao/vx/commit/1d8802f0553bb04b1c365b4a92a8877683fa8b1c))
* **starlark:** complete provider.star migration and fix stdlib ctx access ([965e8bf](https://github.com/loonghao/vx/commit/965e8bfebf13b88ca9c69d083e4f4cbad49c7ded))
* **tests:** add comprehensive Python provider e2e tests ([6c96bc5](https://github.com/loonghao/vx/commit/6c96bc5290dacaa0ac48a1f777b7961f79839e7f))
* track output token savings ([1fcdc83](https://github.com/loonghao/vx/commit/1fcdc835dd2ff25b29dc5ce3c66dea80eadb8802))
* **vx-starlark:** implement Phase 2 Starlark execution engine ([6cd99f8](https://github.com/loonghao/vx/commit/6cd99f83bd0f1353dc7befd3b4c93fe265ea82ec))
* **vx-starlark:** Phase 2 - integrate starlark-rust execution engine ([fae7bc7](https://github.com/loonghao/vx/commit/fae7bc77c87cf298891cff18825910fbcda47053))
* wire provider dynamic deps and fix install routing ([ec4cccf](https://github.com/loonghao/vx/commit/ec4cccf8aba470771c2e6572d1a96f720d233712))


### Bug Fixes

* **7zip:** fix executable name and system_paths to point to binary file ([4f55b26](https://github.com/loonghao/vx/commit/4f55b267b903abeb608f71eb2f6b743de569a34f))
* Add 'if: !startsWith(github.event.head_commit.message, chore: release)' guards to skip these workflows when the push is a release commit. ([7874981](https://github.com/loonghao/vx/commit/787498183f4bb5062c5b54edb7f6d77efbb7521a))
* add a ound flag so END block only prints when the match block did not. Also add head -1 safety to ensure only one line is captured. ([d756244](https://github.com/loonghao/vx/commit/d75624428f8e9c2ef562d14d86c7c3549c683fe1))
* add is_version_installable and prepare_execution for bundled runtimes ([c2b5704](https://github.com/loonghao/vx/commit/c2b570409e6274a3e4a9c6d10de8de1aa17f5c9c))
* add recursive search for bundled executables and remove wrong fallback ([90eb628](https://github.com/loonghao/vx/commit/90eb6284b817409b20fcd438db6c6efa3b52899e))
* add workspace-hack deps and remove invalid CI cache parameter ([60cf870](https://github.com/loonghao/vx/commit/60cf870de5ebcce245d05ac02163db0d2192aa48))
* address provider CI regressions ([035af34](https://github.com/loonghao/vx/commit/035af34603085b5b285df12719f9fa5b7f2653f6))
* **ai:** fix skills format and install all 5 skills on setup ([ec1af7c](https://github.com/loonghao/vx/commit/ec1af7c2f92fea97b430a7e251391fdb6743fc68))
* align starlark mock signatures with stdlib and fix provider tests ([88c16b2](https://github.com/loonghao/vx/commit/88c16b22618d8f4a6cfb1516cebe7b26ebbf8ceb))
* **all:** comprehensive CI fixes and Rust 1.95.0 upgrade ([#843](https://github.com/loonghao/vx/issues/843)) ([8ab4222](https://github.com/loonghao/vx/commit/8ab42220b56d59d26d8e4b795c1d6b876bbf6070))
* auto-disable Spectre mitigation in MSBuild bridge when libs are missing ([81cd697](https://github.com/loonghao/vx/commit/81cd697c2da732ea5537bb23032345a7ca79e596))
* auto-fetch versions when version_date cache miss in download_url ([d7223c2](https://github.com/loonghao/vx/commit/d7223c218c296141ded07998f6b4c3116ee7d15b))
* avoid msvc repair for unrelated commands ([0be0dd9](https://github.com/loonghao/vx/commit/0be0dd90388897cdfa125e53d88c79c64fa3211b))
* **cache:** skip NeedsInstall results in resolution cache; extend TTL to 24h ([7620f77](https://github.com/loonghao/vx/commit/7620f779a662b6d472ab2cebb2dd68dc836a6bba))
* **cargo-audit:** remove unused rust_triple import (lint) ([f21bba5](https://github.com/loonghao/vx/commit/f21bba5bd62117c4ce76bf293d71ca9e69a3804d))
* change internal rustup/toolchain debug logs to trace level ([3744ee5](https://github.com/loonghao/vx/commit/3744ee5facb0e1f6d6b6a0052a7fe065f67a4860))
* **ci:** add sccache setup to all CI workflows ([d9ecba7](https://github.com/loonghao/vx/commit/d9ecba7fd31eac7f764ac0a9f1cfffcc3d0a3cac))
* **ci:** add sccache setup to benchmark workflow ([d67e676](https://github.com/loonghao/vx/commit/d67e67633eb1bb815148473f7953db045aaf6423))
* **ci:** ensure Release workflow triggers even when release is created from update-pr job ([a94709c](https://github.com/loonghao/vx/commit/a94709c5fed8953e58a387412aed5d48e903c421))
* **ci:** exclude vx-msbuild-bridge from cargo-dist & improve skills sync ([fb4503d](https://github.com/loonghao/vx/commit/fb4503dd789b1f95168556c29119609d537f3e6f))
* **ci:** fix discovery parser and CI skip list for Linux/macOS failures ([f2e7632](https://github.com/loonghao/vx/commit/f2e763210948e524dfbce89015a0c78aa6f72ea4))
* **ci:** handle skipped/cancelled jobs in CI Success gate ([2c69ec0](https://github.com/loonghao/vx/commit/2c69ec0c48482d2ac5abb3bc99c38df011030109))
* **ci:** improve sccache path handling on Windows ([b960cb0](https://github.com/loonghao/vx/commit/b960cb063dbcd2f7362a6389f94fbe39460e45f2))
* **ci:** include Cargo.lock in workspace-hack commit step ([62ebb4a](https://github.com/loonghao/vx/commit/62ebb4abcf0c7c3b1b7bc1c909a2555d75c532f1))
* **ci:** increase Windows timeout to prevent CI failures ([217e2e4](https://github.com/loonghao/vx/commit/217e2e43fac7a3764d977b460f65af783478b11e))
* **ci:** install sccache in quick-test job ([32819ad](https://github.com/loonghao/vx/commit/32819addbd1ed977921df4207cc1a4e73395a258))
* **ci:** prevent duplicate release-please PRs on release merge ([7874981](https://github.com/loonghao/vx/commit/787498183f4bb5062c5b54edb7f6d77efbb7521a)), closes [#713](https://github.com/loonghao/vx/issues/713)
* **ci:** remove lld linker on macOS due to compatibility issues ([17bf47f](https://github.com/loonghao/vx/commit/17bf47f40901335697ec39330d332d6577194bb7))
* **ci:** remove max-versions-to-keep from winget-releaser ([87ed136](https://github.com/loonghao/vx/commit/87ed136d8092abbb6cfd650557ba01c9274bf8d6))
* **ci:** replace curl POST with clawhub CLI in sync-skills workflow ([2d09162](https://github.com/loonghao/vx/commit/2d0916271bd605bd7ef4856051dce100e42e854a))
* **ci:** replace dorny/paths-filter with native git diff ([7b0db6e](https://github.com/loonghao/vx/commit/7b0db6e0a0bca8bb9f2a32479b08fec24492d674))
* **ci:** replace remaining vx run cargo scripts with direct vx cargo calls ([c3185a2](https://github.com/loonghao/vx/commit/c3185a231a5d4670caba1b494c11be437cca5998))
* **ci:** resolve required check name conflict blocking PR merges ([2c9def5](https://github.com/loonghao/vx/commit/2c9def5fa941d0b31644377f097798d5b115a551))
* **ci:** sanitize provider cache keys ([355f143](https://github.com/loonghao/vx/commit/355f14334b6bdd459dfd7dfa3332dbcda4145e50))
* **ci:** skip wix and xmake in CI tests ([90a6bcd](https://github.com/loonghao/vx/commit/90a6bcd20867c2ae1a37371a479d3e3a3eef2dfc))
* **ci:** split release-please into two jobs to fix tag creation ([3eebf8b](https://github.com/loonghao/vx/commit/3eebf8b16f35dc40ab897efd95fbcdd3b0d675bd))
* **ci:** switch apt mirror to azure.archive.ubuntu.com for cross builds ([b8f3097](https://github.com/loonghao/vx/commit/b8f3097086f6c3df0219397d6547c0acc58ab61b))
* **ci:** use system cargo directly instead of vx cargo ([1f5917a](https://github.com/loonghao/vx/commit/1f5917ae75cbbb0bfcefc6f6ee5b184571fc9ef7))
* **ci:** use vx cargo prefix in justfile recipes for CI compatibility ([6a5e272](https://github.com/loonghao/vx/commit/6a5e27225009680694120cc50a7bc1317411d320))
* **cleanup:** fix compile errors from ecosystem_aliases feature ([01e55cd](https://github.com/loonghao/vx/commit/01e55cd374dececd4659268d90f0ca99e6a83e26))
* **cli:** add --toon shortcut flag for TOON output format ([b2c4f9b](https://github.com/loonghao/vx/commit/b2c4f9b87a02575e40a8f5d34ecd196aed43b855))
* **cli:** fix Clippy warnings and test compilation errors ([0b2ea2d](https://github.com/loonghao/vx/commit/0b2ea2d01aa73883c6eecdfdd65d4c8672d22d2b))
* **cli:** fix formatting issues (run cargo fmt) ([302d2e6](https://github.com/loonghao/vx/commit/302d2e606726eedf89c15fdc7841d575266e5abc))
* **cli:** fix vx check system_fallback and vx lock for installed tools ([3d6087a](https://github.com/loonghao/vx/commit/3d6087a513ff89c8fcb65dd0f1082db773bc34ca))
* clippy useless_vec warnings in tests ([9ff0999](https://github.com/loonghao/vx/commit/9ff099999fc2e679919863f141ae2a78a8b2ed58))
* **cli:** remove debug print from lib.rs ([613a2f6](https://github.com/loonghao/vx/commit/613a2f68e728a0180fe73b7fca02c8bbbd5c4765))
* **cli:** send update notifications to stderr ([35fb5b9](https://github.com/loonghao/vx/commit/35fb5b9c35ae33d47542f8dd4f8c66269d70e7ae))
* **console:** use eprintln for progress output to avoid stdout contamination ([c7c576c](https://github.com/loonghao/vx/commit/c7c576c9ea8365be27a2b4c78b304888e0813596))
* correct cmake macOS download URL and jq environment key ([851255e](https://github.com/loonghao/vx/commit/851255e38711a3ab965c448c8a98ed78897e567b))
* correct download URLs for grpcurl, k3d, kind, and duckdb providers ([29556f6](https://github.com/loonghao/vx/commit/29556f6dc0eec544934f65718cfcfccae68e61e5))
* correct metrics calculations ([b82cb64](https://github.com/loonghao/vx/commit/b82cb641e80eb106888e3c8ba5bd6d9b9e6ea8c8))
* correct RFC count (40+ -&gt; 50+) and update Rust version badge (1.93+ -&gt; 1.95.0+) ([#879](https://github.com/loonghao/vx/issues/879)) ([9bba76e](https://github.com/loonghao/vx/commit/9bba76ebe89a723902a268f8b3a9ad157cd05d4a))
* **deps:** update rust crate anstream to v1 ([7634188](https://github.com/loonghao/vx/commit/76341885c47e9f0e5b5aac40c2996c2154416814))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 ([07124de](https://github.com/loonghao/vx/commit/07124de48e983b7af2b06f03fc83001924306ec5))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 - abandoned ([004a57b](https://github.com/loonghao/vx/commit/004a57bff6645a0eb9969993d5e4ec0c0cf2db07))
* **deps:** update rust crate starlark_derive to 0.14 ([040fae8](https://github.com/loonghao/vx/commit/040fae8cac092d6f68bde915f0bb2cdfc7dbe2ed))
* **deps:** update rust crate toon-format to 0.5 ([ff851ca](https://github.com/loonghao/vx/commit/ff851ca0e8671fa4fb1e5befc785342b67cd1760))
* **dist:** exclude vx-star-metadata from cargo-dist release artifacts ([b9cc621](https://github.com/loonghao/vx/commit/b9cc621500475f4a433ffa8c0c1a02f9e38bf091))
* **docker:** switch apt mirror to azure.archive.ubuntu.com in Dockerfiles and test workflow ([e6afc73](https://github.com/loonghao/vx/commit/e6afc731f2a54e11331936662f6a0ad22b29fcfa))
* **docs:** fix broken doctests in vx-console and vx-starlark ([6d17206](https://github.com/loonghao/vx/commit/6d172061702fc97d1252c2f188825b873978ef79))
* **engine:** ctx.install_dir now points to actual install location ([e78a652](https://github.com/loonghao/vx/commit/e78a652da73930845ad12fb0648ffb630711d0e6))
* ensure rust targets are installed in CI ([e09a6cb](https://github.com/loonghao/vx/commit/e09a6cb5805e90f53ecb84808b3d0e8d5cb6eb27))
* exclude vx-star-metadata from cargo-hakari workspace-hack ([43b398d](https://github.com/loonghao/vx/commit/43b398dc0fb9c782788fbb41fe37bbb53c41c980))
* **eza:** add platform_constraint to skip macOS in CI tests ([1d49a16](https://github.com/loonghao/vx/commit/1d49a169e1131f41a60977c16c944a315693ef6d))
* ffmpeg use Gyan.dev mirror, witr only override download_url ([a0de490](https://github.com/loonghao/vx/commit/a0de4906990114757294fbb4191658d1ac7127b8))
* **ffmpeg:** use system_install only (remove unreliable GitHub downloads) ([4fdd7da](https://github.com/loonghao/vx/commit/4fdd7daa0e2dde50f2944ff58ff1f38ffedfb9ee))
* **ffmpeg:** use vx-org/mirrors with BtbN static builds (win64+linux64+linuxarm64) ([39542e3](https://github.com/loonghao/vx/commit/39542e373f72693f95483e0bbaf0b386c91b804c))
* filter vault releases by platform artifacts ([b9205de](https://github.com/loonghao/vx/commit/b9205de87c9971c69649750fd53d5cd5da1527b0))
* fix PSReadLine cursor positioning issue in PowerShell prompt ([1a5d995](https://github.com/loonghao/vx/commit/1a5d995a74f49cd39061b41c79dc42ad81c80c14))
* fix system_install providers and starlark test assertions (round 5) ([cabfdeb](https://github.com/loonghao/vx/commit/cabfdeb4fefe2bfe59d5322279ba240fbe3d23e5))
* fix workspace-hack hakari section markers and regenerate dependencies ([7483f3f](https://github.com/loonghao/vx/commit/7483f3f60ed2f1279d59abacd59e3c592cc3e62e))
* flatten InstallLayout JSON so manifest_runtime can read strip_prefix ([71bc81d](https://github.com/loonghao/vx/commit/71bc81d5ab27b4af0db0aa76f2cf08ee84d616b9))
* **gcloud:** update starlark test to use __type field ([9db3a9c](https://github.com/loonghao/vx/commit/9db3a9c45ef97904b53690ef70b99b3648f8ae4c))
* git uses MinGit ZIP on Windows, rust toolchain defaults to stable ([7a18c22](https://github.com/loonghao/vx/commit/7a18c22ec1efbe48e2a90333d05a2be0f954dd32))
* git Windows exe path, rust bundled store mismatch, lock multi-platform URLs + perf optimizations ([#787](https://github.com/loonghao/vx/issues/787)) ([8806adc](https://github.com/loonghao/vx/commit/8806adc38293365f808f81a83461530377c03c75))
* gracefully resolve numeric version hints for pure opaque providers ([40b580f](https://github.com/loonghao/vx/commit/40b580fee471b6efad7832ce42c07b2241bbb883))
* hadolint asset name separator and uvx bundled_with support ([c2e0dd9](https://github.com/loonghao/vx/commit/c2e0dd9c3b2e018f6ae2ed34fbcf6ebfdeb5db2b))
* handle JSON output in where command e2e tests ([c5f7dbe](https://github.com/loonghao/vx/commit/c5f7dbe597b918ab6e935be001f7e67cd75de848))
* handle rust toolchain versions in path selection ([9717a40](https://github.com/loonghao/vx/commit/9717a403531f62523f7863a83112a6d311a284e7))
* handle VX_VERSION=latest in install scripts ([5d30f4a](https://github.com/loonghao/vx/commit/5d30f4ad21236f34f3d2e3e912eba8470a580f91))
* harden release and runtime installs ([9ea29a6](https://github.com/loonghao/vx/commit/9ea29a605e2585ead9a09a489989c0a538c6f187))
* harden release workflow reruns ([70c7871](https://github.com/loonghao/vx/commit/70c78712dad221e7644a854b843cacf36be91547))
* harden release-please and add mcpcall ([5523346](https://github.com/loonghao/vx/commit/5523346e105df92c3fc480561e56feb2d31f025e))
* import env_prepend from env.star instead of provider.star ([0a8503f](https://github.com/loonghao/vx/commit/0a8503fffaf8415d3391861e3265aa7a3c203ef6))
* improve installer fallback and mirror release support ([84e8e35](https://github.com/loonghao/vx/commit/84e8e3597082fa27de2f7f1b52f6c7fdeb4387f5))
* inject parent runtime env for bundled runtimes (npm/node PATH issue) ([f39d51a](https://github.com/loonghao/vx/commit/f39d51a546bcf5d130ae59172273c1b00923807c))
* inject parent runtime PATH for bundled runtimes via spec env_config ([d05e779](https://github.com/loonghao/vx/commit/d05e7796e77848e0ccf6dca86f53e43b965d593f))
* **installer:** apply binary rename fix to RealInstaller.download_with_layout ([b3dcdda](https://github.com/loonghao/vx/commit/b3dcddab21d564fffe6bb6aec47417af36bd9a96))
* **installer:** fix PortableGit .7z.exe not recognized as archive + stop version fallback on layout errors ([fb6d128](https://github.com/loonghao/vx/commit/fb6d12807decf59fc51c7177a4799091ae859803))
* **install:** prevent awk double-output in resolve_latest_version ([d756244](https://github.com/loonghao/vx/commit/d75624428f8e9c2ef562d14d86c7c3549c683fe1))
* **install:** skip releases without binary assets in version resolution ([46553eb](https://github.com/loonghao/vx/commit/46553eb6b34c52f1d0ae71fe4d846ead43ded6aa))
* **just:** correct version_pattern to match 'just X.Y' output ([5238a21](https://github.com/loonghao/vx/commit/5238a21c27605e76593f7f03ad0890f8472356b6))
* **justfile:** fix test-pkgs recipe to not duplicate -p flag ([6bc8aea](https://github.com/loonghao/vx/commit/6bc8aeaefe58512e027f647896ed15024cfb98d0))
* **lint:** resolve provider.star lint issues ([6166bb3](https://github.com/loonghao/vx/commit/6166bb32e5fc7bf47c53f59a1fb2b8e8445fd665))
* **macos:** make sevenz-rust optional to fix macOS build ([6f7d846](https://github.com/loonghao/vx/commit/6f7d8461d72aa4b53d4241d1d54d0dfb31c225b3))
* make E2E version list tests resilient to transient network errors ([2367e94](https://github.com/loonghao/vx/commit/2367e940b9832a249a7b7d5243c20407a6ee16ea))
* make env-dependent tests serial to prevent race conditions ([44974bd](https://github.com/loonghao/vx/commit/44974bd13ce4fa4405243c895e3707a6221b33b8))
* **manifest-runtime:** override resolve_version to return 'system' for system tools ([ced6833](https://github.com/loonghao/vx/commit/ced6833db66c2b32e6479a2e8a29b0a741f5fc46))
* **mise:** avoid strip_prefix on Windows to prevent Access Denied errors ([b1958c9](https://github.com/loonghao/vx/commit/b1958c94bfa3031f2e8edc6f772661a4607da633))
* **mise:** update unit tests to match new install_layout implementation ([3f7d91b](https://github.com/loonghao/vx/commit/3f7d91b549378eb4684157d7056ae1e611076bfa))
* **mise:** use strip_prefix='mise/bin' on Windows to avoid shim detection error ([1703d7f](https://github.com/loonghao/vx/commit/1703d7f416e0e7664ad5ba3cb4f4916f520f9144))
* **paths:** detect unified runtime store versions ([35e16cc](https://github.com/loonghao/vx/commit/35e16cc8bfd688e5a72ac08a98dae7bdd23e7920))
* prepend node_modules/.bin to PATH for npm/npx execution ([#906](https://github.com/loonghao/vx/issues/906)) ([a2b7ea2](https://github.com/loonghao/vx/commit/a2b7ea22678665afe030e36ed1e22da3892f8fde))
* preserve Rust MSRV in vx.toml and enable passthrough for Rust ecosystem ([bec646a](https://github.com/loonghao/vx/commit/bec646a7ac32be27a4ffb9abbbc9e325edbe448f))
* prevent bundled runtime executable misresolution (npm-&gt;node) ([0f50874](https://github.com/loonghao/vx/commit/0f50874619de31e7b9672cf42afc1313c1b9f6ac))
* prevent repeated MSVC component re-installation when Spectre libs unavailable ([94363bf](https://github.com/loonghao/vx/commit/94363bff59b36cfb42d6fa76c247c7c042f458c5))
* propagate locked version to bundled runtime dependencies ([b3a889b](https://github.com/loonghao/vx/commit/b3a889b728959502d690fe03a5a5bcb38df9463c))
* **provider:** correct grpcurl version check ([28e7de8](https://github.com/loonghao/vx/commit/28e7de8d5ff228d008fbd56a5e5fff061d89ca66))
* **providers:** add fetch_versions_with_tag_prefix to layout mock + fix cargo-deny Windows ([ce5c171](https://github.com/loonghao/vx/commit/ce5c171a48056a2a5c0384f9dee6239804b487eb))
* **providers:** correct install_layout strip_prefix and download_url ([45589b8](https://github.com/loonghao/vx/commit/45589b888f561b6f6a586887bf688caad7a08b60))
* **providers:** correct mirror tag version fetching ([248b2fc](https://github.com/loonghao/vx/commit/248b2fc52795e4c0d198d385f6556b44f3772d66))
* **providers:** fix 5 more provider bugs (round 2) + add tar.bz2 support ([c89565b](https://github.com/loonghao/vx/commit/c89565b08ea48eafc2fc671e4d1b9f8b0d36d81c))
* **providers:** fix 5 provider bugs from auto-improve branch ([96dbf55](https://github.com/loonghao/vx/commit/96dbf55ed4e9d0f04cd54a342846e5182fa899e9))
* **providers:** fix binary rename, grpcurl macOS, duckdb macOS, nerdctl platform ([dff7141](https://github.com/loonghao/vx/commit/dff714103d0256cfd1168ad26bde56b8db1ede8d))
* **providers:** fix cargo-nextest macOS triple and cargo-audit test assertions ([21fe463](https://github.com/loonghao/vx/commit/21fe46397f0cc1f91a3ca2083b352e753f97b704))
* **providers:** fix download URL bugs in git, xmake, and ollama ([#777](https://github.com/loonghao/vx/issues/777)) ([c5efbb5](https://github.com/loonghao/vx/commit/c5efbb599a9363fdd54086981c30e3f00d25c2ec))
* **providers:** fix download URLs for cargo-audit, cargo-nextest, and deno ([28af75a](https://github.com/loonghao/vx/commit/28af75a9eaf79c6ea435cb44f4481d4ffd36476d))
* **providers:** fix dust and eza macOS download URL 404 ([ee5bc77](https://github.com/loonghao/vx/commit/ee5bc779adafd69965ba7c87b8cd7ece198b99fc))
* **providers:** fix dust version pattern and tealdeer binary rename ([eb7c3f6](https://github.com/loonghao/vx/commit/eb7c3f6fde473e3d71bb9ad788119ed27d6e26b1))
* **providers:** fix gcloud get_execute_path and terraform fetch_versions ([f0b5d9c](https://github.com/loonghao/vx/commit/f0b5d9cf38c71ad3d2c07214bd83e15782ec187b))
* **providers:** resolve CI download URL failures ([9067375](https://github.com/loonghao/vx/commit/906737558ab53cf9312018b9d603929e4a82f634))
* **providers:** resolve CI issues for new provider batch ([8aa8967](https://github.com/loonghao/vx/commit/8aa89675695d32f212e77d07f811439bf60dbb29))
* **providers:** resolve tealdeer and mise install layouts ([3c2d42a](https://github.com/loonghao/vx/commit/3c2d42a1a2fd72d3ac3759fca6b5c35c9259c214))
* **providers:** use vx-org/mirrors for ffmpeg and witr downloads ([e9d6fc3](https://github.com/loonghao/vx/commit/e9d6fc346d8e02553313106ead2356ef2efb9dfd))
* **provider:** use gnu rustup triples on linux ([2b9e217](https://github.com/loonghao/vx/commit/2b9e217762a6f6561e3b5340a6dd557b307686aa))
* Python install fails due to version_date cache key mismatch ([2edb836](https://github.com/loonghao/vx/commit/2edb836bbc616e521d90863d0d4fb68f1a72fbad))
* **release:** disable sccache rustc-wrapper in release workflow ([248bfdf](https://github.com/loonghao/vx/commit/248bfdfc3084524c0b9ad731ca95f66b3502f16e))
* remove BOM from all provider.star files and improve star syntax checker ([75f14df](https://github.com/loonghao/vx/commit/75f14dfe8d2683624a7e355c5b5390fd856c14a9))
* remove platform subdir from install path, fix providers ([ad05502](https://github.com/loonghao/vx/commit/ad05502e99de01c927447c88f7f782c0b3fc52da))
* remove unused loads and fix lint issues in provider.star files ([cf28c19](https://github.com/loonghao/vx/commit/cf28c190de986994b0496aa6e4228c42da196b7b))
* remove unused variables in witr/provider.star ([93f8dad](https://github.com/loonghao/vx/commit/93f8dadb9ee4665ea7809f248261c30348cf53bb))
* remove windows-sys 0.59 and merge features into 0.61 in workspace-hack ([336e2d8](https://github.com/loonghao/vx/commit/336e2d8d7d072a852e91d2fdb7c633d389fc4750))
* repair provider test resolution and platform gating ([657467e](https://github.com/loonghao/vx/commit/657467eafa78be1adc19ffd21e9e743e4f58ee96))
* repair vx store executable permissions ([f0d57e5](https://github.com/loonghao/vx/commit/f0d57e578eb9a1c3889de3798df7a503004a8ef5))
* replace all ctx dict access with struct attribute access in provider star files ([163be8c](https://github.com/loonghao/vx/commit/163be8c21828ed3dc18b90012d1ab7268d80aa27))
* replace all ctx.http.get_json with fetch_json_versions descriptors in provider.star files ([7f9238c](https://github.com/loonghao/vx/commit/7f9238ce667e793fead37dc999dcf927439c2f70))
* resolve .cmd executables for bundled runtimes on Windows ([bd12254](https://github.com/loonghao/vx/commit/bd122542ed0944b379aceda04a4d4f7279c813eb))
* resolve CI errors ([0a30bed](https://github.com/loonghao/vx/commit/0a30bed957293bdb745255fbf4f16318f3cbb91d))
* resolve CI errors ([454cb9b](https://github.com/loonghao/vx/commit/454cb9bed0b08d531926875a61561b7e4eff8a6c))
* resolve CI failures for imagemagick, ffmpeg, rez, bash, make, nasm ([5880381](https://github.com/loonghao/vx/commit/5880381eb13b0767b186d3a9942c04ec2c30c05b))
* resolve CI failures for lefthook, grpcurl, and kustomize providers ([04be2d9](https://github.com/loonghao/vx/commit/04be2d976bb886fcbdac9fdf820a2a8d6a747384))
* resolve CI failures for yq, wix, xmake, vcpkg providers ([1be437f](https://github.com/loonghao/vx/commit/1be437fbe4197ea3e80f31eabfece3faaca74bd6))
* resolve compiler errors in test files ([5e97fd2](https://github.com/loonghao/vx/commit/5e97fd2b0e057d1b9bb2bfcd1b7f1919325ff9f3))
* resolve Linux CI failures for ffplay/ffprobe/gofmt/lld/xmake/yq ([3c8bf60](https://github.com/loonghao/vx/commit/3c8bf609c8785b02a8134b8e59cd2db943d6cbe6))
* resolve macOS CI failures for ffmpeg and imagemagick ([3e9bcc8](https://github.com/loonghao/vx/commit/3e9bcc824436df6d9404ecd7e9a11b8f006c2946))
* resolve merge conflict in release manifest ([53bb34d](https://github.com/loonghao/vx/commit/53bb34d100b4452117eec8d0fa580d1d832989ea))
* resolve Python PYTHONHOME mismatch ([#696](https://github.com/loonghao/vx/issues/696)), improve version pagination, unify skills ([e477508](https://github.com/loonghao/vx/commit/e47750841f90045027114e4eff3dc59f593c0f13))
* resolve sha2 LowerHex compile errors and upgrade GitHub Actions ([c0ffcc9](https://github.com/loonghao/vx/commit/c0ffcc954dc4c59ad40d42c32819d39fb766d42f))
* **resolver:** resolve bundled runtime fallback executable ([6b03b94](https://github.com/loonghao/vx/commit/6b03b948e4bda66e8c6889f2ff6086175d14951a))
* **resolver:** stop re-installing system-managed runtimes on every vx invocation ([4fdb1c4](https://github.com/loonghao/vx/commit/4fdb1c4b40d38aa4f4ed0c96ba9706ca459254fd))
* **runtime:** check vx store first in ManifestDrivenRuntime.is_installed() and installed_versions() ([1966339](https://github.com/loonghao/vx/commit/19663398517146e231900c9839502df7de63c245))
* **runtime:** preserve bundled command prefixes ([a6560b1](https://github.com/loonghao/vx/commit/a6560b1758425a64be5a0271e3c5c2c420989fec))
* **runtime:** satisfy clippy collapsible-if lint ([70b682a](https://github.com/loonghao/vx/commit/70b682a18979bd511a8c0f6d96aa7e8fadf3cdfa))
* Rust ecosystem passthrough for rustc versions in resolve_version ([6228793](https://github.com/loonghao/vx/commit/6228793b2f55587f4f0583a14855e2184510c40f))
* **rust:** stop re-installing rust on every vx cargo invocation ([26a1aa5](https://github.com/loonghao/vx/commit/26a1aa5b55a4139c44b20dc97e77dce6f868eb0e))
* serialize runtime installs ([9b69487](https://github.com/loonghao/vx/commit/9b6948725f222f14492db2fd6766c74c73c5741d))
* skip broken micromamba windows release ([77025a3](https://github.com/loonghao/vx/commit/77025a31d22ca5a4e0153553011a35a390ecf5a6))
* stabilize test suite and version constraint parsing ([066155f](https://github.com/loonghao/vx/commit/066155fb8bcd66e252f8f84c9be24f735f5bb6a5))
* **starlark:** lower provider loading log level from info to debug ([b8947a5](https://github.com/loonghao/vx/commit/b8947a5e18cbd0a0e954b2b02f36ed2e2adb60fc))
* **starlark:** register all 14 stdlib modules in loader ([20e0ed8](https://github.com/loonghao/vx/commit/20e0ed87dfbf77a2e147741d5d0f96fc49b0e511))
* switch macOS FFmpeg download source from evermeet.cx to osxexperts.net ([4bc40be](https://github.com/loonghao/vx/commit/4bc40be718c53ce41a9f20f4eb19e00ab4f808df))
* temp_dir unbound variable in install.sh and uv strip_prefix ([e24a3fa](https://github.com/loonghao/vx/commit/e24a3fa536ae0ff2e13f27398ee0dd7e4091b26a))
* **tests:** add missing OutputFormat argument to handle_list calls ([2c44db1](https://github.com/loonghao/vx/commit/2c44db1643f4cc157471f3a540dddc134c383e78))
* **tests:** fix 14 failing tests across multiple crates ([df20c05](https://github.com/loonghao/vx/commit/df20c05e7caba950bf29a2f55e637873d30c7e93))
* **tests:** fix cargo-deny Windows URL test and add missing provider tests ([28f8367](https://github.com/loonghao/vx/commit/28f83670e9898e4866d6896323b52c59525ce32f))
* **tests:** fix output_tests and info_tests failures in non-TTY CI ([414bb8a](https://github.com/loonghao/vx/commit/414bb8ac55f722b5b5d1f268995d2275c757663b))
* **tests:** relax assertion in test_vx_toml_python_setup_dry_run ([1cc2979](https://github.com/loonghao/vx/commit/1cc2979bb6122fefff4791f96f760f2691673882))
* **tests:** resolve latest unit test failures ([0458baf](https://github.com/loonghao/vx/commit/0458bafe1fcf931a0ea20864a2567066bce5c4d5))
* **tests:** rewrite all provider runtime_tests to use create_provider() API ([0ae8cba](https://github.com/loonghao/vx/commit/0ae8cbad0fd77f80d0cb1a01ec61038a8d2b95f8))
* **ui:** show Installing feedback during auto-install to avoid perceived hang ([560684e](https://github.com/loonghao/vx/commit/560684e6240f66ee627aef714cec738d56f6eb78))
* unblock remaining CI regressions ([b5498a1](https://github.com/loonghao/vx/commit/b5498a12ba943ad33ef45aff2fb842dd7f960191))
* use bin/bash.exe for git-bash instead of git-bash.exe --attach ([dbe9be4](https://github.com/loonghao/vx/commit/dbe9be4e396039653e4b26349d54fb6781cc360b))
* use child version for bundled proxy runtime installation ([e97e333](https://github.com/loonghao/vx/commit/e97e33341e903a2f7324cde0d0593059a1244fcc))
* use struct attribute access ctx.platform.os instead of dict access ctx[platform][os] in stdlib star files ([b8f4c93](https://github.com/loonghao/vx/commit/b8f4c931d210643effccf73a8d86dd4c58fb92f6))
* use system_install for ffmpeg Linux, fix witr install_layout ([bcf1e7d](https://github.com/loonghao/vx/commit/bcf1e7d0f63a771ac3257df8c68dea8a4086ce81))
* **uv:** route uvx through uv tool run ([f63dcb3](https://github.com/loonghao/vx/commit/f63dcb3ee0ff59837c3e6f8a741020281609f540))
* **versions:** case-insensitive Ecosystem deserialization for vx.lock compatibility ([7f6e2ed](https://github.com/loonghao/vx/commit/7f6e2eda83ed7cdf07c946ba1b8ba85ea7c8cf8d))
* **vx-config:** fix sha2 GenericArray LowerHex compile error ([7aa358e](https://github.com/loonghao/vx/commit/7aa358e95c05ed3393c9fe110e9e04d737d246cf))
* **vx-provider-jj:** strip v prefix from version tags to prevent double-v in download URL ([67486e1](https://github.com/loonghao/vx/commit/67486e193e9d52e07b2e9b64ae844885f741f553))
* **watchexec:** remove unused load imports to pass provider static lint ([f4b7330](https://github.com/loonghao/vx/commit/f4b7330556201d7e32890e8e7987b978195c9cd9))
* **watchexec:** use .zip on Windows, .tar.xz on Linux/macOS ([ef58118](https://github.com/loonghao/vx/commit/ef58118c784259317c40fc431a0678b3e1c6119d))
* **where:** use executable_name() instead of runtime name for exe lookup ([ee5bfb4](https://github.com/loonghao/vx/commit/ee5bfb4cdb2bf00a890ebcb715312b5c60473954))
* **windows:** resolve OS error 193 when executing bundled runtimes (npm/npx) ([9aacc52](https://github.com/loonghao/vx/commit/9aacc5290cab2f5d04ad62f3d146be3abc2d8dc6))
* wire Starlark post_extract hooks into ManifestDrivenRuntime and fix bundled runtime detection ([2802d5a](https://github.com/loonghao/vx/commit/2802d5ab91967ad8be9b2d4f21df5dd8c516c037))
* **witr:** correct __type__ key in install_layout (double underscores) ([2cb2a39](https://github.com/loonghao/vx/commit/2cb2a395b51f7e44fe6c7bf50b5877fdcdb1db3d))
* **witr:** correct version pattern and binary path in provider.star ([e0eb805](https://github.com/loonghao/vx/commit/e0eb80546584867108295fbc4a63d5908202efdc))
* **witr:** override install_layout with correct type ([79bc88c](https://github.com/loonghao/vx/commit/79bc88ca23c1b91a456cd0bdb575ff05cf3fb28e))
* **witr:** rewrite provider without template to handle direct binaries correctly ([a73cef0](https://github.com/loonghao/vx/commit/a73cef072f473e64462f8f58eced438b5a12dcd8))
* **witr:** use 'binary' type for direct binaries (Linux/macOS) ([813eef2](https://github.com/loonghao/vx/commit/813eef280f36b800c8a4f6ef12ae5795068de5d3))


### Performance Improvements

* add cargo build optimization agent rule ([c884d9d](https://github.com/loonghao/vx/commit/c884d9dab26eb9aaddb7e32f6e1953fac2b88b3b))
* implement cargo-hakari workspace-hack + runtime/config refactoring ([b3d6597](https://github.com/loonghao/vx/commit/b3d65973fd1f0c00de19903f8f5f89172d143d34))
* optimize test and build configuration ([06727c5](https://github.com/loonghao/vx/commit/06727c54e2730bc72ee599230fb4277eca64230a))
* optimize workspace compilation settings ([1f1f528](https://github.com/loonghao/vx/commit/1f1f528fbe581fd0c5d18d121e58cd6ced17388c))


### Code Refactoring

* **build:** remove legacy provider.toml support, simplify build.rs and registry.rs ([a0cb55b](https://github.com/loonghao/vx/commit/a0cb55bb4038333df777ee065fcb2e8b95c8890b))
* **cli:** update commands and test utilities for runtime refactoring ([8e80df3](https://github.com/loonghao/vx/commit/8e80df37b279f3ae184435ddf6cc50b697c0a455))
* **env,version-fetcher:** eliminate platform/version utils duplication ([d271ac8](https://github.com/loonghao/vx/commit/d271ac834308f4d7a9da7b97ecd1ccc8ba7b119a))
* extract vx-star-metadata crate and eliminate Box::leak usage ([160a618](https://github.com/loonghao/vx/commit/160a61892c3070d85c8f51b30bca83f949737271))
* improve code quality - replace unwrap() and eprintln! with proper error handling ([3ac83d1](https://github.com/loonghao/vx/commit/3ac83d1c5507c79c6fdacbbde6b8bd14ab9b9bee))
* improve code safety and remove dead code ([92a5af7](https://github.com/loonghao/vx/commit/92a5af7716a0f9cafe85642ed64312f6062995c7))
* improve code safety by eliminating unsafe unwrap calls ([bf838c3](https://github.com/loonghao/vx/commit/bf838c3f8547882052acbb523370cadf08730c97))
* merge vx-core into vx-runtime-core ([e0f3e26](https://github.com/loonghao/vx/commit/e0f3e26ab532792d39c76a247daeaff02d1ab411))
* optimize provider.star files using stdlib templates ([646093f](https://github.com/loonghao/vx/commit/646093f24f469e4d6394f21533d9146acb01cce2))
* **provider:** conda use provider.star only (remove Rust code) ([a56af66](https://github.com/loonghao/vx/commit/a56af667ae1459e3eba7718c7e1d585e77268d3a))
* **providers:** replace all hand-written permissions dicts with stdlib helpers ([d49da4d](https://github.com/loonghao/vx/commit/d49da4d0b79efb487adb678deecd7535faef09a7))
* **providers:** simplify providers to use standard templates ([2b4bdbb](https://github.com/loonghao/vx/commit/2b4bdbb60f2b3e4943a0048236effa2653618371))
* replace bare .unwrap() with descriptive .expect() in production code ([991b4e2](https://github.com/loonghao/vx/commit/991b4e21d7db7c6532967b101df83df4c525d814))
* **resolver:** integrate ResolutionCache into execution pipeline ([03d6864](https://github.com/loonghao/vx/commit/03d6864d60427ce71d59834a263978522d5f081f))
* **runtime-core:** remove dead Runtime trait and provider machinery ([f9659e1](https://github.com/loonghao/vx/commit/f9659e1ff192cb105f7737f0bdd1d865cd927a52))
* **runtime:** split runtime.rs into module and add ISP sub-traits ([93a1f6d](https://github.com/loonghao/vx/commit/93a1f6d399f3db27359753884e4656659798e877))
* simplify all providers to PROVIDER_STAR only, remove redundant create_provider and star_metadata ([adf3484](https://github.com/loonghao/vx/commit/adf34845d2ebbdc49331c4403fe58c31a9ab6e56))
* split tests to tests/ dir, extract bridge/builder modules, remove metadata indirection, fix clippy warnings ([064af9a](https://github.com/loonghao/vx/commit/064af9aa324a8b8661414c0650d32b4b369e1ccf))
* unify progress bars and restructure docs progressively ([#812](https://github.com/loonghao/vx/issues/812)) ([aa48d18](https://github.com/loonghao/vx/commit/aa48d18f71ae539f6e814d8170b174c3faf2991f))
* use LazyLock for regex compilation and improve error handling ([c0c9946](https://github.com/loonghao/vx/commit/c0c994669c6cafd1fc72e3fefade9ec56954dff5))
* **vx-starlark:** replace path-based cache with content-hash incremental analysis cache ([3acb930](https://github.com/loonghao/vx/commit/3acb9302421ec23e295a70b149d553fedad7f7ac))


### Documentation

* add age and sops to tools overview and CHANGELOG ([eacdda8](https://github.com/loonghao/vx/commit/eacdda8fd69417f0e187e383967ae8357332d9af))
* add CLAUDE.md, .cursor/rules/*.mdc, and improve AI agent documentation ([#747](https://github.com/loonghao/vx/issues/747)) ([2ffce28](https://github.com/loonghao/vx/commit/2ffce2828212198edf61680debcb01e3459f5119))
* add complete Supported Tools section to llms-full.txt ([071cf62](https://github.com/loonghao/vx/commit/071cf620988c03c5288aa950ef7ee74a69bd74ed))
* add critical rules section to AGENTS.md for AI agents ([#869](https://github.com/loonghao/vx/issues/869)) ([d0fed57](https://github.com/loonghao/vx/commit/d0fed57b5daef8d7939c94e2fae7435ee9cc9b21))
* add latest RFCs (0037, 0039, 0040) to llms-full.txt ([#873](https://github.com/loonghao/vx/issues/873)) ([b6e5385](https://github.com/loonghao/vx/commit/b6e53850f1a4fdb4e729ba0292ae86788df00c68))
* add llms.txt and llms-full.txt following llmstxt.org protocol ([4ab49e9](https://github.com/loonghao/vx/commit/4ab49e934214ff1167d0feb97b75269e5b2fc729))
* add missing tools (actrun, ctlptl, gws) to documentation ([412a66e](https://github.com/loonghao/vx/commit/412a66e15fba75825298ef9bd8c44019b2745471))
* add missing tools to documentation ([6b0a94f](https://github.com/loonghao/vx/commit/6b0a94f9fb58e40acc4b37d0c20d06b2d189241d))
* add more tool examples to AGENTS.md ([bfde740](https://github.com/loonghao/vx/commit/bfde740d87c21ef92d086ffb08c0fe4b667ca440))
* add pre-commit hooks documentation (EN/ZH) and update contributing guides ([274aec5](https://github.com/loonghao/vx/commit/274aec5a0e85be9dc6f266437ee0077b31bf845d))
* add self-update command documentation with channel support ([f2a3e80](https://github.com/loonghao/vx/commit/f2a3e80c971a00e26fdbd311e14dafd41ba105fb))
* add Starlark Providers advanced guide (bilingual) ([38739c0](https://github.com/loonghao/vx/commit/38739c09be752fdf2cc460f3e2cd8938fc29651a))
* add worktrunk (wt) tool documentation ([bfa403f](https://github.com/loonghao/vx/commit/bfa403f20e434ea29214c97fc511aadce1def10c))
* **agent:** add cross-language global install contract and fix RFC links ([bc47efb](https://github.com/loonghao/vx/commit/bc47efb89a593de35ffd0b4959d89279e6258565))
* **cargo:** add fast build optimizations inspired by Bevy ([31a0588](https://github.com/loonghao/vx/commit/31a0588021b353f3612a70d84f0923b2e4e09437))
* **cleanup:** sync provider count from 105 to 111 across all docs ([3784b5d](https://github.com/loonghao/vx/commit/3784b5dcd2587c4dac3ac6c0ceffb7ce415e2fb9))
* enhance AI agent documentation and sync skills ([#736](https://github.com/loonghao/vx/issues/736)) ([043b7aa](https://github.com/loonghao/vx/commit/043b7aaca2bffb3a9069a1cd76d295df7976fc8c))
* enhance AI agent documentation with decision framework, MCP guide, and version fixes ([#732](https://github.com/loonghao/vx/issues/732)) ([e806f1f](https://github.com/loonghao/vx/commit/e806f1f57fc76af44be705b78da1028b76903e72))
* enhance AI agent ecosystem with 15+ agent support ([#749](https://github.com/loonghao/vx/issues/749)) ([fff3c42](https://github.com/loonghao/vx/commit/fff3c421e9925ad0ea3762655e12f5b5865c8a5f))
* fix dead links in docs build ([3cc85ca](https://github.com/loonghao/vx/commit/3cc85cad2cebaece71ce559283331c8e31fdf53d))
* fix duplicate Critical Rules sections in AGENTS.md ([#870](https://github.com/loonghao/vx/issues/870)) ([444cae2](https://github.com/loonghao/vx/commit/444cae296cb0530415a57db05f65a4454916130f))
* fix syntax errors in other.md and quality.md ([#866](https://github.com/loonghao/vx/issues/866)) ([5292516](https://github.com/loonghao/vx/commit/52925165b6022618610952454e7b32fe067a53dc))
* improve agent documentation for better AI discoverability ([#701](https://github.com/loonghao/vx/issues/701)) ([ca2bfe7](https://github.com/loonghao/vx/commit/ca2bfe7fd4df5ce59dae07b685512fdaa41cb669))
* improve agent knowledge - update provider count to 78, enhance AGENTS.md, sync skills ([#687](https://github.com/loonghao/vx/issues/687)) ([2686f86](https://github.com/loonghao/vx/commit/2686f86e3b610181f9e287b8af90274299190a5f))
* improve agent knowledge - update provider.star docs, fix tool counts, add creating-provider guide ([cdfcf32](https://github.com/loonghao/vx/commit/cdfcf325add5ba559ce505b882e99ef576e9e2ed))
* improve AI agent documentation and fix version inconsistencies ([#710](https://github.com/loonghao/vx/issues/710)) ([72389b7](https://github.com/loonghao/vx/commit/72389b7f8efa5c891c4b3b22ad9220ee11017db1))
* improve AI agent documentation ecosystem ([#741](https://github.com/loonghao/vx/issues/741)) ([59c4f12](https://github.com/loonghao/vx/commit/59c4f127f2b81b86bc4d875c91a42bf22814270c))
* optimize agent docs for v0.8.20 — add Copilot instructions, expand AI agent support to 17+ ([#762](https://github.com/loonghao/vx/issues/762)) ([0430e70](https://github.com/loonghao/vx/commit/0430e70daf43338c5de0bb07c7fc60bb5ea5e01f))
* optimize AGENTS.md as progressive disclosure map ([#868](https://github.com/loonghao/vx/issues/868)) ([5c22bd5](https://github.com/loonghao/vx/commit/5c22bd505fb47fa361a606072f4f9ec09b3ec3bb))
* remove emojis from README and update descriptions ([1ebb626](https://github.com/loonghao/vx/commit/1ebb626faaa506319af7c975f273d281e43cb2a3))
* **rfc-0032:** update Plan D (hakari implemented), Plan E/F tracking status ([22794da](https://github.com/loonghao/vx/commit/22794dae2c3a2543470727c68f746eafe7633f2f))
* **rfc:** add RFC 0036 - Starlark Provider Support ([8501be4](https://github.com/loonghao/vx/commit/8501be436ba06d175ed82aa4b0ea14b28acaf008))
* **rfc:** update Phase 2 progress in RFC 0032 ([b10be5a](https://github.com/loonghao/vx/commit/b10be5ae44beb5dc447ab52d332e7e9dc73057cf))
* **rfc:** update Phase 2 status in RFC 0032 ([b7f69d8](https://github.com/loonghao/vx/commit/b7f69d840b224c879dda900ac3d9d7da57b19089))
* **rfc:** update RFC 0036 v0.3 - add Buck2 typed provider_field, load() module system, incremental analysis cache, declarative actions ([46a939c](https://github.com/loonghao/vx/commit/46a939ca40104e839955176d470c27454dd64f83))
* simplify AI agent configs, add vx wt/witr examples ([729f224](https://github.com/loonghao/vx/commit/729f2247eb9a522127c9098730c00adc1e1fc374))
* sync zh contributing.md and add zh fixes docs ([e2dbe08](https://github.com/loonghao/vx/commit/e2dbe08d136d45f82f8517db43dd1ab7185e3a0d))
* teach token-efficient vx agent workflows ([bfa669a](https://github.com/loonghao/vx/commit/bfa669a2f04d899df1301dcf290d90018b4f617c))
* update AGENTS.md, GEMINI.md, CLAUDE.md, add SECURITY.md and CODE_OF_CONDUCT.md ([cb395ce](https://github.com/loonghao/vx/commit/cb395cef13483506a45fc39c666722fd396d3756))
* update AGENTS.md, GEMINI.md, CLAUDE.md, add SECURITY.md and CODE_OF_CONDUCT.md ([#915](https://github.com/loonghao/vx/issues/915)) ([cb395ce](https://github.com/loonghao/vx/commit/cb395cef13483506a45fc39c666722fd396d3756))
* update cargo-build-optimization agent rule with implemented optimizations ([1fb7132](https://github.com/loonghao/vx/commit/1fb7132dda5b4e52c1eb6c5837c13551e3e5dd7a))
* update documentation for self-update and worktrunk ([53d4ddf](https://github.com/loonghao/vx/commit/53d4ddf901f41cdf24f7433eee844eb823753ddc))
* update media.md and witr.md with vx-org/mirrors download source ([#864](https://github.com/loonghao/vx/issues/864)) ([3fa5f2a](https://github.com/loonghao/vx/commit/3fa5f2acc63ba54b181ff314300cc95e4579c694))
* update outdated version numbers in README.md ([#874](https://github.com/loonghao/vx/issues/874)) ([8a8d218](https://github.com/loonghao/vx/commit/8a8d218a05912cfd107ab2bf70673ecb091aa481))
* update provider count 136 -&gt; 137 (add witr) ([#859](https://github.com/loonghao/vx/issues/859)) ([58ae269](https://github.com/loonghao/vx/commit/58ae26943d66ed0cd7cb8555a1b3a497f409037e))
* update provider count from 129/131 to 132 ([b25138c](https://github.com/loonghao/vx/commit/b25138cc618dd0fdaded91fcebe0fa016537881b))
* update provider count from 129/131 to 132 across all documentation ([d7a7b67](https://github.com/loonghao/vx/commit/d7a7b674ff7ac4b52685a502de3c2db097225ee1))
* update provider count from 132/135 to 136 across all docs ([382260d](https://github.com/loonghao/vx/commit/382260d99a46da9c9075149fe233e9c8ed5e5925))
* update provider count from 132/135 to 136 across all docs ([1cfffe2](https://github.com/loonghao/vx/commit/1cfffe26239d07a2a4137cc4960ec8f556e77afe))
* update provider count from 136 to 137 across documentation ([#865](https://github.com/loonghao/vx/issues/865)) ([1e732a9](https://github.com/loonghao/vx/commit/1e732a90af70357b1688516e4a6be0d42d766ea0))
* update provider count to 129 in AGENTS.md ([a231b85](https://github.com/loonghao/vx/commit/a231b85d11ccef5c28cefee9444eb23c786a13cf))
* update provider count to 131 in AGENTS.md ([ca6b517](https://github.com/loonghao/vx/commit/ca6b517f1806d9ac4f75df322a05c171ccbf5433))
* update provider count to 132 and add conda/micromamba/mamba ([70b4244](https://github.com/loonghao/vx/commit/70b42445e55a4479c17f1d51fc3998910c3f2c85))
* update provider count to 132 and add conda/micromamba/mamba to overview ([420a01d](https://github.com/loonghao/vx/commit/420a01d00e6fc46116f6bd7000fb141a0568d03c))
* update provider count to 137 (add witr) ([672ff0b](https://github.com/loonghao/vx/commit/672ff0b4ef35ac0a6027a7ec26edf663229e9b75))
* update rules to reflect provider.star migration ([ef14534](https://github.com/loonghao/vx/commit/ef14534f4b9a69b6521944f5a5b924b075c42454))
* update Rust version requirement in contributing.md ([ba93d32](https://github.com/loonghao/vx/commit/ba93d32659782d845986b0a4f284e7ef968b6729))
* update Rust version requirement to 1.93+ (Edition 2024) in contributing.md ([63ecae5](https://github.com/loonghao/vx/commit/63ecae56fb68330e09149dd2ae82e6ba87ad2d95))
* update Rust version to 1.95.0+ and fix RFC count ([#875](https://github.com/loonghao/vx/issues/875)) ([0e1228b](https://github.com/loonghao/vx/commit/0e1228b0704b9fac7a1972429990a650aa4228c0))
* update tools overview and quality documentation ([134cfac](https://github.com/loonghao/vx/commit/134cfacfacca95069895cfd53866518ba08d77e0))
* update version to 0.8.32 and provider count to 129 ([873d19e](https://github.com/loonghao/vx/commit/873d19eaadc8afe5714db2d126712f2fd39d5121))
* update version to v0.8.35 and provider count to 136 ([7c7e407](https://github.com/loonghao/vx/commit/7c7e407f144066add20617459129b163f099e2ef))
* update version to v0.8.35 and provider count to 136 ([a1c1c15](https://github.com/loonghao/vx/commit/a1c1c155ff0a9cdf991a46f3edcadd4cde082c64))
* update version to v0.8.36 and provider count to 136 ([b4e1261](https://github.com/loonghao/vx/commit/b4e1261b90e9062124c3c7dd1bcb155c9642ae11))
* update version to v0.8.36 and provider count to 136 ([6a9865a](https://github.com/loonghao/vx/commit/6a9865acaca79282ba98e067671c126296492b08))
* update version to v0.8.37 ([0bc8d0a](https://github.com/loonghao/vx/commit/0bc8d0a961b6b90d25e1e59ac76741b4f8c78ebf))
* update version to v0.8.37 in AGENTS.md and README.md ([3587632](https://github.com/loonghao/vx/commit/3587632df498435f45c16cc3a7e961a991cf5669))
* update version to v0.8.39 ([a03513c](https://github.com/loonghao/vx/commit/a03513c473f9826d1657fe22515c7c6a11cd5ca8))
* update version to v0.8.39 in AGENTS.md and README.md ([3bf41a6](https://github.com/loonghao/vx/commit/3bf41a6291979300bfb4492fb05f8764cdff9e6a))

## [0.9.7](https://github.com/loonghao/vx/compare/v0.9.6...v0.9.7) (2026-05-26)


### Bug Fixes

* **ci:** replace dorny/paths-filter with native git diff ([aab8f2b](https://github.com/loonghao/vx/commit/aab8f2b0f009ab466aa21175d94af65afb4acc93))


### Documentation

* remove emojis from README and update descriptions ([8499fb0](https://github.com/loonghao/vx/commit/8499fb059bacafb5807e34e8e02cdbf090198e7e))

## [0.9.6](https://github.com/loonghao/vx/compare/v0.9.5...v0.9.6) (2026-05-26)


### Documentation

* update AGENTS.md, GEMINI.md, CLAUDE.md, add SECURITY.md and CODE_OF_CONDUCT.md ([7547b20](https://github.com/loonghao/vx/commit/7547b200e1afd18ed225d8ef3f410d4e0b4e903e))
* update AGENTS.md, GEMINI.md, CLAUDE.md, add SECURITY.md and CODE_OF_CONDUCT.md ([#915](https://github.com/loonghao/vx/issues/915)) ([7547b20](https://github.com/loonghao/vx/commit/7547b200e1afd18ed225d8ef3f410d4e0b4e903e))

## [0.9.5](https://github.com/loonghao/vx/compare/v0.9.4...v0.9.5) (2026-05-26)


### Bug Fixes

* harden release workflow reruns ([631ad08](https://github.com/loonghao/vx/commit/631ad08419077d68573431151656e31aaf03054c))


### Documentation

* teach token-efficient vx agent workflows ([06c3558](https://github.com/loonghao/vx/commit/06c35585517f1de8709393c3361c8819de241cd9))

## [0.9.4](https://github.com/loonghao/vx/compare/v0.9.3...v0.9.4) (2026-05-26)


### Features

* add 11 new providers (mise, gitleaks, biome, lazydocker, k9s, gping, watchexec, duf, trippy, sd, actionlint) ([a6abcab](https://github.com/loonghao/vx/commit/a6abcab83d78bc2d04d9d8dc6df0437ad0aef8ba))
* add 7 high-priority developer tool providers (lazygit, delta, hyperfine, zoxide, atuin, chezmoi, eza) ([3ec51ef](https://github.com/loonghao/vx/commit/3ec51efb29084be123c450f46b34b90e51ab2be4))
* add 7 new providers (tealdeer, dust, xh, bottom, trivy, zellij, dive) ([5bc63ab](https://github.com/loonghao/vx/commit/5bc63ab4b608bee61bfb9140cb0aa63c13ffe162))
* add age and sops providers, update project analyzer frameworks ([00eadf2](https://github.com/loonghao/vx/commit/00eadf2ef31aaca29f33d0aec3eea75007355ec3))
* add build cache providers (sccache, ccache, buildcache) ([c518f90](https://github.com/loonghao/vx/commit/c518f90862058a8b43cf79e98e380b15b8d6c7e7))
* add dynamic package_prefixes from provider.star metadata ([6d8a197](https://github.com/loonghao/vx/commit/6d8a1973ea2aac611cffd7de547b9d71acc74530))
* add FilterLevel enum (Light/Normal/Aggressive) for compact output ([#804](https://github.com/loonghao/vx/issues/804)) ([9582561](https://github.com/loonghao/vx/commit/958256105950bea61b87ebf45ec3550a97da3314))
* add helix and yazi providers ([b62985b](https://github.com/loonghao/vx/commit/b62985b6d19d11c6c971d64fea8f8f2e10fcb8c2))
* add llvm/conan/xmake/wix providers and enhance msvc/msbuild for C++/C# build automation ([adb9720](https://github.com/loonghao/vx/commit/adb97206b2dee7eb4b8281ebae5022d9e6f1c582))
* add new oneshot runner ecosystems and update CLI help/docs ([fcccbdd](https://github.com/loonghao/vx/commit/fcccbddd6faf7a72cab081b2e3dae18f418df74a))
* add Nx and Turborepo cache providers, fix CI sccache issue ([30954da](https://github.com/loonghao/vx/commit/30954da54563a41f7aa21824de36485784a632d3))
* add python 3.7 and wasm runtimes ([117c999](https://github.com/loonghao/vx/commit/117c9990da01f4919451c96dc43700109a92bf01))
* add rust wasm providers ([4206a5e](https://github.com/loonghao/vx/commit/4206a5e2839d8936e0fd17c1e9a26825cd5457bc))
* add self-update check with non-blocking notification ([9d50d99](https://github.com/loonghao/vx/commit/9d50d99c526fa6dad08e46148f27cb19784b7278))
* add starlark provider support with bash, curl, meson, openssl, pre-commit, release-please, rez, systemctl, xcodebuild providers ([cd50d2f](https://github.com/loonghao/vx/commit/cd50d2fb07ea554e759f831fa3d6bfd491fa1e8a))
* add vcpkg provider for C++ dependency management ([dad8705](https://github.com/loonghao/vx/commit/dad8705ee05d85bc9d64d44d561f9fd929c42250))
* add vx skill for ClawHub publication ([#638](https://github.com/loonghao/vx/issues/638)) ([42e2dc7](https://github.com/loonghao/vx/commit/42e2dc7d44cbab95121e2435f83524e7ae68105f))
* add vx-output-filter crate for compact subprocess output ([#802](https://github.com/loonghao/vx/issues/802)) ([7cc7d7b](https://github.com/loonghao/vx/commit/7cc7d7b4c2dbe5589e5ff3fc87b77c882ca5f15b))
* add well-known Python version fallback for python-build-standalone ([3d81387](https://github.com/loonghao/vx/commit/3d81387085e8c5abbd1d5094b52407efc2dccb6c))
* add witr provider (137 providers total) ([76fffd7](https://github.com/loonghao/vx/commit/76fffd7397832f7dd7284401eaf58f24b107ce9f))
* add worktrunk provider (132 tools total) ([#839](https://github.com/loonghao/vx/issues/839)) ([730e935](https://github.com/loonghao/vx/commit/730e935d0e2b24c846e630d79451ccb98a1a32c7))
* **ai:** implement RFC 0035 AI integration optimization ([595f77f](https://github.com/loonghao/vx/commit/595f77f059e4587ceb550b310454840f10417711))
* **auto-improve:** squash merge auto-improve branch ([4ca49e4](https://github.com/loonghao/vx/commit/4ca49e4bfe85b93973fc3e2b0bb6bcabe4052a0b))
* bridge global install commands to vx package shim workflow ([60465b1](https://github.com/loonghao/vx/commit/60465b1070ee09665d7b361e7dec3703d8185ec8))
* **build:** add rust-lld linker for faster builds (RFC 0032 Phase 1) ([7187439](https://github.com/loonghao/vx/commit/7187439a784eb18bebf2b64d67a28303879cc7e1))
* **build:** add vx-runtime-core and vx-runtime-archive to workspace dependencies ([555d405](https://github.com/loonghao/vx/commit/555d4054108e58428c581f58bc3d2c69c4083a58))
* **build:** create vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([dc0e646](https://github.com/loonghao/vx/commit/dc0e646421e449d9c6f964a2374fa3b1c95f9797))
* **build:** integrate vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([7077ba2](https://github.com/loonghao/vx/commit/7077ba273e8ec65d21cad44e9d1582546fff26d3))
* change non-TTY default output from JSON to Toon, disable CDN by default ([d5cc4bd](https://github.com/loonghao/vx/commit/d5cc4bdfd30d58d8c04c0f371b17e659d555817b))
* **cli:** add update channel support (stable, beta, dev) ([7f98dcc](https://github.com/loonghao/vx/commit/7f98dcc59075c866bc2fcc77b23fab474312fcc2))
* **cli:** Agent DX improvements for AI agents ([1e5ac9d](https://github.com/loonghao/vx/commit/1e5ac9db00d3501b8f8ba124800ef99751679869))
* **cli:** bridge global install commands to vx package shims ([b53bf95](https://github.com/loonghao/vx/commit/b53bf952d16369b5f6fd3ada6dccc31fe2f460bd))
* **cli:** enable direct global command shims in vx bin dir ([5bb93c9](https://github.com/loonghao/vx/commit/5bb93c96a06aca8f6786f31b715bbf555439ede1))
* **cli:** overhaul vx add/remove with format-preserving edits ([8e2fab5](https://github.com/loonghao/vx/commit/8e2fab57073ac246028ac8aecfa1b7a3de28cb49))
* **ecosystem_aliases:** route ecosystem:package to dedicated provider binary ([5f78ea9](https://github.com/loonghao/vx/commit/5f78ea9a6b7b5f7a4bffc053ce19b86eafe8c072))
* **hooks:** upgrade cargo-hakari pre-commit hook to auto-fix mode ([3f30652](https://github.com/loonghao/vx/commit/3f3065274840def26a2c796ed3460728f141a694))
* land provider and CLI improvements ([2c8e2fd](https://github.com/loonghao/vx/commit/2c8e2fdc74f72ee10d17bad9cf2b88f769a99115))
* **list:** sort tool list alphabetically (a-z) ([9f18585](https://github.com/loonghao/vx/commit/9f18585ec4c38d25ed6a05c5dc4a624a89543c24))
* propagate explicit version from bundled runtime to parent dependency ([#766](https://github.com/loonghao/vx/issues/766)) ([223b131](https://github.com/loonghao/vx/commit/223b131f9a5de6fec92ab50936eb05fb765c2347))
* **providers:** add cargo-audit provider ([138735e](https://github.com/loonghao/vx/commit/138735ea6068b587278b373dd22d3c28f5d9da13))
* **providers:** add cargo-nextest and cargo-deny providers ([19ca7c1](https://github.com/loonghao/vx/commit/19ca7c1cb4acff0571cfd459032c1b5fdeeb1a87))
* **providers:** add conda provider with micromamba, conda and mamba ([6c6aa46](https://github.com/loonghao/vx/commit/6c6aa4636a6a9877d657e40da63abc3e24c78f7e))
* **providers:** add conda provider with micromamba, conda and mamba ([868a4fa](https://github.com/loonghao/vx/commit/868a4fa310db45c7aeb2b6e4cf6935ce78a45252)), closes [#389](https://github.com/loonghao/vx/issues/389)
* **providers:** add grpcurl provider + update provider count to 114 ([c015b88](https://github.com/loonghao/vx/commit/c015b880b01f9ac1ea18013f7dbb954ee9c8a622))
* **providers:** add kind and k3d providers ([9ff4388](https://github.com/loonghao/vx/commit/9ff4388f43873c564cc7283434bd8514573a1051))
* **providers:** add tokei provider + triage stale issues ([f130fe3](https://github.com/loonghao/vx/commit/f130fe3b00c87dedf57390299a82ee2d280eb5e5))
* **resolver:** implement version priority with vx.lock support ([9fe8714](https://github.com/loonghao/vx/commit/9fe8714c1663fbedb9818381e85d89e0939b2e9a))
* **rfc-0037:** implement ProviderHandle unified facade for CLI commands ([cb2b65a](https://github.com/loonghao/vx/commit/cb2b65af1bca2b1a5d2b92ec21016bafb079dae4))
* **rfc-0040:** implement version_info() for toolchain version indirection ([126fbb1](https://github.com/loonghao/vx/commit/126fbb1d22b1c3b184a92dc511a8c3bc77e31ba1))
* **routing:** prefer dedicated provider over cargo install for ecosystem:package ([ef21222](https://github.com/loonghao/vx/commit/ef2122227c9cbeecdacdd1aa67cdc50a011fdbc1))
* **starlark:** add github.star stdlib + jj provider.star migration ([1d8802f](https://github.com/loonghao/vx/commit/1d8802f0553bb04b1c365b4a92a8877683fa8b1c))
* **starlark:** complete provider.star migration and fix stdlib ctx access ([965e8bf](https://github.com/loonghao/vx/commit/965e8bfebf13b88ca9c69d083e4f4cbad49c7ded))
* **tests:** add comprehensive Python provider e2e tests ([6c96bc5](https://github.com/loonghao/vx/commit/6c96bc5290dacaa0ac48a1f777b7961f79839e7f))
* track output token savings ([ea8099d](https://github.com/loonghao/vx/commit/ea8099dd02f14d4767cb8771a8c7abece7b9e38e))
* use RuntimeContext for install_options instead of env vars (Phase 1) ([2eece16](https://github.com/loonghao/vx/commit/2eece163adf7310bb759e265968d329690a66d7a))
* **vx-starlark:** implement Phase 2 Starlark execution engine ([6cd99f8](https://github.com/loonghao/vx/commit/6cd99f83bd0f1353dc7befd3b4c93fe265ea82ec))
* **vx-starlark:** Phase 2 - integrate starlark-rust execution engine ([fae7bc7](https://github.com/loonghao/vx/commit/fae7bc77c87cf298891cff18825910fbcda47053))
* wire provider dynamic deps and fix install routing ([ec4cccf](https://github.com/loonghao/vx/commit/ec4cccf8aba470771c2e6572d1a96f720d233712))


### Bug Fixes

* **7zip:** fix executable name and system_paths to point to binary file ([4f55b26](https://github.com/loonghao/vx/commit/4f55b267b903abeb608f71eb2f6b743de569a34f))
* Add 'if: !startsWith(github.event.head_commit.message, chore: release)' guards to skip these workflows when the push is a release commit. ([7874981](https://github.com/loonghao/vx/commit/787498183f4bb5062c5b54edb7f6d77efbb7521a))
* add a ound flag so END block only prints when the match block did not. Also add head -1 safety to ensure only one line is captured. ([d756244](https://github.com/loonghao/vx/commit/d75624428f8e9c2ef562d14d86c7c3549c683fe1))
* add is_version_installable and prepare_execution for bundled runtimes ([c2b5704](https://github.com/loonghao/vx/commit/c2b570409e6274a3e4a9c6d10de8de1aa17f5c9c))
* add recursive search for bundled executables and remove wrong fallback ([90eb628](https://github.com/loonghao/vx/commit/90eb6284b817409b20fcd438db6c6efa3b52899e))
* add workspace-hack deps and remove invalid CI cache parameter ([60cf870](https://github.com/loonghao/vx/commit/60cf870de5ebcce245d05ac02163db0d2192aa48))
* address provider CI regressions ([035af34](https://github.com/loonghao/vx/commit/035af34603085b5b285df12719f9fa5b7f2653f6))
* **ai:** fix skills format and install all 5 skills on setup ([ec1af7c](https://github.com/loonghao/vx/commit/ec1af7c2f92fea97b430a7e251391fdb6743fc68))
* align starlark mock signatures with stdlib and fix provider tests ([88c16b2](https://github.com/loonghao/vx/commit/88c16b22618d8f4a6cfb1516cebe7b26ebbf8ceb))
* **all:** comprehensive CI fixes and Rust 1.95.0 upgrade ([#843](https://github.com/loonghao/vx/issues/843)) ([19b1dff](https://github.com/loonghao/vx/commit/19b1dff9765782853cc179a3e9def1dab0844006))
* auto-disable Spectre mitigation in MSBuild bridge when libs are missing ([81cd697](https://github.com/loonghao/vx/commit/81cd697c2da732ea5537bb23032345a7ca79e596))
* auto-fetch versions when version_date cache miss in download_url ([d7223c2](https://github.com/loonghao/vx/commit/d7223c218c296141ded07998f6b4c3116ee7d15b))
* avoid msvc repair for unrelated commands ([8e31791](https://github.com/loonghao/vx/commit/8e3179120713983cc4f2227b80099b058770c8ae))
* **cache:** skip NeedsInstall results in resolution cache; extend TTL to 24h ([7620f77](https://github.com/loonghao/vx/commit/7620f779a662b6d472ab2cebb2dd68dc836a6bba))
* **cargo-audit:** remove unused rust_triple import (lint) ([2f4c5f4](https://github.com/loonghao/vx/commit/2f4c5f4125da3da118a4b51a7015948d04a0e000))
* change internal rustup/toolchain debug logs to trace level ([3744ee5](https://github.com/loonghao/vx/commit/3744ee5facb0e1f6d6b6a0052a7fe065f67a4860))
* **ci:** add sccache setup to all CI workflows ([d9ecba7](https://github.com/loonghao/vx/commit/d9ecba7fd31eac7f764ac0a9f1cfffcc3d0a3cac))
* **ci:** add sccache setup to benchmark workflow ([d67e676](https://github.com/loonghao/vx/commit/d67e67633eb1bb815148473f7953db045aaf6423))
* **ci:** ensure Release workflow triggers even when release is created from update-pr job ([843025d](https://github.com/loonghao/vx/commit/843025d732398042ef9c982d79e5469fd1c36898))
* **ci:** exclude vx-msbuild-bridge from cargo-dist & improve skills sync ([fb4503d](https://github.com/loonghao/vx/commit/fb4503dd789b1f95168556c29119609d537f3e6f))
* **ci:** fix discovery parser and CI skip list for Linux/macOS failures ([f2e7632](https://github.com/loonghao/vx/commit/f2e763210948e524dfbce89015a0c78aa6f72ea4))
* **ci:** handle skipped/cancelled jobs in CI Success gate ([2c69ec0](https://github.com/loonghao/vx/commit/2c69ec0c48482d2ac5abb3bc99c38df011030109))
* **ci:** improve sccache path handling on Windows ([b960cb0](https://github.com/loonghao/vx/commit/b960cb063dbcd2f7362a6389f94fbe39460e45f2))
* **ci:** include Cargo.lock in workspace-hack commit step ([62ebb4a](https://github.com/loonghao/vx/commit/62ebb4abcf0c7c3b1b7bc1c909a2555d75c532f1))
* **ci:** increase Windows timeout to prevent CI failures ([217e2e4](https://github.com/loonghao/vx/commit/217e2e43fac7a3764d977b460f65af783478b11e))
* **ci:** install sccache in quick-test job ([32819ad](https://github.com/loonghao/vx/commit/32819addbd1ed977921df4207cc1a4e73395a258))
* **ci:** prevent duplicate release-please PRs on release merge ([7874981](https://github.com/loonghao/vx/commit/787498183f4bb5062c5b54edb7f6d77efbb7521a)), closes [#713](https://github.com/loonghao/vx/issues/713)
* **ci:** remove lld linker on macOS due to compatibility issues ([17bf47f](https://github.com/loonghao/vx/commit/17bf47f40901335697ec39330d332d6577194bb7))
* **ci:** remove max-versions-to-keep from winget-releaser ([87ed136](https://github.com/loonghao/vx/commit/87ed136d8092abbb6cfd650557ba01c9274bf8d6))
* **ci:** replace curl POST with clawhub CLI in sync-skills workflow ([2d09162](https://github.com/loonghao/vx/commit/2d0916271bd605bd7ef4856051dce100e42e854a))
* **ci:** replace remaining vx run cargo scripts with direct vx cargo calls ([c3185a2](https://github.com/loonghao/vx/commit/c3185a231a5d4670caba1b494c11be437cca5998))
* **ci:** resolve required check name conflict blocking PR merges ([2c9def5](https://github.com/loonghao/vx/commit/2c9def5fa941d0b31644377f097798d5b115a551))
* **ci:** sanitize provider cache keys ([355f143](https://github.com/loonghao/vx/commit/355f14334b6bdd459dfd7dfa3332dbcda4145e50))
* **ci:** skip wix and xmake in CI tests ([90a6bcd](https://github.com/loonghao/vx/commit/90a6bcd20867c2ae1a37371a479d3e3a3eef2dfc))
* **ci:** split release-please into two jobs to fix tag creation ([3eebf8b](https://github.com/loonghao/vx/commit/3eebf8b16f35dc40ab897efd95fbcdd3b0d675bd))
* **ci:** switch apt mirror to azure.archive.ubuntu.com for cross builds ([7da92d9](https://github.com/loonghao/vx/commit/7da92d9f50ed9088967c0afd2aa94038eec6dd67))
* **ci:** use system cargo directly instead of vx cargo ([1f5917a](https://github.com/loonghao/vx/commit/1f5917ae75cbbb0bfcefc6f6ee5b184571fc9ef7))
* **ci:** use vx cargo prefix in justfile recipes for CI compatibility ([6a5e272](https://github.com/loonghao/vx/commit/6a5e27225009680694120cc50a7bc1317411d320))
* clean extraction markers before re-installing missing MSVC components ([5d436c7](https://github.com/loonghao/vx/commit/5d436c788abd064516e13954c7fef4acf7a7da78))
* **cleanup:** fix compile errors from ecosystem_aliases feature ([a1704f0](https://github.com/loonghao/vx/commit/a1704f07506484f323a6f13e804247f96c8cb406))
* **cli:** add --toon shortcut flag for TOON output format ([b2c4f9b](https://github.com/loonghao/vx/commit/b2c4f9b87a02575e40a8f5d34ecd196aed43b855))
* **cli:** fix Clippy warnings and test compilation errors ([91d5691](https://github.com/loonghao/vx/commit/91d5691819ab19afd533fc3b5289e9e36a170858))
* **cli:** fix formatting issues (run cargo fmt) ([4c478c0](https://github.com/loonghao/vx/commit/4c478c08d4f3ae19999ebf4ea72f6af76bec2b8b))
* **cli:** fix vx check system_fallback and vx lock for installed tools ([3d6087a](https://github.com/loonghao/vx/commit/3d6087a513ff89c8fcb65dd0f1082db773bc34ca))
* clippy useless_vec warnings in tests ([9ff0999](https://github.com/loonghao/vx/commit/9ff099999fc2e679919863f141ae2a78a8b2ed58))
* **cli:** remove debug print from lib.rs ([5296982](https://github.com/loonghao/vx/commit/52969829718f067e6b5229284054f5a34f814f9f))
* **cli:** send update notifications to stderr ([ec12a42](https://github.com/loonghao/vx/commit/ec12a42646e2edb9813ce5fb78d64b383f267c5b))
* collapse nested if statements to satisfy clippy ([80b1f25](https://github.com/loonghao/vx/commit/80b1f25222e83677ff79b7043ed1ed8719b5ee8a))
* **console:** use eprintln for progress output to avoid stdout contamination ([9a56a9d](https://github.com/loonghao/vx/commit/9a56a9dfd459c2602025a026e380744d25b88fb3))
* correct cmake macOS download URL and jq environment key ([3d654c1](https://github.com/loonghao/vx/commit/3d654c127b6abc0c992ef2d1a395260b0e050a06))
* correct download URLs for grpcurl, k3d, kind, and duckdb providers ([4f95e21](https://github.com/loonghao/vx/commit/4f95e21f8e8660c5757fddf05b191f447fd17e87))
* correct metrics calculations ([005ef5a](https://github.com/loonghao/vx/commit/005ef5abe5ebe3dd4088a9829152e32ee4574be2))
* correct RFC count (40+ -&gt; 50+) and update Rust version badge (1.93+ -&gt; 1.95.0+) ([#879](https://github.com/loonghao/vx/issues/879)) ([619b7f1](https://github.com/loonghao/vx/commit/619b7f18cd657e35d0ad421828527cdb63da230e))
* **deps:** update rust crate anstream to v1 ([7634188](https://github.com/loonghao/vx/commit/76341885c47e9f0e5b5aac40c2996c2154416814))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 ([2cbf0ae](https://github.com/loonghao/vx/commit/2cbf0aef1610a9aaebeacac8d7ee3c4185052229))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 - abandoned ([35a81af](https://github.com/loonghao/vx/commit/35a81afbec661c5623940cc4ff5dbd5a2919f380))
* **deps:** update rust crate starlark_derive to 0.14 ([883e1b5](https://github.com/loonghao/vx/commit/883e1b52512d911531de8b5fda08b9c1d5d02ec6))
* **deps:** update rust crate toon-format to 0.5 ([aede396](https://github.com/loonghao/vx/commit/aede39631502ed68116ce62deb4ade0059177fd0))
* **dist:** exclude vx-star-metadata from cargo-dist release artifacts ([b9cc621](https://github.com/loonghao/vx/commit/b9cc621500475f4a433ffa8c0c1a02f9e38bf091))
* **docker:** switch apt mirror to azure.archive.ubuntu.com in Dockerfiles and test workflow ([e1979b8](https://github.com/loonghao/vx/commit/e1979b820c42dd85c5789aceec71c49c15fc78d8))
* **docs:** escape angle brackets in RFC 0033 headings ([a7f2c43](https://github.com/loonghao/vx/commit/a7f2c434d3cb44789118f38d6741a29a4f6cbf2a))
* **docs:** fix broken doctests in vx-console and vx-starlark ([fbf11dc](https://github.com/loonghao/vx/commit/fbf11dcbd1db7610e801726a3daee579bac33081))
* **engine:** ctx.install_dir now points to actual install location ([7f5c6ed](https://github.com/loonghao/vx/commit/7f5c6ed7ad050e13479b49609a77a75b3af348c9))
* ensure MSVC Spectre component integrity check for already-installed companion tools ([56ed41f](https://github.com/loonghao/vx/commit/56ed41fb456cc7100619687a090a56b206820f6a))
* ensure rust targets are installed in CI ([e09a6cb](https://github.com/loonghao/vx/commit/e09a6cb5805e90f53ecb84808b3d0e8d5cb6eb27))
* exclude vx-star-metadata from cargo-hakari workspace-hack ([43b398d](https://github.com/loonghao/vx/commit/43b398dc0fb9c782788fbb41fe37bbb53c41c980))
* **eza:** add platform_constraint to skip macOS in CI tests ([1d49a16](https://github.com/loonghao/vx/commit/1d49a169e1131f41a60977c16c944a315693ef6d))
* ffmpeg use Gyan.dev mirror, witr only override download_url ([3202629](https://github.com/loonghao/vx/commit/3202629f72155104af9f0a8054b6409b21a546d0))
* **ffmpeg:** use system_install only (remove unreliable GitHub downloads) ([cb3aa49](https://github.com/loonghao/vx/commit/cb3aa49493cdeadf70e6346b30272f6a05f434f4))
* **ffmpeg:** use vx-org/mirrors with BtbN static builds (win64+linux64+linuxarm64) ([a7e7d73](https://github.com/loonghao/vx/commit/a7e7d73fad1554dcbdc8b81e83feeef081a23634))
* filter vault releases by platform artifacts ([51ec2d5](https://github.com/loonghao/vx/commit/51ec2d5b8b1bb89cff952b030e72d515c6b47e7a))
* fix PSReadLine cursor positioning issue in PowerShell prompt ([1a5d995](https://github.com/loonghao/vx/commit/1a5d995a74f49cd39061b41c79dc42ad81c80c14))
* fix system_install providers and starlark test assertions (round 5) ([cabfdeb](https://github.com/loonghao/vx/commit/cabfdeb4fefe2bfe59d5322279ba240fbe3d23e5))
* fix workspace-hack hakari section markers and regenerate dependencies ([7483f3f](https://github.com/loonghao/vx/commit/7483f3f60ed2f1279d59abacd59e3c592cc3e62e))
* flatten InstallLayout JSON so manifest_runtime can read strip_prefix ([71bc81d](https://github.com/loonghao/vx/commit/71bc81d5ab27b4af0db0aa76f2cf08ee84d616b9))
* **gcloud:** update starlark test to use __type field ([9db3a9c](https://github.com/loonghao/vx/commit/9db3a9c45ef97904b53690ef70b99b3648f8ae4c))
* git uses MinGit ZIP on Windows, rust toolchain defaults to stable ([be077cd](https://github.com/loonghao/vx/commit/be077cda730e4e5b972d78b50c633d50fbd4b0c3))
* git Windows exe path, rust bundled store mismatch, lock multi-platform URLs + perf optimizations ([#787](https://github.com/loonghao/vx/issues/787)) ([4640e2c](https://github.com/loonghao/vx/commit/4640e2cd53873256897e08643da18bcc5b5b9425))
* gracefully resolve numeric version hints for pure opaque providers ([ac6c0e4](https://github.com/loonghao/vx/commit/ac6c0e48c1293c04e46bee14915faa2a6a00672a))
* hadolint asset name separator and uvx bundled_with support ([c2e0dd9](https://github.com/loonghao/vx/commit/c2e0dd9c3b2e018f6ae2ed34fbcf6ebfdeb5db2b))
* handle JSON output in where command e2e tests ([c5f7dbe](https://github.com/loonghao/vx/commit/c5f7dbe597b918ab6e935be001f7e67cd75de848))
* handle rust toolchain versions in path selection ([b166e36](https://github.com/loonghao/vx/commit/b166e36b24d635738c41e1970177b3e030a022de))
* handle VX_VERSION=latest in install scripts ([5d30f4a](https://github.com/loonghao/vx/commit/5d30f4ad21236f34f3d2e3e912eba8470a580f91))
* harden release-please and add mcpcall ([5aa3cb1](https://github.com/loonghao/vx/commit/5aa3cb16c8e669894b4ab32692684b7c90af8b6d))
* import env_prepend from env.star instead of provider.star ([0a8503f](https://github.com/loonghao/vx/commit/0a8503fffaf8415d3391861e3265aa7a3c203ef6))
* improve installer fallback and mirror release support ([84e8e35](https://github.com/loonghao/vx/commit/84e8e3597082fa27de2f7f1b52f6c7fdeb4387f5))
* inject parent runtime env for bundled runtimes (npm/node PATH issue) ([f39d51a](https://github.com/loonghao/vx/commit/f39d51a546bcf5d130ae59172273c1b00923807c))
* inject parent runtime PATH for bundled runtimes via spec env_config ([d05e779](https://github.com/loonghao/vx/commit/d05e7796e77848e0ccf6dca86f53e43b965d593f))
* **installer:** apply binary rename fix to RealInstaller.download_with_layout ([1983eb2](https://github.com/loonghao/vx/commit/1983eb2dd9a6ac809f66ac50599c8ab5560ec3f9))
* **installer:** fix PortableGit .7z.exe not recognized as archive + stop version fallback on layout errors ([893806a](https://github.com/loonghao/vx/commit/893806ac6b4afa81a06aaf61951cc4052bd9ff03))
* **install:** prevent awk double-output in resolve_latest_version ([d756244](https://github.com/loonghao/vx/commit/d75624428f8e9c2ef562d14d86c7c3549c683fe1))
* **install:** skip releases without binary assets in version resolution ([46553eb](https://github.com/loonghao/vx/commit/46553eb6b34c52f1d0ae71fe4d846ead43ded6aa))
* **just:** correct version_pattern to match 'just X.Y' output ([5238a21](https://github.com/loonghao/vx/commit/5238a21c27605e76593f7f03ad0890f8472356b6))
* **justfile:** fix test-pkgs recipe to not duplicate -p flag ([6bc8aea](https://github.com/loonghao/vx/commit/6bc8aeaefe58512e027f647896ed15024cfb98d0))
* **lint:** resolve provider.star lint issues ([6166bb3](https://github.com/loonghao/vx/commit/6166bb32e5fc7bf47c53f59a1fb2b8e8445fd665))
* **macos:** make sevenz-rust optional to fix macOS build ([6f7d846](https://github.com/loonghao/vx/commit/6f7d8461d72aa4b53d4241d1d54d0dfb31c225b3))
* make E2E version list tests resilient to transient network errors ([2367e94](https://github.com/loonghao/vx/commit/2367e940b9832a249a7b7d5243c20407a6ee16ea))
* make env-dependent tests serial to prevent race conditions ([e15ee51](https://github.com/loonghao/vx/commit/e15ee51811dd0af0c2f9cc007514f0691b4b5cc4))
* **manifest-runtime:** override resolve_version to return 'system' for system tools ([ced6833](https://github.com/loonghao/vx/commit/ced6833db66c2b32e6479a2e8a29b0a741f5fc46))
* **mise:** avoid strip_prefix on Windows to prevent Access Denied errors ([b1958c9](https://github.com/loonghao/vx/commit/b1958c94bfa3031f2e8edc6f772661a4607da633))
* **mise:** update unit tests to match new install_layout implementation ([3f7d91b](https://github.com/loonghao/vx/commit/3f7d91b549378eb4684157d7056ae1e611076bfa))
* **mise:** use strip_prefix='mise/bin' on Windows to avoid shim detection error ([1703d7f](https://github.com/loonghao/vx/commit/1703d7f416e0e7664ad5ba3cb4f4916f520f9144))
* **paths:** detect unified runtime store versions ([0d3d7cb](https://github.com/loonghao/vx/commit/0d3d7cb24819ea891dfb736157e0de57f77101cf))
* prepend node_modules/.bin to PATH for npm/npx execution ([#906](https://github.com/loonghao/vx/issues/906)) ([f639b2a](https://github.com/loonghao/vx/commit/f639b2ad1e441d65904702ed3a5372adb9ef66f6))
* preserve Rust MSRV in vx.toml and enable passthrough for Rust ecosystem ([bec646a](https://github.com/loonghao/vx/commit/bec646a7ac32be27a4ffb9abbbc9e325edbe448f))
* prevent bundled runtime executable misresolution (npm-&gt;node) ([0f50874](https://github.com/loonghao/vx/commit/0f50874619de31e7b9672cf42afc1313c1b9f6ac))
* prevent repeated MSVC component re-installation when Spectre libs unavailable ([94363bf](https://github.com/loonghao/vx/commit/94363bff59b36cfb42d6fa76c247c7c042f458c5))
* propagate locked version to bundled runtime dependencies ([b3a889b](https://github.com/loonghao/vx/commit/b3a889b728959502d690fe03a5a5bcb38df9463c))
* **provider:** correct grpcurl version check ([e6ce2b1](https://github.com/loonghao/vx/commit/e6ce2b180dd55a56e5e0700e2b67dbc553ec14f6))
* **providers:** add fetch_versions_with_tag_prefix to layout mock + fix cargo-deny Windows ([267ca08](https://github.com/loonghao/vx/commit/267ca08d49e53056dd4e52a5d80aea2f11fb5b89))
* **providers:** correct install_layout strip_prefix and download_url ([18034e5](https://github.com/loonghao/vx/commit/18034e5271040afd5debef5ac61772fd5015bbab))
* **providers:** correct mirror tag version fetching ([e2a5979](https://github.com/loonghao/vx/commit/e2a59796f31a36cfb3b4cfae2bb19d042bb03c16))
* **providers:** fix 5 more provider bugs (round 2) + add tar.bz2 support ([9726195](https://github.com/loonghao/vx/commit/97261951fd75f6b422212704b6260903ec3cb50c))
* **providers:** fix 5 provider bugs from auto-improve branch ([85b51f2](https://github.com/loonghao/vx/commit/85b51f22c3c92ce82ae35d46cb5e36314763c12b))
* **providers:** fix binary rename, grpcurl macOS, duckdb macOS, nerdctl platform ([007f016](https://github.com/loonghao/vx/commit/007f016bfff63cc843a832b93be8230c158ae291))
* **providers:** fix cargo-nextest macOS triple and cargo-audit test assertions ([9c8941e](https://github.com/loonghao/vx/commit/9c8941e917622276fc8c93daafbac36c1611b824))
* **providers:** fix download URL bugs in git, xmake, and ollama ([#777](https://github.com/loonghao/vx/issues/777)) ([5cebab7](https://github.com/loonghao/vx/commit/5cebab7a7baa2d9a9b2ffaf13c80f13fd4da091e))
* **providers:** fix download URLs for cargo-audit, cargo-nextest, and deno ([8113e63](https://github.com/loonghao/vx/commit/8113e636dff4fc6028f208c26ad36fd516a473d2))
* **providers:** fix dust and eza macOS download URL 404 ([ee5bc77](https://github.com/loonghao/vx/commit/ee5bc779adafd69965ba7c87b8cd7ece198b99fc))
* **providers:** fix dust version pattern and tealdeer binary rename ([eb7c3f6](https://github.com/loonghao/vx/commit/eb7c3f6fde473e3d71bb9ad788119ed27d6e26b1))
* **providers:** fix gcloud get_execute_path and terraform fetch_versions ([f0b5d9c](https://github.com/loonghao/vx/commit/f0b5d9cf38c71ad3d2c07214bd83e15782ec187b))
* **providers:** resolve CI download URL failures ([9067375](https://github.com/loonghao/vx/commit/906737558ab53cf9312018b9d603929e4a82f634))
* **providers:** resolve CI issues for new provider batch ([8aa8967](https://github.com/loonghao/vx/commit/8aa89675695d32f212e77d07f811439bf60dbb29))
* **providers:** resolve tealdeer and mise install layouts ([3c2d42a](https://github.com/loonghao/vx/commit/3c2d42a1a2fd72d3ac3759fca6b5c35c9259c214))
* **providers:** use vx-org/mirrors for ffmpeg and witr downloads ([9332423](https://github.com/loonghao/vx/commit/933242301d4b97030c8b3aa797fdf438d8912409))
* **provider:** use gnu rustup triples on linux ([9bee93d](https://github.com/loonghao/vx/commit/9bee93ddcecad6a4f4dd7405f9b12f02069d2347))
* Python install fails due to version_date cache key mismatch ([2edb836](https://github.com/loonghao/vx/commit/2edb836bbc616e521d90863d0d4fb68f1a72fbad))
* **release:** disable sccache rustc-wrapper in release workflow ([248bfdf](https://github.com/loonghao/vx/commit/248bfdfc3084524c0b9ad731ca95f66b3502f16e))
* remove BOM from all provider.star files and improve star syntax checker ([75f14df](https://github.com/loonghao/vx/commit/75f14dfe8d2683624a7e355c5b5390fd856c14a9))
* remove platform subdir from install path, fix providers ([6f5fe38](https://github.com/loonghao/vx/commit/6f5fe386af74efc907249f1a130a09336fb59347))
* remove unused loads and fix lint issues in provider.star files ([cf28c19](https://github.com/loonghao/vx/commit/cf28c190de986994b0496aa6e4228c42da196b7b))
* remove unused variables in witr/provider.star ([cc7133d](https://github.com/loonghao/vx/commit/cc7133d0bf2d58e6acae0245caaa2e6327018f20))
* remove windows-sys 0.59 and merge features into 0.61 in workspace-hack ([336e2d8](https://github.com/loonghao/vx/commit/336e2d8d7d072a852e91d2fdb7c633d389fc4750))
* repair provider test resolution and platform gating ([657467e](https://github.com/loonghao/vx/commit/657467eafa78be1adc19ffd21e9e743e4f58ee96))
* repair vx store executable permissions ([6393c22](https://github.com/loonghao/vx/commit/6393c224f37efce87d4100a762099df4745d7c0a))
* replace all ctx dict access with struct attribute access in provider star files ([163be8c](https://github.com/loonghao/vx/commit/163be8c21828ed3dc18b90012d1ab7268d80aa27))
* replace all ctx.http.get_json with fetch_json_versions descriptors in provider.star files ([7f9238c](https://github.com/loonghao/vx/commit/7f9238ce667e793fead37dc999dcf927439c2f70))
* resolve .cmd executables for bundled runtimes on Windows ([bd12254](https://github.com/loonghao/vx/commit/bd122542ed0944b379aceda04a4d4f7279c813eb))
* resolve CI errors ([0a30bed](https://github.com/loonghao/vx/commit/0a30bed957293bdb745255fbf4f16318f3cbb91d))
* resolve CI errors ([454cb9b](https://github.com/loonghao/vx/commit/454cb9bed0b08d531926875a61561b7e4eff8a6c))
* resolve CI failures for imagemagick, ffmpeg, rez, bash, make, nasm ([5880381](https://github.com/loonghao/vx/commit/5880381eb13b0767b186d3a9942c04ec2c30c05b))
* resolve CI failures for lefthook, grpcurl, and kustomize providers ([77a0d0f](https://github.com/loonghao/vx/commit/77a0d0fd4f3b515cca694db0fabd72146d512f28))
* resolve CI failures for yq, wix, xmake, vcpkg providers ([1be437f](https://github.com/loonghao/vx/commit/1be437fbe4197ea3e80f31eabfece3faaca74bd6))
* resolve clippy warnings and test assertion ([0234041](https://github.com/loonghao/vx/commit/0234041d4a513899a484dc2b0774c7aa085391e8))
* resolve compiler errors in test files ([5e97fd2](https://github.com/loonghao/vx/commit/5e97fd2b0e057d1b9bb2bfcd1b7f1919325ff9f3))
* resolve Linux CI failures for ffplay/ffprobe/gofmt/lld/xmake/yq ([3c8bf60](https://github.com/loonghao/vx/commit/3c8bf609c8785b02a8134b8e59cd2db943d6cbe6))
* resolve macOS CI failures for ffmpeg and imagemagick ([3e9bcc8](https://github.com/loonghao/vx/commit/3e9bcc824436df6d9404ecd7e9a11b8f006c2946))
* resolve merge conflict in release manifest ([0e35d29](https://github.com/loonghao/vx/commit/0e35d293dca2da7918cdc4356ac7caf0d50d5bdf))
* resolve Python PYTHONHOME mismatch ([#696](https://github.com/loonghao/vx/issues/696)), improve version pagination, unify skills ([e477508](https://github.com/loonghao/vx/commit/e47750841f90045027114e4eff3dc59f593c0f13))
* resolve sha2 LowerHex compile errors and upgrade GitHub Actions ([c0ffcc9](https://github.com/loonghao/vx/commit/c0ffcc954dc4c59ad40d42c32819d39fb766d42f))
* **resolver:** resolve bundled runtime fallback executable ([c97dd94](https://github.com/loonghao/vx/commit/c97dd94ea094f1612e7ab529c737794c36662f49))
* **resolver:** stop re-installing system-managed runtimes on every vx invocation ([f1170a5](https://github.com/loonghao/vx/commit/f1170a56c7792a4f19e8df75c312106561c140e2))
* **runtime:** check vx store first in ManifestDrivenRuntime.is_installed() and installed_versions() ([1966339](https://github.com/loonghao/vx/commit/19663398517146e231900c9839502df7de63c245))
* **runtime:** preserve bundled command prefixes ([9c6ef0e](https://github.com/loonghao/vx/commit/9c6ef0e6715068669aa7cb02040af5330315ceb4))
* **runtime:** satisfy clippy collapsible-if lint ([198342f](https://github.com/loonghao/vx/commit/198342fc67b4c6ce88ed35c756cd1ff24bb692d6))
* Rust ecosystem passthrough for rustc versions in resolve_version ([6228793](https://github.com/loonghao/vx/commit/6228793b2f55587f4f0583a14855e2184510c40f))
* **rust:** stop re-installing rust on every vx cargo invocation ([e6156b4](https://github.com/loonghao/vx/commit/e6156b4dc6da569027da119ef2f468ef29b08002))
* serialize runtime installs ([efa3d88](https://github.com/loonghao/vx/commit/efa3d889af893529568186b73c56db3b864c4ff9))
* skip broken micromamba windows release ([b31a218](https://github.com/loonghao/vx/commit/b31a2187030bb980b8005ecca7647b8a9153479f))
* stabilize test suite and version constraint parsing ([066155f](https://github.com/loonghao/vx/commit/066155fb8bcd66e252f8f84c9be24f735f5bb6a5))
* **starlark:** lower provider loading log level from info to debug ([b8947a5](https://github.com/loonghao/vx/commit/b8947a5e18cbd0a0e954b2b02f36ed2e2adb60fc))
* **starlark:** register all 14 stdlib modules in loader ([20e0ed8](https://github.com/loonghao/vx/commit/20e0ed87dfbf77a2e147741d5d0f96fc49b0e511))
* switch macOS FFmpeg download source from evermeet.cx to osxexperts.net ([4bc40be](https://github.com/loonghao/vx/commit/4bc40be718c53ce41a9f20f4eb19e00ab4f808df))
* temp_dir unbound variable in install.sh and uv strip_prefix ([e24a3fa](https://github.com/loonghao/vx/commit/e24a3fa536ae0ff2e13f27398ee0dd7e4091b26a))
* **test:** add missing package_alias field in manifest_registry_tests ([ce6614a](https://github.com/loonghao/vx/commit/ce6614ad0004ec764c1eb2c45e4a0e9b7bad762d))
* **test:** add missing package_alias field in ProviderMeta test ([b5382bf](https://github.com/loonghao/vx/commit/b5382bfdefa17f495b281a42d5be52fa748dda8b))
* **tests:** add missing OutputFormat argument to handle_list calls ([2c44db1](https://github.com/loonghao/vx/commit/2c44db1643f4cc157471f3a540dddc134c383e78))
* **tests:** fix 14 failing tests across multiple crates ([af003de](https://github.com/loonghao/vx/commit/af003dec15e905e8c426657d85d1a3da6f8cf717))
* **tests:** fix cargo-deny Windows URL test and add missing provider tests ([27e03ad](https://github.com/loonghao/vx/commit/27e03ad7990565133b493d0c97b45a268ceccdec))
* **tests:** fix output_tests and info_tests failures in non-TTY CI ([414bb8a](https://github.com/loonghao/vx/commit/414bb8ac55f722b5b5d1f268995d2275c757663b))
* **test:** skip package_alias providers in CI tests ([9a7c3a4](https://github.com/loonghao/vx/commit/9a7c3a4416ffc467da9f68c1189b2d618f957532))
* **tests:** relax assertion in test_vx_toml_python_setup_dry_run ([1cc2979](https://github.com/loonghao/vx/commit/1cc2979bb6122fefff4791f96f760f2691673882))
* **tests:** resolve latest unit test failures ([0442882](https://github.com/loonghao/vx/commit/04428827543364ee71ff0468f25ce9a4c643ff0b))
* **tests:** rewrite all provider runtime_tests to use create_provider() API ([0ae8cba](https://github.com/loonghao/vx/commit/0ae8cbad0fd77f80d0cb1a01ec61038a8d2b95f8))
* **ui:** show Installing feedback during auto-install to avoid perceived hang ([560684e](https://github.com/loonghao/vx/commit/560684e6240f66ee627aef714cec738d56f6eb78))
* unblock remaining CI regressions ([b5498a1](https://github.com/loonghao/vx/commit/b5498a12ba943ad33ef45aff2fb842dd7f960191))
* use bin/bash.exe for git-bash instead of git-bash.exe --attach ([dbe9be4](https://github.com/loonghao/vx/commit/dbe9be4e396039653e4b26349d54fb6781cc360b))
* use child version for bundled proxy runtime installation ([e97e333](https://github.com/loonghao/vx/commit/e97e33341e903a2f7324cde0d0593059a1244fcc))
* use struct attribute access ctx.platform.os instead of dict access ctx[platform][os] in stdlib star files ([b8f4c93](https://github.com/loonghao/vx/commit/b8f4c931d210643effccf73a8d86dd4c58fb92f6))
* use system_install for ffmpeg Linux, fix witr install_layout ([56e6832](https://github.com/loonghao/vx/commit/56e6832308e0641d65aa4c161049527c00adab88))
* **uv:** route uvx through uv tool run ([53b038f](https://github.com/loonghao/vx/commit/53b038f6ff0f2a329e7aab391dbbe6b0a13d673a))
* **versions:** case-insensitive Ecosystem deserialization for vx.lock compatibility ([7f6e2ed](https://github.com/loonghao/vx/commit/7f6e2eda83ed7cdf07c946ba1b8ba85ea7c8cf8d))
* **vx-config:** fix sha2 GenericArray LowerHex compile error ([7aa358e](https://github.com/loonghao/vx/commit/7aa358e95c05ed3393c9fe110e9e04d737d246cf))
* **vx-provider-jj:** strip v prefix from version tags to prevent double-v in download URL ([67486e1](https://github.com/loonghao/vx/commit/67486e193e9d52e07b2e9b64ae844885f741f553))
* **watchexec:** remove unused load imports to pass provider static lint ([f4b7330](https://github.com/loonghao/vx/commit/f4b7330556201d7e32890e8e7987b978195c9cd9))
* **watchexec:** use .zip on Windows, .tar.xz on Linux/macOS ([ef58118](https://github.com/loonghao/vx/commit/ef58118c784259317c40fc431a0678b3e1c6119d))
* **where:** use executable_name() instead of runtime name for exe lookup ([ee5bfb4](https://github.com/loonghao/vx/commit/ee5bfb4cdb2bf00a890ebcb715312b5c60473954))
* **windows:** resolve OS error 193 when executing bundled runtimes (npm/npx) ([9aacc52](https://github.com/loonghao/vx/commit/9aacc5290cab2f5d04ad62f3d146be3abc2d8dc6))
* wire Starlark post_extract hooks into ManifestDrivenRuntime and fix bundled runtime detection ([17db1a4](https://github.com/loonghao/vx/commit/17db1a41e524bf1381d7a4c1858a0a9871978344))
* **witr:** correct __type__ key in install_layout (double underscores) ([b1e11b2](https://github.com/loonghao/vx/commit/b1e11b2757cd02dc7440f92c4ab4d44091c805e2))
* **witr:** correct version pattern and binary path in provider.star ([b19248d](https://github.com/loonghao/vx/commit/b19248d4421b184db2f0a0f58901ccb7e3eafdea))
* **witr:** override install_layout with correct type ([bc5cc35](https://github.com/loonghao/vx/commit/bc5cc350d6983e0b84b1380ffee4590ea1506279))
* **witr:** rewrite provider without template to handle direct binaries correctly ([4c3b525](https://github.com/loonghao/vx/commit/4c3b52563245c30bbd33724fe553056ae8f3f694))
* **witr:** use 'binary' type for direct binaries (Linux/macOS) ([825b55a](https://github.com/loonghao/vx/commit/825b55a08402d2dae35943f9872b1665fcb4213a))


### Performance Improvements

* add cargo build optimization agent rule ([342f16d](https://github.com/loonghao/vx/commit/342f16daeb6cd2d549c595bdbd4433054d683bda))
* implement cargo-hakari workspace-hack + runtime/config refactoring ([b3d6597](https://github.com/loonghao/vx/commit/b3d65973fd1f0c00de19903f8f5f89172d143d34))
* optimize test and build configuration ([06727c5](https://github.com/loonghao/vx/commit/06727c54e2730bc72ee599230fb4277eca64230a))
* optimize workspace compilation settings ([d30fae3](https://github.com/loonghao/vx/commit/d30fae3bb0fe70e82cbb1fcea6b14c0c7872e7b6))


### Code Refactoring

* **build:** remove legacy provider.toml support, simplify build.rs and registry.rs ([a0cb55b](https://github.com/loonghao/vx/commit/a0cb55bb4038333df777ee065fcb2e8b95c8890b))
* **cli:** update commands and test utilities for runtime refactoring ([8e80df3](https://github.com/loonghao/vx/commit/8e80df37b279f3ae184435ddf6cc50b697c0a455))
* **env,version-fetcher:** eliminate platform/version utils duplication ([d271ac8](https://github.com/loonghao/vx/commit/d271ac834308f4d7a9da7b97ecd1ccc8ba7b119a))
* extract vx-star-metadata crate and eliminate Box::leak usage ([160a618](https://github.com/loonghao/vx/commit/160a61892c3070d85c8f51b30bca83f949737271))
* improve code quality - replace unwrap() and eprintln! with proper error handling ([3ac83d1](https://github.com/loonghao/vx/commit/3ac83d1c5507c79c6fdacbbde6b8bd14ab9b9bee))
* improve code safety and remove dead code ([92a5af7](https://github.com/loonghao/vx/commit/92a5af7716a0f9cafe85642ed64312f6062995c7))
* improve code safety by eliminating unsafe unwrap calls ([bf838c3](https://github.com/loonghao/vx/commit/bf838c3f8547882052acbb523370cadf08730c97))
* merge vx-core into vx-runtime-core ([1208488](https://github.com/loonghao/vx/commit/12084885b31a548419f89e15e407ad7fe00abc38))
* optimize provider.star files using stdlib templates ([646093f](https://github.com/loonghao/vx/commit/646093f24f469e4d6394f21533d9146acb01cce2))
* **provider:** conda use provider.star only (remove Rust code) ([ce6b469](https://github.com/loonghao/vx/commit/ce6b469004294b3c3b83d2fb892d71608ee9966b))
* **providers:** replace all hand-written permissions dicts with stdlib helpers ([d49da4d](https://github.com/loonghao/vx/commit/d49da4d0b79efb487adb678deecd7535faef09a7))
* **providers:** simplify providers to use standard templates ([92a1179](https://github.com/loonghao/vx/commit/92a117985764c95f43f5ac56ab19ed9c72a8d232))
* replace bare .unwrap() with descriptive .expect() in production code ([991b4e2](https://github.com/loonghao/vx/commit/991b4e21d7db7c6532967b101df83df4c525d814))
* **resolver:** integrate ResolutionCache into execution pipeline ([03d6864](https://github.com/loonghao/vx/commit/03d6864d60427ce71d59834a263978522d5f081f))
* **runtime-core:** remove dead Runtime trait and provider machinery ([f9659e1](https://github.com/loonghao/vx/commit/f9659e1ff192cb105f7737f0bdd1d865cd927a52))
* **runtime:** split runtime.rs into module and add ISP sub-traits ([93a1f6d](https://github.com/loonghao/vx/commit/93a1f6d399f3db27359753884e4656659798e877))
* simplify all providers to PROVIDER_STAR only, remove redundant create_provider and star_metadata ([adf3484](https://github.com/loonghao/vx/commit/adf34845d2ebbdc49331c4403fe58c31a9ab6e56))
* split tests to tests/ dir, extract bridge/builder modules, remove metadata indirection, fix clippy warnings ([064af9a](https://github.com/loonghao/vx/commit/064af9aa324a8b8661414c0650d32b4b369e1ccf))
* unify progress bars and restructure docs progressively ([#812](https://github.com/loonghao/vx/issues/812)) ([679d1da](https://github.com/loonghao/vx/commit/679d1da0874a5a36bb859ee7736eb1b0be687733))
* use LazyLock for regex compilation and improve error handling ([c0c9946](https://github.com/loonghao/vx/commit/c0c994669c6cafd1fc72e3fefade9ec56954dff5))
* **vx-starlark:** replace path-based cache with content-hash incremental analysis cache ([3acb930](https://github.com/loonghao/vx/commit/3acb9302421ec23e295a70b149d553fedad7f7ac))


### Documentation

* add age and sops to tools overview and CHANGELOG ([01559da](https://github.com/loonghao/vx/commit/01559da2847e517ac8378f2dcc45d6471d97fef9))
* add CLAUDE.md, .cursor/rules/*.mdc, and improve AI agent documentation ([#747](https://github.com/loonghao/vx/issues/747)) ([2ffce28](https://github.com/loonghao/vx/commit/2ffce2828212198edf61680debcb01e3459f5119))
* add complete Supported Tools section to llms-full.txt ([44e7ec2](https://github.com/loonghao/vx/commit/44e7ec25475485b24130f287f5f6ba4818ad762c))
* add critical rules section to AGENTS.md for AI agents ([#869](https://github.com/loonghao/vx/issues/869)) ([dd5e49d](https://github.com/loonghao/vx/commit/dd5e49d6680c8f9e17e8163026988f73ae87c70b))
* add latest RFCs (0037, 0039, 0040) to llms-full.txt ([#873](https://github.com/loonghao/vx/issues/873)) ([adcc9a6](https://github.com/loonghao/vx/commit/adcc9a6ebd5fde09db3867fdb54a696e30596a30))
* add llms.txt and llms-full.txt following llmstxt.org protocol ([4ab49e9](https://github.com/loonghao/vx/commit/4ab49e934214ff1167d0feb97b75269e5b2fc729))
* add missing tools (actrun, ctlptl, gws) to documentation ([11ea805](https://github.com/loonghao/vx/commit/11ea8050bbb8867abb863a0b28fed112a7d17169))
* add missing tools to documentation ([4d223d3](https://github.com/loonghao/vx/commit/4d223d362d5b0bcbd330c467d780fd786f8c8460))
* add more tool examples to AGENTS.md ([38b188c](https://github.com/loonghao/vx/commit/38b188c813420849d7bb1bf2431dd5ded40ca41b))
* add Package Alias documentation (EN + ZH) ([eaf81e3](https://github.com/loonghao/vx/commit/eaf81e39067540c7af3dd4541f14c9fcee357730))
* add pre-commit hooks documentation (EN/ZH) and update contributing guides ([274aec5](https://github.com/loonghao/vx/commit/274aec5a0e85be9dc6f266437ee0077b31bf845d))
* add self-update command documentation with channel support ([e46ad5c](https://github.com/loonghao/vx/commit/e46ad5c4270a9f3b4e7dea6377443854aabb5d96))
* add Starlark Providers advanced guide (bilingual) ([38739c0](https://github.com/loonghao/vx/commit/38739c09be752fdf2cc460f3e2cd8938fc29651a))
* add worktrunk (wt) tool documentation ([d4dfae6](https://github.com/loonghao/vx/commit/d4dfae6a364128ac1edd9612466f6aeedd43e958))
* **agent:** add cross-language global install contract and fix RFC links ([290f089](https://github.com/loonghao/vx/commit/290f08941cf6fc823226d44ac4bb9aa80a59c133))
* **cargo:** add fast build optimizations inspired by Bevy ([31a0588](https://github.com/loonghao/vx/commit/31a0588021b353f3612a70d84f0923b2e4e09437))
* **cleanup:** sync provider count from 105 to 111 across all docs ([79ca181](https://github.com/loonghao/vx/commit/79ca181a3d1906c72910f820eded1885bcf4a30a))
* enhance AI agent documentation and sync skills ([#736](https://github.com/loonghao/vx/issues/736)) ([043b7aa](https://github.com/loonghao/vx/commit/043b7aaca2bffb3a9069a1cd76d295df7976fc8c))
* enhance AI agent documentation with decision framework, MCP guide, and version fixes ([#732](https://github.com/loonghao/vx/issues/732)) ([e806f1f](https://github.com/loonghao/vx/commit/e806f1f57fc76af44be705b78da1028b76903e72))
* enhance AI agent ecosystem with 15+ agent support ([#749](https://github.com/loonghao/vx/issues/749)) ([fff3c42](https://github.com/loonghao/vx/commit/fff3c421e9925ad0ea3762655e12f5b5865c8a5f))
* fix dead links in docs build ([faa8709](https://github.com/loonghao/vx/commit/faa87094f618837a9150ea7ef39d76f93b0065a8))
* fix duplicate Critical Rules sections in AGENTS.md ([#870](https://github.com/loonghao/vx/issues/870)) ([18b0a6d](https://github.com/loonghao/vx/commit/18b0a6db5eb8995f52cf158101840f81501fe668))
* fix syntax errors in other.md and quality.md ([#866](https://github.com/loonghao/vx/issues/866)) ([a07534c](https://github.com/loonghao/vx/commit/a07534c28456a5a5304f3e8a6bf2656e78bb66e2))
* improve agent documentation for better AI discoverability ([#701](https://github.com/loonghao/vx/issues/701)) ([ca2bfe7](https://github.com/loonghao/vx/commit/ca2bfe7fd4df5ce59dae07b685512fdaa41cb669))
* improve agent knowledge - update provider count to 78, enhance AGENTS.md, sync skills ([#687](https://github.com/loonghao/vx/issues/687)) ([2686f86](https://github.com/loonghao/vx/commit/2686f86e3b610181f9e287b8af90274299190a5f))
* improve agent knowledge - update provider.star docs, fix tool counts, add creating-provider guide ([cdfcf32](https://github.com/loonghao/vx/commit/cdfcf325add5ba559ce505b882e99ef576e9e2ed))
* improve AI agent documentation and fix version inconsistencies ([#710](https://github.com/loonghao/vx/issues/710)) ([72389b7](https://github.com/loonghao/vx/commit/72389b7f8efa5c891c4b3b22ad9220ee11017db1))
* improve AI agent documentation ecosystem ([#741](https://github.com/loonghao/vx/issues/741)) ([59c4f12](https://github.com/loonghao/vx/commit/59c4f127f2b81b86bc4d875c91a42bf22814270c))
* optimize agent docs for v0.8.20 — add Copilot instructions, expand AI agent support to 17+ ([#762](https://github.com/loonghao/vx/issues/762)) ([0430e70](https://github.com/loonghao/vx/commit/0430e70daf43338c5de0bb07c7fc60bb5ea5e01f))
* optimize AGENTS.md as progressive disclosure map ([#868](https://github.com/loonghao/vx/issues/868)) ([954f3fd](https://github.com/loonghao/vx/commit/954f3fd2adf9307b57a0e5c0e0540f2e56420359))
* **rfc-0032:** update Plan D (hakari implemented), Plan E/F tracking status ([22794da](https://github.com/loonghao/vx/commit/22794dae2c3a2543470727c68f746eafe7633f2f))
* **rfc:** add RFC 0036 - Starlark Provider Support ([8501be4](https://github.com/loonghao/vx/commit/8501be436ba06d175ed82aa4b0ea14b28acaf008))
* **rfc:** update Phase 2 progress in RFC 0032 ([b10be5a](https://github.com/loonghao/vx/commit/b10be5ae44beb5dc447ab52d332e7e9dc73057cf))
* **rfc:** update Phase 2 status in RFC 0032 ([b7f69d8](https://github.com/loonghao/vx/commit/b7f69d840b224c879dda900ac3d9d7da57b19089))
* **rfc:** update RFC 0036 v0.3 - add Buck2 typed provider_field, load() module system, incremental analysis cache, declarative actions ([46a939c](https://github.com/loonghao/vx/commit/46a939ca40104e839955176d470c27454dd64f83))
* simplify AI agent configs, add vx wt/witr examples ([eee6ae0](https://github.com/loonghao/vx/commit/eee6ae0d833f156321fc1b002a0e78f5a004730e))
* **skill:** add new vx capabilities to SKILL.md ([bf5516b](https://github.com/loonghao/vx/commit/bf5516b3817be223697fe0c6395d0257d8560484))
* sync zh contributing.md and add zh fixes docs ([e2dbe08](https://github.com/loonghao/vx/commit/e2dbe08d136d45f82f8517db43dd1ab7185e3a0d))
* update cargo-build-optimization agent rule with implemented optimizations ([cdf3623](https://github.com/loonghao/vx/commit/cdf3623b686a0b109618d6c609e7e8a9baca7605))
* update documentation for self-update and worktrunk ([4461ee6](https://github.com/loonghao/vx/commit/4461ee621d32adc1f60866a2923653a005c8a1a9))
* update media.md and witr.md with vx-org/mirrors download source ([#864](https://github.com/loonghao/vx/issues/864)) ([7f4cadd](https://github.com/loonghao/vx/commit/7f4cadd1169b0a48a973015d9a927ad6e8fbd156))
* update outdated version numbers in README.md ([#874](https://github.com/loonghao/vx/issues/874)) ([b181117](https://github.com/loonghao/vx/commit/b18111710d2aaf58f483b5f96ca41199bd2f0d8d))
* update provider count 136 -&gt; 137 (add witr) ([#859](https://github.com/loonghao/vx/issues/859)) ([e039934](https://github.com/loonghao/vx/commit/e039934f3ba91bb5d4a1ec2aff68eab474928192))
* update provider count from 129/131 to 132 ([267444b](https://github.com/loonghao/vx/commit/267444bcabb0b187573673fdcdc4ca62bdf1d3bc))
* update provider count from 129/131 to 132 across all documentation ([09a51bc](https://github.com/loonghao/vx/commit/09a51bce9621c24925e8b992eac2aadcfa2bc832))
* update provider count from 132/135 to 136 across all docs ([011559f](https://github.com/loonghao/vx/commit/011559f0e759273ea728eb31ea67087b8dce4a4c))
* update provider count from 132/135 to 136 across all docs ([282e5f6](https://github.com/loonghao/vx/commit/282e5f62d86a51379a2452026c22514229524c30))
* update provider count from 136 to 137 across documentation ([#865](https://github.com/loonghao/vx/issues/865)) ([cf54ea1](https://github.com/loonghao/vx/commit/cf54ea11499f9f50c262824025a363ee1cebe0f2))
* update provider count to 129 in AGENTS.md ([2c0a554](https://github.com/loonghao/vx/commit/2c0a55414f05bab77272113db060ae215577d07e))
* update provider count to 131 in AGENTS.md ([b7fd52d](https://github.com/loonghao/vx/commit/b7fd52dc69b93cbb5a3f9fd4d66effd8c2e9fa11))
* update provider count to 132 and add conda/micromamba/mamba ([8775174](https://github.com/loonghao/vx/commit/8775174bce7b96dca5ec6256070b3ea70bcbc54c))
* update provider count to 132 and add conda/micromamba/mamba to overview ([f3d166c](https://github.com/loonghao/vx/commit/f3d166ceb3bce2d45d1f6221838ea2d14788feae))
* update provider count to 137 (add witr) ([bf1d258](https://github.com/loonghao/vx/commit/bf1d258dc45eef35402707a6cff87deadfb403e1))
* update rules to reflect provider.star migration ([ef14534](https://github.com/loonghao/vx/commit/ef14534f4b9a69b6521944f5a5b924b075c42454))
* update Rust version requirement in contributing.md ([bc36969](https://github.com/loonghao/vx/commit/bc369690650101ecfbf7fa9c8ec388dcf67df90d))
* update Rust version requirement to 1.93+ (Edition 2024) in contributing.md ([60b0b95](https://github.com/loonghao/vx/commit/60b0b95c7308b5a49c45e0612f5b9c3104675d13))
* update Rust version to 1.95.0+ and fix RFC count ([#875](https://github.com/loonghao/vx/issues/875)) ([7308b00](https://github.com/loonghao/vx/commit/7308b00430e1e01d79dda4ed4e899e5aa4de78ba))
* update tools overview and quality documentation ([9194aaa](https://github.com/loonghao/vx/commit/9194aaa76285a42ad991e0cf5f613d66c7d4cdda))
* update version to 0.8.32 and provider count to 129 ([04b6425](https://github.com/loonghao/vx/commit/04b6425724d43929f82f5f01e5bd73d73dd43909))
* update version to v0.8.35 and provider count to 136 ([379f304](https://github.com/loonghao/vx/commit/379f304c33400eefc771212a044bffeac0813e1c))
* update version to v0.8.35 and provider count to 136 ([33b6a68](https://github.com/loonghao/vx/commit/33b6a68a9933ab4dcdddc936df61ec7cede46c23))
* update version to v0.8.36 and provider count to 136 ([dd477e2](https://github.com/loonghao/vx/commit/dd477e2127698d2a0c21a0536f3c16f8fe27f583))
* update version to v0.8.36 and provider count to 136 ([ef17d93](https://github.com/loonghao/vx/commit/ef17d936df00c3b3f9c0c036a09a1234368c2ecd))
* update version to v0.8.37 ([a1c5766](https://github.com/loonghao/vx/commit/a1c5766364251907d5c4e8027be8c0d4199ed7a7))
* update version to v0.8.37 in AGENTS.md and README.md ([7fa23fe](https://github.com/loonghao/vx/commit/7fa23fea1667da11729d2924670e49af259e9411))
* update version to v0.8.39 ([1dae31d](https://github.com/loonghao/vx/commit/1dae31d80702b38d6d963a9c25ae0b0dd1098a69))
* update version to v0.8.39 in AGENTS.md and README.md ([7deb825](https://github.com/loonghao/vx/commit/7deb825b779fa5e6e7e47172d3223d7ef736eeaa))

## [0.9.3](https://github.com/loonghao/vx/compare/v0.9.2...v0.9.3) (2026-05-25)


### Features

* add python 3.7 and wasm runtimes ([4e5f011](https://github.com/loonghao/vx/commit/4e5f01140b2b8d1c392416eb73e7223ae56d417e))

## [0.9.2](https://github.com/loonghao/vx/compare/v0.9.1...v0.9.2) (2026-05-25)


### Features

* add rust wasm providers ([caa7afd](https://github.com/loonghao/vx/commit/caa7afd606701d23fff8a94883f2444ce79d64a4))

## [0.9.1](https://github.com/loonghao/vx/compare/v0.9.0...v0.9.1) (2026-05-25)


### Bug Fixes

* correct metrics calculations ([05c35f1](https://github.com/loonghao/vx/commit/05c35f12e7dedcfb87f88b43f650b649176b3fc8))
* harden release-please and add mcpcall ([e62080b](https://github.com/loonghao/vx/commit/e62080b95d78e68ab63816368e8e3962b070527d))
* repair vx store executable permissions ([63e190a](https://github.com/loonghao/vx/commit/63e190a38302c5b2c2b3e4dd197caff144e2fe80))
* serialize runtime installs ([0b33913](https://github.com/loonghao/vx/commit/0b3391333f9de350fc84ca9fa78d3eb8e2fb3a96))

## [0.9.0](https://github.com/loonghao/vx/compare/v0.8.39...v0.9.0) (2026-05-24)


### ⚠ BREAKING CHANGES

* migrate providers, add bridge system, fix Windows env injection
* migrate providers, add bridge system, fix Windows env injection

### Features

* add 11 new providers (mise, gitleaks, biome, lazydocker, k9s, gping, watchexec, duf, trippy, sd, actionlint) ([7874ac8](https://github.com/loonghao/vx/commit/7874ac821a2d5798910a3d33807f35050d3d2b29))
* add 7 high-priority developer tool providers (lazygit, delta, hyperfine, zoxide, atuin, chezmoi, eza) ([b221a45](https://github.com/loonghao/vx/commit/b221a45f46e13c6b45d17adf7de32323f65cb923))
* add 7 new providers (tealdeer, dust, xh, bottom, trivy, zellij, dive) ([5aa0e2d](https://github.com/loonghao/vx/commit/5aa0e2da9af225ca010ad18e1d852f5292d0fde3))
* add age and sops providers, update project analyzer frameworks ([f048f10](https://github.com/loonghao/vx/commit/f048f1000b6471d905b143bb6aae8c9ea832fd93))
* add build cache providers (sccache, ccache, buildcache) ([62dbacb](https://github.com/loonghao/vx/commit/62dbacb7d67aafc5a0cd2208e4543c444992d923))
* add dynamic package_prefixes from provider.star metadata ([2bfddd2](https://github.com/loonghao/vx/commit/2bfddd219b381c10c63099ed8fcabf0dfb4ad66b))
* add FilterLevel enum (Light/Normal/Aggressive) for compact output ([#804](https://github.com/loonghao/vx/issues/804)) ([876731f](https://github.com/loonghao/vx/commit/876731f8cb495f403d66e1aeeb9a3ab4c3ea94bb))
* add hadolint (Dockerfile linter) provider ([c9036c6](https://github.com/loonghao/vx/commit/c9036c6b4d521c7de00fa7113f7b937f889a1b9f))
* add helix and yazi providers ([acf70c3](https://github.com/loonghao/vx/commit/acf70c3940812116cf5fb85ac65e389cde028262))
* add llvm/conan/xmake/wix providers and enhance msvc/msbuild for C++/C# build automation ([cfc336a](https://github.com/loonghao/vx/commit/cfc336af1f1b64cb404353253ba5d48f1dad3b91))
* add new oneshot runner ecosystems and update CLI help/docs ([eb78070](https://github.com/loonghao/vx/commit/eb78070dd4e5f1f7a188f9a032bd46e3f1a0bce1))
* add Nx and Turborepo cache providers, fix CI sccache issue ([fdb59a3](https://github.com/loonghao/vx/commit/fdb59a3dd83dacbb7edb57259363e271da867b4e))
* add self-update check with non-blocking notification ([4599d8c](https://github.com/loonghao/vx/commit/4599d8c22c7c853f7775f71127249f8129bd2fb8))
* add starlark provider support with bash, curl, meson, openssl, pre-commit, release-please, rez, systemctl, xcodebuild providers ([4d0ff41](https://github.com/loonghao/vx/commit/4d0ff418b0efb9aaccd9d2740d8e3436f0c4bc35))
* add vcpkg provider for C++ dependency management ([fc6f317](https://github.com/loonghao/vx/commit/fc6f31739bee912800ec7e73ebb2c336dc786b9b))
* add vx skill for ClawHub publication ([#638](https://github.com/loonghao/vx/issues/638)) ([0430588](https://github.com/loonghao/vx/commit/0430588a71d78ff7901bf4edb5ac31dbae9301f7))
* add vx-output-filter crate for compact subprocess output ([#802](https://github.com/loonghao/vx/issues/802)) ([ba69f04](https://github.com/loonghao/vx/commit/ba69f0402b90b318a90510ec4459d99337aaa5c3))
* add well-known Python version fallback for python-build-standalone ([35f85fd](https://github.com/loonghao/vx/commit/35f85fddc8c9fcf3957c5c4847c6e6a80a26b608))
* add witr provider (137 providers total) ([e74d708](https://github.com/loonghao/vx/commit/e74d70837ff3452ae156f463735dff779d267ad9))
* add worktrunk provider (132 tools total) ([#839](https://github.com/loonghao/vx/issues/839)) ([79c179d](https://github.com/loonghao/vx/commit/79c179d1a20ceed3b58335763ea74d61eb7f8c79))
* **ai:** implement RFC 0035 AI integration optimization ([7ab320c](https://github.com/loonghao/vx/commit/7ab320c0898678806829dcdc81673abab0453ce7))
* **auto-improve:** squash merge auto-improve branch ([2797ede](https://github.com/loonghao/vx/commit/2797ede7ae83210a15606321ceab7e8775389b8c))
* bridge global install commands to vx package shim workflow ([0d7ecf2](https://github.com/loonghao/vx/commit/0d7ecf289dbd637c708bfc7c2271cd931aad5571))
* **build:** add rust-lld linker for faster builds (RFC 0032 Phase 1) ([60d7a17](https://github.com/loonghao/vx/commit/60d7a171600a9b22bc455c63fcf7e7b87e79b198))
* **build:** add vx-runtime-core and vx-runtime-archive to workspace dependencies ([c0e13bd](https://github.com/loonghao/vx/commit/c0e13bdddb0d38e2d83bd3ef4f6c1b6971ae1844))
* **build:** create vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([5c2a118](https://github.com/loonghao/vx/commit/5c2a118f06c8471994f7012ff8423ba74d9c9589))
* **build:** integrate vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([cb49cb2](https://github.com/loonghao/vx/commit/cb49cb2d2fb0f609dcaddcc8230862cbed5a9788))
* change non-TTY default output from JSON to Toon, disable CDN by default ([cc46de7](https://github.com/loonghao/vx/commit/cc46de76e29412fc4cd9cbc726587ed7ad7c1dba))
* **cli:** add update channel support (stable, beta, dev) ([75b696b](https://github.com/loonghao/vx/commit/75b696b52466487d28e3e2663b72bd572a8277c8))
* **cli:** Agent DX improvements for AI agents ([cc805e0](https://github.com/loonghao/vx/commit/cc805e0e285f1640961e1a9ac0f4802b243c7658))
* **cli:** bridge global install commands to vx package shims ([c753951](https://github.com/loonghao/vx/commit/c7539518170f28b62c5f8ec5dc330e741808e60e))
* **cli:** enable direct global command shims in vx bin dir ([98131e4](https://github.com/loonghao/vx/commit/98131e429a8c9b7be83c7ee12c15975267ff83b2))
* **cli:** overhaul vx add/remove with format-preserving edits ([c391912](https://github.com/loonghao/vx/commit/c39191214ea7b8782347ae52b1dd7726f5d2c667))
* **ecosystem_aliases:** route ecosystem:package to dedicated provider binary ([e7ccfb4](https://github.com/loonghao/vx/commit/e7ccfb438fda1eff06f5c55c00642389a57dbbad))
* **hooks:** upgrade cargo-hakari pre-commit hook to auto-fix mode ([59d1c58](https://github.com/loonghao/vx/commit/59d1c580ee04103dc4bcbbc8910f98f85f2802aa))
* land provider and CLI improvements ([f173ca7](https://github.com/loonghao/vx/commit/f173ca7859a08d3b019d45046fab6064a21e870b))
* **list:** sort tool list alphabetically (a-z) ([46147f9](https://github.com/loonghao/vx/commit/46147f93f0866a758972524d648b0080dc2e769f))
* propagate explicit version from bundled runtime to parent dependency ([#766](https://github.com/loonghao/vx/issues/766)) ([b48abe6](https://github.com/loonghao/vx/commit/b48abe6aaceec92455830c2476770a4657fecd65))
* **providers:** add cargo-audit provider ([dc2734a](https://github.com/loonghao/vx/commit/dc2734a1157ac168b3c12115811c1b3459c4308a))
* **providers:** add cargo-nextest and cargo-deny providers ([a484a8b](https://github.com/loonghao/vx/commit/a484a8b99c7266924bdf5511d7eab24ce542b3ee))
* **providers:** add conda provider with micromamba, conda and mamba ([94f1aec](https://github.com/loonghao/vx/commit/94f1aec09ccc9a363a883d830fe29816f43d6a0f))
* **providers:** add conda provider with micromamba, conda and mamba ([be63719](https://github.com/loonghao/vx/commit/be63719ff37236be313e222e27eff6b8228b38b2)), closes [#389](https://github.com/loonghao/vx/issues/389)
* **providers:** add grpcurl provider + update provider count to 114 ([906d97e](https://github.com/loonghao/vx/commit/906d97ea0c45496f2bb43d74426cf9b879403d11))
* **providers:** add kind and k3d providers ([ea33834](https://github.com/loonghao/vx/commit/ea338348cd3e53f350a1cc0f78fc5f708c5090f4))
* **providers:** add tokei provider + triage stale issues ([f1077a1](https://github.com/loonghao/vx/commit/f1077a15b5224bf289af15f67f0ee710063cb29b))
* **resolver:** implement version priority with vx.lock support ([37cda5e](https://github.com/loonghao/vx/commit/37cda5e5d8dac4635003d6c0db3b047323f51c86))
* **rfc-0037:** implement ProviderHandle unified facade for CLI commands ([8864cd1](https://github.com/loonghao/vx/commit/8864cd16fbf3bced1770166d84df5515311c415c))
* **rfc-0040:** implement version_info() for toolchain version indirection ([8771443](https://github.com/loonghao/vx/commit/8771443299c5dbbf6c2160ea48c2f7aa5c9af4c1))
* **routing:** prefer dedicated provider over cargo install for ecosystem:package ([1cefc75](https://github.com/loonghao/vx/commit/1cefc75f8e8a0ef4a8b845a2aff2994c33eb0ab8))
* **starlark:** add github.star stdlib + jj provider.star migration ([2666e9c](https://github.com/loonghao/vx/commit/2666e9c35d48962caa4943614fe422d7e7a886b3))
* **starlark:** complete provider.star migration and fix stdlib ctx access ([eb9c882](https://github.com/loonghao/vx/commit/eb9c8821c16458edfcbe61d2650485abf798cef9))
* **tests:** add comprehensive Python provider e2e tests ([861473d](https://github.com/loonghao/vx/commit/861473d25b889ad55aab0e38018c9abfd389fd23))
* use RuntimeContext for install_options instead of env vars (Phase 1) ([96c98a2](https://github.com/loonghao/vx/commit/96c98a2958412defb173f393e0143e206ba96bec))
* **vx-starlark:** implement Phase 2 Starlark execution engine ([46ace14](https://github.com/loonghao/vx/commit/46ace140ae17d909b12ace6b1f1df51a180cf2dd))
* **vx-starlark:** Phase 2 - integrate starlark-rust execution engine ([61b2fd3](https://github.com/loonghao/vx/commit/61b2fd3c167aa851c84c12ffe3ce48efa179591f))
* wire provider dynamic deps and fix install routing ([d60c8b9](https://github.com/loonghao/vx/commit/d60c8b97182a2f3445703a10b52f12ad472f8cfc))


### Bug Fixes

* **7zip:** fix executable name and system_paths to point to binary file ([845419a](https://github.com/loonghao/vx/commit/845419a5d5884bbe00fbb3c68bc880e2289a56e3))
* Add 'if: !startsWith(github.event.head_commit.message, chore: release)' guards to skip these workflows when the push is a release commit. ([d32009f](https://github.com/loonghao/vx/commit/d32009f24555fadde638248c3a17ff0ebb5db644))
* add a ound flag so END block only prints when the match block did not. Also add head -1 safety to ensure only one line is captured. ([15ae81f](https://github.com/loonghao/vx/commit/15ae81f944b6e77f370cc00336bf2a4a1e39fd40))
* add fzf to manifest registry and use source-built vx in docker CI ([d35c0a5](https://github.com/loonghao/vx/commit/d35c0a5d24e547a961dc2d36c30e4255b1f69588))
* add is_version_installable and prepare_execution for bundled runtimes ([daf6a66](https://github.com/loonghao/vx/commit/daf6a66046d6a52e014fc0daca8082d46ecceebe))
* add recursive search for bundled executables and remove wrong fallback ([5db6505](https://github.com/loonghao/vx/commit/5db65050952402709cc3c01e6ba533fe35a8a5b7))
* add workspace-hack deps and remove invalid CI cache parameter ([fed823d](https://github.com/loonghao/vx/commit/fed823de6ca6e8eb4105de7820e85160b2fa8b67))
* address provider CI regressions ([b0d6658](https://github.com/loonghao/vx/commit/b0d6658bb96d9bb95b5887d13e6669312df278e4))
* **ai:** fix skills format and install all 5 skills on setup ([83bf579](https://github.com/loonghao/vx/commit/83bf57939ac92774106abbe680daa9fe78931c9c))
* align starlark mock signatures with stdlib and fix provider tests ([8a8be08](https://github.com/loonghao/vx/commit/8a8be084061faaf1b01ba487ab82c7352d505454))
* **all:** comprehensive CI fixes and Rust 1.95.0 upgrade ([#843](https://github.com/loonghao/vx/issues/843)) ([97d0890](https://github.com/loonghao/vx/commit/97d0890b83b9d6abf9446904c01e13c6a61f4b09))
* auto-disable Spectre mitigation in MSBuild bridge when libs are missing ([a4e1623](https://github.com/loonghao/vx/commit/a4e16232dba6c32e4b6a0f230bb34156a9bd8007))
* auto-fetch versions when version_date cache miss in download_url ([fa81b5f](https://github.com/loonghao/vx/commit/fa81b5fba84da4632c1068f112ab8880dd44342c))
* avoid msvc repair for unrelated commands ([bb6b292](https://github.com/loonghao/vx/commit/bb6b29289687a620b4aa6a56ce9bf5f549aef6d1))
* **cache:** skip NeedsInstall results in resolution cache; extend TTL to 24h ([0cd8c36](https://github.com/loonghao/vx/commit/0cd8c364445189dc8d38cd8dde4ae334f91978ba))
* **cargo-audit:** remove unused rust_triple import (lint) ([48a1a87](https://github.com/loonghao/vx/commit/48a1a87ea8b90c1634c0b41db33fd3040e0ca38d))
* change internal rustup/toolchain debug logs to trace level ([bdbbe93](https://github.com/loonghao/vx/commit/bdbbe938cdc463e7f6ef15f9f321077625a8305c))
* **ci:** add sccache setup to all CI workflows ([67237a6](https://github.com/loonghao/vx/commit/67237a685b901fd80c8ebbd99e88897a30fa89cc))
* **ci:** add sccache setup to benchmark workflow ([dbaefc8](https://github.com/loonghao/vx/commit/dbaefc8a279c38df310b2da41629dc7ac0d776b3))
* **ci:** ensure Release workflow triggers even when release is created from update-pr job ([7259bc6](https://github.com/loonghao/vx/commit/7259bc6fab2b45aad62fc4c7c583577f6157209f))
* **ci:** exclude vx-msbuild-bridge from cargo-dist & improve skills sync ([2ca24a3](https://github.com/loonghao/vx/commit/2ca24a3962f66295da4022feee6ca13b8aa98698))
* **ci:** fix discovery parser and CI skip list for Linux/macOS failures ([5d4b834](https://github.com/loonghao/vx/commit/5d4b8348bcd83512e011d8ee9aa9e738ad04f088))
* **ci:** handle skipped/cancelled jobs in CI Success gate ([18ebc6c](https://github.com/loonghao/vx/commit/18ebc6c2242ec56e580b71f0b377336b82860af0))
* **ci:** improve sccache path handling on Windows ([7412df4](https://github.com/loonghao/vx/commit/7412df4895b16b9866e83a11c60e92ed1a410623))
* **ci:** include Cargo.lock in workspace-hack commit step ([0e75aab](https://github.com/loonghao/vx/commit/0e75aab10237ad9521727751b567aca9792392a5))
* **ci:** increase Windows timeout to prevent CI failures ([313b008](https://github.com/loonghao/vx/commit/313b008dd616b9b18c108a7e210c2c3013679327))
* **ci:** install sccache in quick-test job ([4e32e1a](https://github.com/loonghao/vx/commit/4e32e1acfcd03cdbc499f413182975903f0d5a33))
* **ci:** prevent duplicate release-please PRs on release merge ([d32009f](https://github.com/loonghao/vx/commit/d32009f24555fadde638248c3a17ff0ebb5db644)), closes [#713](https://github.com/loonghao/vx/issues/713)
* **ci:** remove lld linker on macOS due to compatibility issues ([1f283d7](https://github.com/loonghao/vx/commit/1f283d76140a64b910a94f0196a5b897462934fd))
* **ci:** remove max-versions-to-keep from winget-releaser ([21fa5f7](https://github.com/loonghao/vx/commit/21fa5f768cedcc332238c90fc8db5633abc798c4))
* **ci:** replace curl POST with clawhub CLI in sync-skills workflow ([94ab959](https://github.com/loonghao/vx/commit/94ab959d6b2b1bf53dc41e33b6274428a2fb4938))
* **ci:** replace remaining vx run cargo scripts with direct vx cargo calls ([948d26a](https://github.com/loonghao/vx/commit/948d26a40081e726abd86aa12b56b4f922bc2deb))
* **ci:** resolve required check name conflict blocking PR merges ([8f92a54](https://github.com/loonghao/vx/commit/8f92a54e82ee4772364efb40e82a464d667d1def))
* **ci:** sanitize provider cache keys ([11e46fe](https://github.com/loonghao/vx/commit/11e46fe6fe5f2bce70bac911beea15cc35dde42f))
* **ci:** skip wix and xmake in CI tests ([bccc34a](https://github.com/loonghao/vx/commit/bccc34a1c0c2f150570392699b5906bf7a97866b))
* **ci:** split release-please into two jobs to fix tag creation ([7dd72cf](https://github.com/loonghao/vx/commit/7dd72cfa6760528d8305ed264bd1fb9b9d70a20e))
* **ci:** switch apt mirror to azure.archive.ubuntu.com for cross builds ([344e043](https://github.com/loonghao/vx/commit/344e04310761b34ce16443854c6f06c6c9c9ae91))
* **ci:** use system cargo directly instead of vx cargo ([0b45f66](https://github.com/loonghao/vx/commit/0b45f6665845c2f3356c867dbe004790d15e7dbf))
* **ci:** use vx cargo prefix in justfile recipes for CI compatibility ([5984668](https://github.com/loonghao/vx/commit/5984668e65c61beea025e9471374fac30e442050))
* clean extraction markers before re-installing missing MSVC components ([96cc05a](https://github.com/loonghao/vx/commit/96cc05a830f56677df52af39b4c5969bee30a138))
* **cleanup:** fix compile errors from ecosystem_aliases feature ([d89bc38](https://github.com/loonghao/vx/commit/d89bc38489f7f670d738d199b866027cf7965dff))
* **cli:** add --toon shortcut flag for TOON output format ([0a5cc91](https://github.com/loonghao/vx/commit/0a5cc9118fde4cc06dc6cd8cc428138cc49f0f8b))
* **cli:** fix Clippy warnings and test compilation errors ([20f4cd7](https://github.com/loonghao/vx/commit/20f4cd75d3e13ab0d6eb8edda5183535a8db8178))
* **cli:** fix formatting issues (run cargo fmt) ([0cac2ed](https://github.com/loonghao/vx/commit/0cac2ed6f24661cffab2285e6c1d3df7db939fcf))
* **cli:** fix vx check system_fallback and vx lock for installed tools ([06b47b0](https://github.com/loonghao/vx/commit/06b47b0368a0952d213286a6ddd726b6df56eee4))
* clippy useless_vec warnings in tests ([0028d2b](https://github.com/loonghao/vx/commit/0028d2b9cfcfe79f7e4a0de8625c481f6c84a70e))
* **cli:** remove debug print from lib.rs ([58d35be](https://github.com/loonghao/vx/commit/58d35bec5ddf6e278af464ed718d8e2153b0c8d9))
* **cli:** send update notifications to stderr ([fcf373a](https://github.com/loonghao/vx/commit/fcf373a45451324a5ad42eed83db56e3e859d927))
* collapse nested if statements to satisfy clippy ([0338d30](https://github.com/loonghao/vx/commit/0338d3059bb232ecbe2f333ab2a3833774ab7493))
* **console:** use eprintln for progress output to avoid stdout contamination ([7ebf3e7](https://github.com/loonghao/vx/commit/7ebf3e7dc4db23a13927b9e65d4ee5c93618d2ac))
* correct cmake macOS download URL and jq environment key ([e47d36b](https://github.com/loonghao/vx/commit/e47d36b39a3846a5585aefa16c173975b9e89b46))
* correct download URLs for grpcurl, k3d, kind, and duckdb providers ([108e290](https://github.com/loonghao/vx/commit/108e290654d06867bae9f837301d5a8c96cf09a0))
* correct RFC count (40+ -&gt; 50+) and update Rust version badge (1.93+ -&gt; 1.95.0+) ([#879](https://github.com/loonghao/vx/issues/879)) ([a400f57](https://github.com/loonghao/vx/commit/a400f57db7fcd01763f1d15656b5da9dd1e6f404))
* **deps:** update rust crate anstream to v1 ([b837fcd](https://github.com/loonghao/vx/commit/b837fcdbca1344095e65f1523c65bec4c01d04bd))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 ([90f011f](https://github.com/loonghao/vx/commit/90f011f71a7d2b7bc4c89d80984b76e9d6e02c42))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 - abandoned ([d934ad1](https://github.com/loonghao/vx/commit/d934ad179813669f0c343585019676eb32a82913))
* **deps:** update rust crate starlark_derive to 0.14 ([82c274b](https://github.com/loonghao/vx/commit/82c274b29d5dd4deb80e2318dba7cdf97fea83e7))
* **deps:** update rust crate toon-format to 0.5 ([bb23093](https://github.com/loonghao/vx/commit/bb23093796635b88b8de85cc5db526952664ee9e))
* **dist:** exclude vx-star-metadata from cargo-dist release artifacts ([2c30069](https://github.com/loonghao/vx/commit/2c3006951d57bea85b8391628704437872ea9e1a))
* **docker:** switch apt mirror to azure.archive.ubuntu.com in Dockerfiles and test workflow ([c95f733](https://github.com/loonghao/vx/commit/c95f7335208129fdc988bce102338225d44b0ef9))
* **docs:** escape angle brackets in RFC 0033 headings ([2c76f06](https://github.com/loonghao/vx/commit/2c76f06b3104ea316a01d25f829f90ee0c118f30))
* **docs:** fix broken doctests in vx-console and vx-starlark ([58d88e8](https://github.com/loonghao/vx/commit/58d88e8ac48cd2e9ad73e607b9e5bbc15282f8c9))
* **engine:** ctx.install_dir now points to actual install location ([19237c4](https://github.com/loonghao/vx/commit/19237c415bbb75c195f9300f6ed91e6202c46e2f))
* ensure MSVC Spectre component integrity check for already-installed companion tools ([3da828c](https://github.com/loonghao/vx/commit/3da828cdaf68862b2f413a16f097fdefd6875233))
* ensure rust targets are installed in CI ([9bffec3](https://github.com/loonghao/vx/commit/9bffec3aa24b600c7aa44eab403e53dda45713d9))
* exclude vx-star-metadata from cargo-hakari workspace-hack ([2a8bb5f](https://github.com/loonghao/vx/commit/2a8bb5ff6ffb995046670095da0b8b0f3d332733))
* **eza:** add platform_constraint to skip macOS in CI tests ([65c9571](https://github.com/loonghao/vx/commit/65c9571af9fea6a48ac3599e501c357279cf2e84))
* ffmpeg use Gyan.dev mirror, witr only override download_url ([10a7a50](https://github.com/loonghao/vx/commit/10a7a5007e506a42f7a46c007c9332ae8204c2ad))
* **ffmpeg:** use system_install only (remove unreliable GitHub downloads) ([1f8780a](https://github.com/loonghao/vx/commit/1f8780adbe1df7c4cafaf5a6213cf1a2663b7d58))
* **ffmpeg:** use vx-org/mirrors with BtbN static builds (win64+linux64+linuxarm64) ([77d741c](https://github.com/loonghao/vx/commit/77d741c7f7f8a69358a70f9fa13d614d8bb9c81d))
* filter vault releases by platform artifacts ([1a01df2](https://github.com/loonghao/vx/commit/1a01df2490a3d2ea68fddf05f4afd28d718de11d))
* fix PSReadLine cursor positioning issue in PowerShell prompt ([e802fee](https://github.com/loonghao/vx/commit/e802fee8e36ffcdbd774179c03153b8464dbea44))
* fix system_install providers and starlark test assertions (round 5) ([772edbd](https://github.com/loonghao/vx/commit/772edbd39371c01b279c46b8d99e6fbf2db6ab45))
* fix workspace-hack hakari section markers and regenerate dependencies ([f41c6c6](https://github.com/loonghao/vx/commit/f41c6c694d102f6a39492987d3de16190dc9093a))
* flatten InstallLayout JSON so manifest_runtime can read strip_prefix ([ab4fea7](https://github.com/loonghao/vx/commit/ab4fea7bf54048a00035de3a4630c79c87d152dc))
* **gcloud:** update starlark test to use __type field ([2a35711](https://github.com/loonghao/vx/commit/2a357115eea69195be4285e73e00bdbf9747f684))
* git uses MinGit ZIP on Windows, rust toolchain defaults to stable ([da6405d](https://github.com/loonghao/vx/commit/da6405d17c38cc39a29a46136ef2fb734d4cc95a))
* git Windows exe path, rust bundled store mismatch, lock multi-platform URLs + perf optimizations ([#787](https://github.com/loonghao/vx/issues/787)) ([f208ef5](https://github.com/loonghao/vx/commit/f208ef535b434e3eab4d0e047ac7a61e164806d1))
* gracefully resolve numeric version hints for pure opaque providers ([4891b84](https://github.com/loonghao/vx/commit/4891b84942b85972e53e129937545b4e311ff633))
* hadolint asset name separator and uvx bundled_with support ([65a2fad](https://github.com/loonghao/vx/commit/65a2fad4c80b7feb0ac7572c7ae6c1d072824059))
* hadolint provider executable layout and static registry ([70ff5d6](https://github.com/loonghao/vx/commit/70ff5d605d1590bbe4acc40684058409e5170410))
* handle JSON output in where command e2e tests ([8ea99c4](https://github.com/loonghao/vx/commit/8ea99c4aa03a98ff8cc066ab47a16e9a6c4c8696))
* handle rust toolchain versions in path selection ([585353e](https://github.com/loonghao/vx/commit/585353ea73647b1a4c05d1c6fe02cc5e729fc25c))
* handle VX_VERSION=latest in install scripts ([5351963](https://github.com/loonghao/vx/commit/5351963159e0abee0a93498d694778dfc8ce9e7e))
* import env_prepend from env.star instead of provider.star ([f63a814](https://github.com/loonghao/vx/commit/f63a814c395205c95fed8f87149e42df11991dab))
* improve installer fallback and mirror release support ([a56b239](https://github.com/loonghao/vx/commit/a56b239c8b00e1df09ed8c0568c85ac022a7685e))
* inject companion tools environment variables for vx.toml co-configured tools ([7fe198d](https://github.com/loonghao/vx/commit/7fe198d4b8e7e07008caf631b0b77df8abfaa422)), closes [#582](https://github.com/loonghao/vx/issues/582)
* inject parent runtime env for bundled runtimes (npm/node PATH issue) ([0b3de77](https://github.com/loonghao/vx/commit/0b3de7724f670c8e37f4f09ea0057246772ad89e))
* inject parent runtime PATH for bundled runtimes via spec env_config ([06ca8aa](https://github.com/loonghao/vx/commit/06ca8aa8a75d15a6e78689ff3daac36559bceec1))
* **installer:** apply binary rename fix to RealInstaller.download_with_layout ([925d6d5](https://github.com/loonghao/vx/commit/925d6d54f963dcd1ee4bb96209eef984a356827c))
* **installer:** fix PortableGit .7z.exe not recognized as archive + stop version fallback on layout errors ([d62adba](https://github.com/loonghao/vx/commit/d62adba7b119807c5f35d70a93403f1b8ece2272))
* **install:** prevent awk double-output in resolve_latest_version ([15ae81f](https://github.com/loonghao/vx/commit/15ae81f944b6e77f370cc00336bf2a4a1e39fd40))
* **install:** skip releases without binary assets in version resolution ([ff7438a](https://github.com/loonghao/vx/commit/ff7438a2491346cbfe1c0bf941b1f206615957a4))
* **just:** correct version_pattern to match 'just X.Y' output ([942e6a3](https://github.com/loonghao/vx/commit/942e6a32a53fd9e8dcb615053fa2a5163a9049bc))
* **justfile:** fix test-pkgs recipe to not duplicate -p flag ([cbfc402](https://github.com/loonghao/vx/commit/cbfc402a896bbb0e40ee6352fc3383a32d4bed24))
* **lint:** resolve provider.star lint issues ([aaa95a3](https://github.com/loonghao/vx/commit/aaa95a31ad00e07c03059afc7ef3c8a628d8a88e))
* lower rust-version to 1.93.0 for CI compatibility ([5c71b09](https://github.com/loonghao/vx/commit/5c71b094f5eb1d3570fa00230b4ddbf5eb495d88))
* **macos:** make sevenz-rust optional to fix macOS build ([5042c58](https://github.com/loonghao/vx/commit/5042c582152491d3c8ac2844de7ed9edc56db988))
* make E2E version list tests resilient to transient network errors ([99d8eba](https://github.com/loonghao/vx/commit/99d8eba57182fb1cd6273599c260cc9b50f103df))
* make env-dependent tests serial to prevent race conditions ([81e82e0](https://github.com/loonghao/vx/commit/81e82e079c626f07d2976f978afb657cfcc5f2a6))
* **manifest-runtime:** override resolve_version to return 'system' for system tools ([0e0c101](https://github.com/loonghao/vx/commit/0e0c101b9598502f76e685ab5a7b5eefb065957c))
* **mise:** avoid strip_prefix on Windows to prevent Access Denied errors ([a955566](https://github.com/loonghao/vx/commit/a95556696b2be579f3750e5c383d90e3142c79d0))
* **mise:** update unit tests to match new install_layout implementation ([4d2946f](https://github.com/loonghao/vx/commit/4d2946fcb121edee0518079390d1a2026aae3745))
* **mise:** use strip_prefix='mise/bin' on Windows to avoid shim detection error ([5c06c76](https://github.com/loonghao/vx/commit/5c06c765dcb6634d3ed8adc7618e81ff49c34d92))
* **paths:** detect unified runtime store versions ([574c8ff](https://github.com/loonghao/vx/commit/574c8ff581b5d9e7f2b844d1cb1c35e175281a59))
* platform-aware executable fallback in ResolvedLayout ([3a79a16](https://github.com/loonghao/vx/commit/3a79a161af6a0dfb1d4b90ade1a9be261d22a8c7))
* preserve Rust MSRV in vx.toml and enable passthrough for Rust ecosystem ([1ded9c9](https://github.com/loonghao/vx/commit/1ded9c98c0e740e17f50df4b239abbec2a11c040))
* prevent bundled runtime executable misresolution (npm-&gt;node) ([12d5d50](https://github.com/loonghao/vx/commit/12d5d50cee8ab95976b84d6e01ef047d58c2a0f5))
* prevent repeated MSVC component re-installation when Spectre libs unavailable ([9c93a0c](https://github.com/loonghao/vx/commit/9c93a0c2d8b6d25cee51fe588974147bdef67c0a))
* propagate locked version to bundled runtime dependencies ([1fa037f](https://github.com/loonghao/vx/commit/1fa037faa473e077cec32eb3247ac8b4e72fd5e6))
* **provider:** correct grpcurl version check ([7c917ef](https://github.com/loonghao/vx/commit/7c917ef4e660127a5c8fa343fc421dba8eb38093))
* **providers:** add fetch_versions_with_tag_prefix to layout mock + fix cargo-deny Windows ([eea4d7e](https://github.com/loonghao/vx/commit/eea4d7e813170e8a9e6e05ba4f36e3fbdcd01282))
* **providers:** correct install_layout strip_prefix and download_url ([3cad3c9](https://github.com/loonghao/vx/commit/3cad3c97c4578b5ca228f3765b60ea7f823dedbd))
* **providers:** correct mirror tag version fetching ([03a28a2](https://github.com/loonghao/vx/commit/03a28a2c36623fe1f331837dea37a548c903ec4a))
* **providers:** fix 5 more provider bugs (round 2) + add tar.bz2 support ([88874cd](https://github.com/loonghao/vx/commit/88874cdf2884f62cfc8dce9f3a56623842c9559a))
* **providers:** fix 5 provider bugs from auto-improve branch ([c6938cf](https://github.com/loonghao/vx/commit/c6938cf760762d1c874280dc8f8f66086fdfa1c3))
* **providers:** fix binary rename, grpcurl macOS, duckdb macOS, nerdctl platform ([4a07643](https://github.com/loonghao/vx/commit/4a076431b43b45e00e2e1e38862008f8700fce66))
* **providers:** fix cargo-nextest macOS triple and cargo-audit test assertions ([86186d9](https://github.com/loonghao/vx/commit/86186d9cfd1c5c53af16b64ca352ef6a597280aa))
* **providers:** fix download URL bugs in git, xmake, and ollama ([#777](https://github.com/loonghao/vx/issues/777)) ([0287fff](https://github.com/loonghao/vx/commit/0287fff7a6e7f86b8a8cd07aa06be004158678fc))
* **providers:** fix download URLs for cargo-audit, cargo-nextest, and deno ([66555ed](https://github.com/loonghao/vx/commit/66555ed3e2a642a9965e209f3b597e033435d8bc))
* **providers:** fix dust and eza macOS download URL 404 ([5eb5b20](https://github.com/loonghao/vx/commit/5eb5b201ae34359cf7a8a29e0cc4b555f03247b2))
* **providers:** fix dust version pattern and tealdeer binary rename ([3f3cd31](https://github.com/loonghao/vx/commit/3f3cd31ec6e83b75d6858a4a1a446539e5672ecc))
* **providers:** fix gcloud get_execute_path and terraform fetch_versions ([5c2505d](https://github.com/loonghao/vx/commit/5c2505da487877fbe0331f7704bd83403d061853))
* **providers:** resolve CI download URL failures ([7391695](https://github.com/loonghao/vx/commit/7391695b724fae5c971353e1cdc29e62918b89fa))
* **providers:** resolve CI issues for new provider batch ([2b229ab](https://github.com/loonghao/vx/commit/2b229ab21a012fff3d200997ab892000431eff50))
* **providers:** resolve tealdeer and mise install layouts ([ed6b6b6](https://github.com/loonghao/vx/commit/ed6b6b630910eec5374c5eff820ab5d88f1f8a56))
* **providers:** use vx-org/mirrors for ffmpeg and witr downloads ([ac4dc6a](https://github.com/loonghao/vx/commit/ac4dc6a82dff09cc6cb21e236d50f71d4d9ab4ce))
* **provider:** use gnu rustup triples on linux ([a1984e2](https://github.com/loonghao/vx/commit/a1984e21a8de4fcc4eca1ef1dd573467915399ac))
* Python install fails due to version_date cache key mismatch ([9ab1ea4](https://github.com/loonghao/vx/commit/9ab1ea4b25e83d87daf5823b3e871a5fa4a95ff2))
* reinstall runtime when executable missing from cached install ([9f894aa](https://github.com/loonghao/vx/commit/9f894aaca1969fff6870bc2098e037be19faf693))
* relax MSVC prepare_environment validation to only check cl.exe existence ([9ca374d](https://github.com/loonghao/vx/commit/9ca374d119a05a14bdff9cc199584dee7bb6c866))
* **release:** disable sccache rustc-wrapper in release workflow ([b202568](https://github.com/loonghao/vx/commit/b202568208b421a9eaa1e7f76a44d9900b58181a))
* remove BOM from all provider.star files and improve star syntax checker ([dbafa40](https://github.com/loonghao/vx/commit/dbafa40eee01a867e35011bc79ed3d433e16efc0))
* remove platform subdir from install path, fix providers ([c80f726](https://github.com/loonghao/vx/commit/c80f726c6b9a5b54ea573bbbde0d2ec83528036a))
* remove unstable as_str() usage in environment.rs ([18d34e2](https://github.com/loonghao/vx/commit/18d34e26c2ee48072d06af8efc4ffcc20a90dc80))
* remove unused loads and fix lint issues in provider.star files ([dc83b6f](https://github.com/loonghao/vx/commit/dc83b6f138d76f22d924b8ce87fa54a06ef92208))
* remove unused variables in witr/provider.star ([ef0d871](https://github.com/loonghao/vx/commit/ef0d87125f5d2a6c1371ae56cd6a95dfa45208da))
* remove windows-sys 0.59 and merge features into 0.61 in workspace-hack ([92594e4](https://github.com/loonghao/vx/commit/92594e43c5de98faefc899e3eb2a39d30e7ee64e))
* repair provider test resolution and platform gating ([8dc3aa5](https://github.com/loonghao/vx/commit/8dc3aa55c40dcbcdc2d1426cd0f545603f0b7587))
* replace all ctx dict access with struct attribute access in provider star files ([f1e93f1](https://github.com/loonghao/vx/commit/f1e93f150e2bb486abaccdd6942f0a4533908d56))
* replace all ctx.http.get_json with fetch_json_versions descriptors in provider.star files ([7bcdd33](https://github.com/loonghao/vx/commit/7bcdd333b81a1e2cb5bdbb5eb5816984488f38c0))
* resolve .cmd executables for bundled runtimes on Windows ([0442c91](https://github.com/loonghao/vx/commit/0442c91ff6f7704db4f65a3ab568ba9a256586a6))
* resolve CI errors ([de08ab2](https://github.com/loonghao/vx/commit/de08ab29ac0a7e9a0c04350722ad464ec9b997d0))
* resolve CI errors ([90a1a7c](https://github.com/loonghao/vx/commit/90a1a7cac4ede8305ff154f2f5cb65c236e47ff5))
* resolve CI failures for imagemagick, ffmpeg, rez, bash, make, nasm ([d900aae](https://github.com/loonghao/vx/commit/d900aaed962ce47705dd419d620815d725135834))
* resolve CI failures for lefthook, grpcurl, and kustomize providers ([3bcc0ec](https://github.com/loonghao/vx/commit/3bcc0ecd4c1eaf0cabb76bd83b0eb95801b6033f))
* resolve CI failures for yq, wix, xmake, vcpkg providers ([afee387](https://github.com/loonghao/vx/commit/afee38790239bd65ec19e9ebdc1c4781974ffc5b))
* resolve clippy warnings and test assertion ([484f35c](https://github.com/loonghao/vx/commit/484f35c19d946bf1931bd64b6999b3921980f6df))
* resolve compiler errors in test files ([fe12f86](https://github.com/loonghao/vx/commit/fe12f8697ff085080a00dcc847f434e7d73263ee))
* resolve Linux CI failures for ffplay/ffprobe/gofmt/lld/xmake/yq ([f379ab2](https://github.com/loonghao/vx/commit/f379ab25b746ba94a17051cee6abb7ba9a2c61f0))
* resolve macOS CI failures for ffmpeg and imagemagick ([10949b0](https://github.com/loonghao/vx/commit/10949b0590b8716c16c3ae544a0992c2ebdb2e9e))
* resolve merge conflict in release manifest ([deab703](https://github.com/loonghao/vx/commit/deab7036e02c58b42e68a99b0d62f5db013fc70f))
* resolve Python PYTHONHOME mismatch ([#696](https://github.com/loonghao/vx/issues/696)), improve version pagination, unify skills ([fbadc74](https://github.com/loonghao/vx/commit/fbadc745050d6ff951ea4822ad65eda807f344ca))
* resolve sha2 LowerHex compile errors and upgrade GitHub Actions ([d148868](https://github.com/loonghao/vx/commit/d148868e133136b9d33627d0a793dadffa9ee5af))
* **resolver:** resolve bundled runtime fallback executable ([099ff92](https://github.com/loonghao/vx/commit/099ff92fc7d1f87f3b9d7857d6f7d4bd9811b25c))
* **resolver:** stop re-installing system-managed runtimes on every vx invocation ([83d239b](https://github.com/loonghao/vx/commit/83d239b29af6304e7dfee21ae6b48a8575200220))
* **runtime:** check vx store first in ManifestDrivenRuntime.is_installed() and installed_versions() ([fe803e3](https://github.com/loonghao/vx/commit/fe803e370376fef347c1acd10608e15819152bf7))
* **runtime:** preserve bundled command prefixes ([821e779](https://github.com/loonghao/vx/commit/821e779c2f1f96d9dfa6a7c5fa43a393aaf90d85))
* **runtime:** satisfy clippy collapsible-if lint ([51dc8f4](https://github.com/loonghao/vx/commit/51dc8f4d2eb6f79b011bd1d18f9f486b35701973))
* Rust ecosystem passthrough for rustc versions in resolve_version ([a18548a](https://github.com/loonghao/vx/commit/a18548abbf479a075218f77c5cb7b4866fef1742))
* **rust:** stop re-installing rust on every vx cargo invocation ([988858c](https://github.com/loonghao/vx/commit/988858c665d1faebe267bfba8d2b1a76c26e468a))
* skip broken micromamba windows release ([e4457a9](https://github.com/loonghao/vx/commit/e4457a9a877f9a7969eba38f2e3a90fcf040e8d4))
* stabilize test suite and version constraint parsing ([a3ae77a](https://github.com/loonghao/vx/commit/a3ae77af823e28a49f265afd063000f0647be2a2))
* **starlark:** lower provider loading log level from info to debug ([96aed89](https://github.com/loonghao/vx/commit/96aed893d275aac78d693f1a110efc8d6d5d971e))
* **starlark:** register all 14 stdlib modules in loader ([583b16b](https://github.com/loonghao/vx/commit/583b16b892104bc9059cc521eeb8796baa6dc873))
* switch macOS FFmpeg download source from evermeet.cx to osxexperts.net ([1fbd926](https://github.com/loonghao/vx/commit/1fbd926e8dfbd6316c742d1778def3ccdcec0e2a))
* temp_dir unbound variable in install.sh and uv strip_prefix ([bf9cef4](https://github.com/loonghao/vx/commit/bf9cef4410700f1134b5edd4e34c70c9eb55097a))
* **test:** add missing package_alias field in manifest_registry_tests ([df2bbec](https://github.com/loonghao/vx/commit/df2bbeca7c89d36ecb01afe9d4618124ae409495))
* **test:** add missing package_alias field in ProviderMeta test ([10b7d1e](https://github.com/loonghao/vx/commit/10b7d1e7b58c7a66fc7959e904ec0967e4153f72))
* **tests:** add missing OutputFormat argument to handle_list calls ([0756270](https://github.com/loonghao/vx/commit/07562706ae789b2483a2f8a4023f34f3b6e3e824))
* **tests:** fix 14 failing tests across multiple crates ([5069772](https://github.com/loonghao/vx/commit/5069772a3a7a9aa9edca8a8581acdb00d696b7e9))
* **tests:** fix cargo-deny Windows URL test and add missing provider tests ([aaae704](https://github.com/loonghao/vx/commit/aaae704b513bcf018ed115e1ba1443a9a9a8da31))
* **tests:** fix output_tests and info_tests failures in non-TTY CI ([c3ab0bd](https://github.com/loonghao/vx/commit/c3ab0bd30dfaa5737eb3846a203ee4bcdb47dfb1))
* **test:** skip package_alias providers in CI tests ([46b9676](https://github.com/loonghao/vx/commit/46b967622af540ba62f13b0f1f7189bebf167181))
* **tests:** relax assertion in test_vx_toml_python_setup_dry_run ([baa00df](https://github.com/loonghao/vx/commit/baa00df8f04a5140039911958f42513a7a369ca1))
* **tests:** resolve latest unit test failures ([5c9684f](https://github.com/loonghao/vx/commit/5c9684fbc7c4102a596ea0b69cc5f25044e93586))
* **tests:** rewrite all provider runtime_tests to use create_provider() API ([a0edcfc](https://github.com/loonghao/vx/commit/a0edcfcf181f65372b47c7403839f480a85469db))
* **ui:** show Installing feedback during auto-install to avoid perceived hang ([45fbd39](https://github.com/loonghao/vx/commit/45fbd394f390c6e23f9b39209cca22a914eec4fe))
* unblock remaining CI regressions ([c2c7b91](https://github.com/loonghao/vx/commit/c2c7b91b245017939755249b683288e9d0c41b51))
* use bin/bash.exe for git-bash instead of git-bash.exe --attach ([7e72af2](https://github.com/loonghao/vx/commit/7e72af28c8cda9d860729a1adc5e365271aca6ee))
* use child version for bundled proxy runtime installation ([02af11c](https://github.com/loonghao/vx/commit/02af11cc35f29d1b1021f07577311d7ff2ff5218))
* use struct attribute access ctx.platform.os instead of dict access ctx[platform][os] in stdlib star files ([ea14d6c](https://github.com/loonghao/vx/commit/ea14d6c3640c35d07f8ddb6f1d20b834726881a4))
* use system_install for ffmpeg Linux, fix witr install_layout ([293a4f5](https://github.com/loonghao/vx/commit/293a4f5b621312be21fd5baaf6c1b3020c11b0eb))
* **uv:** route uvx through uv tool run ([7b65a63](https://github.com/loonghao/vx/commit/7b65a63ed28d15b158b7c500a56dcf567e97dff7))
* **versions:** case-insensitive Ecosystem deserialization for vx.lock compatibility ([bc33606](https://github.com/loonghao/vx/commit/bc33606fa8db60af5fac771b830f27e9a8642f97))
* **vx-config:** fix sha2 GenericArray LowerHex compile error ([0c60d1b](https://github.com/loonghao/vx/commit/0c60d1b3f88f6378aa7d80eeb49f5f63d2c2abb4))
* **vx-provider-jj:** strip v prefix from version tags to prevent double-v in download URL ([e0b13eb](https://github.com/loonghao/vx/commit/e0b13eb8f7c9995682eb3024d798c2fed8ef2288))
* **watchexec:** remove unused load imports to pass provider static lint ([e4da30a](https://github.com/loonghao/vx/commit/e4da30ab193eba7dc1f809554f10f1b09a2324e6))
* **watchexec:** use .zip on Windows, .tar.xz on Linux/macOS ([6c8e978](https://github.com/loonghao/vx/commit/6c8e978fd60c424bb91d3da4b8898ab7a65d0d01))
* **where:** use executable_name() instead of runtime name for exe lookup ([610310f](https://github.com/loonghao/vx/commit/610310fbba23d21940214cc0c820730fcc57882c))
* **windows:** resolve OS error 193 when executing bundled runtimes (npm/npx) ([f7329c9](https://github.com/loonghao/vx/commit/f7329c9d4f4f8bea2b403a392359146b40e5fcc7))
* wire Starlark post_extract hooks into ManifestDrivenRuntime and fix bundled runtime detection ([0cab93c](https://github.com/loonghao/vx/commit/0cab93c60355e1ea12fc9b332a41e1fbf18d46d8))
* **witr:** correct __type__ key in install_layout (double underscores) ([b287fe5](https://github.com/loonghao/vx/commit/b287fe520f6c3df04a3babd887172110d50591fe))
* **witr:** correct version pattern and binary path in provider.star ([b6683e1](https://github.com/loonghao/vx/commit/b6683e1576ee1f943f7e6f78232588e5b4545f21))
* **witr:** override install_layout with correct type ([6bf7bbd](https://github.com/loonghao/vx/commit/6bf7bbd63a5e12730e79351f24282053e85c9e7c))
* **witr:** rewrite provider without template to handle direct binaries correctly ([dcd08cc](https://github.com/loonghao/vx/commit/dcd08ccdd5051e4de05cc743ef8ceb33a1362be7))
* **witr:** use 'binary' type for direct binaries (Linux/macOS) ([b37b591](https://github.com/loonghao/vx/commit/b37b591952902505e3220cfd087b9f0e79211860))


### Performance Improvements

* add cargo build optimization agent rule ([f042114](https://github.com/loonghao/vx/commit/f042114000e11bd3fc94fc3c585aac1baf8511a8))
* implement cargo-hakari workspace-hack + runtime/config refactoring ([aa28ce3](https://github.com/loonghao/vx/commit/aa28ce330be7605cc107a3159654327ddc5c6415))
* optimize test and build configuration ([7228164](https://github.com/loonghao/vx/commit/7228164be85faaac1bc9585ab9e0a5b1d7c73544))
* optimize workspace compilation settings ([af082f7](https://github.com/loonghao/vx/commit/af082f777eaa521534181a924060bcd35e67c48f))


### Code Refactoring

* **build:** remove legacy provider.toml support, simplify build.rs and registry.rs ([b0dd935](https://github.com/loonghao/vx/commit/b0dd935619a4454c65aec137cef79a202d9bc44b))
* **cli:** update commands and test utilities for runtime refactoring ([d2640d8](https://github.com/loonghao/vx/commit/d2640d8f22436f37417efcce1373314a2ca3ae31))
* **env,version-fetcher:** eliminate platform/version utils duplication ([1a1e4d4](https://github.com/loonghao/vx/commit/1a1e4d4b73b9cdccaabfaef62cdb8ef0aebebbc9))
* extract vx-star-metadata crate and eliminate Box::leak usage ([153d77a](https://github.com/loonghao/vx/commit/153d77acb7bd95d865548473a1a17f57c2775220))
* improve code quality - replace unwrap() and eprintln! with proper error handling ([a2c4c0b](https://github.com/loonghao/vx/commit/a2c4c0b05ec9abebf0cead84e61b70f058f80b62))
* improve code safety and remove dead code ([95e405f](https://github.com/loonghao/vx/commit/95e405fd12fe15bf59647f5f6ea6b4db40dd97d2))
* improve code safety by eliminating unsafe unwrap calls ([d8778c7](https://github.com/loonghao/vx/commit/d8778c72834f4e9deb619400b69318b0209209f2))
* merge vx-core into vx-runtime-core ([d91989e](https://github.com/loonghao/vx/commit/d91989e3d1bd667106e43c0b0f0dfe41885bf862))
* migrate providers, add bridge system, fix Windows env injection ([b3decdb](https://github.com/loonghao/vx/commit/b3decdbc93649e5214dc434e341e073a6c445e13))
* migrate providers, add bridge system, fix Windows env injection ([a2b9c0c](https://github.com/loonghao/vx/commit/a2b9c0c732f5910eb9aea148eb2b660ab7d7bf8b))
* optimize provider.star files using stdlib templates ([71126bc](https://github.com/loonghao/vx/commit/71126bc5322565185b2557c18eac0800251cc894))
* **provider:** conda use provider.star only (remove Rust code) ([4ac74a6](https://github.com/loonghao/vx/commit/4ac74a66261ce771619e7120c449e522cdaae6a5))
* **providers:** replace all hand-written permissions dicts with stdlib helpers ([a3d66b1](https://github.com/loonghao/vx/commit/a3d66b1e441a45a6c955120b500096ca8f4a14b1))
* **providers:** simplify providers to use standard templates ([0085259](https://github.com/loonghao/vx/commit/008525996aebffc16cdd56422efeece7f03ee1aa))
* replace bare .unwrap() with descriptive .expect() in production code ([eb151eb](https://github.com/loonghao/vx/commit/eb151eb3ee8acce69abd1b0fe085fc01545811ca))
* **resolver:** integrate ResolutionCache into execution pipeline ([4b9c0ca](https://github.com/loonghao/vx/commit/4b9c0ca532e70e61b380a025d7c66aa79143f2bf))
* **runtime-core:** remove dead Runtime trait and provider machinery ([56dbf1b](https://github.com/loonghao/vx/commit/56dbf1b6168591261a68c3e46bcb14f10fae2d3f))
* **runtime:** split runtime.rs into module and add ISP sub-traits ([3ebd68f](https://github.com/loonghao/vx/commit/3ebd68fb20469a4789c40cbcd9ae36212a568903))
* simplify all providers to PROVIDER_STAR only, remove redundant create_provider and star_metadata ([b232a8f](https://github.com/loonghao/vx/commit/b232a8fe7a76fbf62b9b39fa7e7e143a8ebe80c0))
* split tests to tests/ dir, extract bridge/builder modules, remove metadata indirection, fix clippy warnings ([84f6454](https://github.com/loonghao/vx/commit/84f645478bf1ca4c1abe9ef9667b82e2c56b4025))
* unify progress bars and restructure docs progressively ([#812](https://github.com/loonghao/vx/issues/812)) ([1889cf4](https://github.com/loonghao/vx/commit/1889cf44718ac98bff4b07ab2d96d62ab822fcb1))
* use LazyLock for regex compilation and improve error handling ([c72a14c](https://github.com/loonghao/vx/commit/c72a14c8196cca2865dbe69af3f9e363ad7e818b))
* **vx-starlark:** replace path-based cache with content-hash incremental analysis cache ([23c9918](https://github.com/loonghao/vx/commit/23c9918382c93a9081ffeb8b5dbbcfaf52a2e19e))


### Documentation

* add age and sops to tools overview and CHANGELOG ([9f8dff3](https://github.com/loonghao/vx/commit/9f8dff34dca7271d80aa83f9ee61266fb0254a51))
* add CLAUDE.md, .cursor/rules/*.mdc, and improve AI agent documentation ([#747](https://github.com/loonghao/vx/issues/747)) ([6dc6884](https://github.com/loonghao/vx/commit/6dc688452b21d68318743a0ca94f7b38d1fb9928))
* add complete Supported Tools section to llms-full.txt ([19c656d](https://github.com/loonghao/vx/commit/19c656dfaf612ab3a1ce6d0d6ed404231d04f8dd))
* add critical rules section to AGENTS.md for AI agents ([#869](https://github.com/loonghao/vx/issues/869)) ([c0e1470](https://github.com/loonghao/vx/commit/c0e14703c83c61f5e68ea67093de3094a81d0bce))
* add latest RFCs (0037, 0039, 0040) to llms-full.txt ([#873](https://github.com/loonghao/vx/issues/873)) ([6fecbd6](https://github.com/loonghao/vx/commit/6fecbd653ac6b4994e3e06c2f0ce8d5f3d539ab3))
* add llms.txt and llms-full.txt following llmstxt.org protocol ([6499b09](https://github.com/loonghao/vx/commit/6499b096a8eb0ad733ebe4b9af352ef32edd5c60))
* add missing tools (actrun, ctlptl, gws) to documentation ([b8cf5ac](https://github.com/loonghao/vx/commit/b8cf5ac01c8b2e2a3f77c1a3a54955fb76f1d0b0))
* add missing tools to documentation ([da73262](https://github.com/loonghao/vx/commit/da73262c8fa9efbcf59c4ada715b1ea4e82772c4))
* add more tool examples to AGENTS.md ([358ca9f](https://github.com/loonghao/vx/commit/358ca9fb72e0d223ddfbdd2f9b61ce1747367fc9))
* add Package Alias documentation (EN + ZH) ([2d2be12](https://github.com/loonghao/vx/commit/2d2be1233d741f93d54081b713ca0870f3d87771))
* add pre-commit hooks documentation (EN/ZH) and update contributing guides ([770a4f5](https://github.com/loonghao/vx/commit/770a4f535d89f905cd17a3913db349be98d506dc))
* add RFC 0032 and RFC 0033, add opencode skills ([370eb15](https://github.com/loonghao/vx/commit/370eb158d41858328ca5c5ce77b91f037eef1a1f))
* add self-update command documentation with channel support ([c8c413e](https://github.com/loonghao/vx/commit/c8c413ef488d6e75a0ca0c83129a9d654978bda3))
* add Starlark Providers advanced guide (bilingual) ([c649320](https://github.com/loonghao/vx/commit/c6493208d74687e4e29c6c0be097a666a67a1009))
* add worktrunk (wt) tool documentation ([721a00b](https://github.com/loonghao/vx/commit/721a00b964c4616ae64d66f82286eda2617fd92d))
* **agent:** add cross-language global install contract and fix RFC links ([2d6f0a8](https://github.com/loonghao/vx/commit/2d6f0a80535086f2dee2df6f476b42fb2c9e9cce))
* **cargo:** add fast build optimizations inspired by Bevy ([09c2311](https://github.com/loonghao/vx/commit/09c2311e8c06e26146c4b8d261a85e34fa196488))
* clarify companion tool injection applies to all tools, not just Node.js ([9ec51f1](https://github.com/loonghao/vx/commit/9ec51f135b5fc526b07930ffe6744986bab9f012))
* **cleanup:** sync provider count from 105 to 111 across all docs ([3ffd6af](https://github.com/loonghao/vx/commit/3ffd6aff8f9051e36c26dcfd88cb509850d233c7))
* enhance AI agent documentation and sync skills ([#736](https://github.com/loonghao/vx/issues/736)) ([20affa6](https://github.com/loonghao/vx/commit/20affa63f35c70ac51858d96192df2320ff22d5b))
* enhance AI agent documentation with decision framework, MCP guide, and version fixes ([#732](https://github.com/loonghao/vx/issues/732)) ([90774ee](https://github.com/loonghao/vx/commit/90774eeb518ec046eae7c570bb316f2ad52f9f11))
* enhance AI agent ecosystem with 15+ agent support ([#749](https://github.com/loonghao/vx/issues/749)) ([61bb0cb](https://github.com/loonghao/vx/commit/61bb0cb06a8097333af0f26227f88b797d83bcb4))
* fix dead links in docs build ([c18fa9b](https://github.com/loonghao/vx/commit/c18fa9b98737a67bb751a28729b96058b48458d0))
* fix duplicate Critical Rules sections in AGENTS.md ([#870](https://github.com/loonghao/vx/issues/870)) ([3c0fac2](https://github.com/loonghao/vx/commit/3c0fac2787788431a928019082dc7e3769372fba))
* fix syntax errors in other.md and quality.md ([#866](https://github.com/loonghao/vx/issues/866)) ([779b15c](https://github.com/loonghao/vx/commit/779b15cc4fd78259a1388beb5119202b44812b65))
* improve agent documentation for better AI discoverability ([#701](https://github.com/loonghao/vx/issues/701)) ([75eadff](https://github.com/loonghao/vx/commit/75eadff95031a1f5cd69828ce89997eac8daf75b))
* improve agent knowledge - update provider count to 78, enhance AGENTS.md, sync skills ([#687](https://github.com/loonghao/vx/issues/687)) ([7185a30](https://github.com/loonghao/vx/commit/7185a30d42d0f6fc5ea1c9685387363e1b6aba88))
* improve agent knowledge - update provider.star docs, fix tool counts, add creating-provider guide ([dcff435](https://github.com/loonghao/vx/commit/dcff43556fd7258478394af8c5dd3d623e2d9f1b))
* improve AI agent documentation and fix version inconsistencies ([#710](https://github.com/loonghao/vx/issues/710)) ([1f22ea4](https://github.com/loonghao/vx/commit/1f22ea4e2b25e5184ea7cc60219baab5d7ddbc0b))
* improve AI agent documentation ecosystem ([#741](https://github.com/loonghao/vx/issues/741)) ([298f340](https://github.com/loonghao/vx/commit/298f340eb0007ec36c3830eed1961bbe8051d78e))
* optimize agent docs for v0.8.20 — add Copilot instructions, expand AI agent support to 17+ ([#762](https://github.com/loonghao/vx/issues/762)) ([95f30f4](https://github.com/loonghao/vx/commit/95f30f43c4d91aa4da37ba0e826ca06728f5cd3b))
* optimize AGENTS.md as progressive disclosure map ([#868](https://github.com/loonghao/vx/issues/868)) ([abf8fbc](https://github.com/loonghao/vx/commit/abf8fbc33bbcfba7f81c33b528eb918af26c85d3))
* **rfc-0032:** update Plan D (hakari implemented), Plan E/F tracking status ([ad96225](https://github.com/loonghao/vx/commit/ad96225c7aaba5ac07f6979458468239dbf928a7))
* **rfc:** add RFC 0036 - Starlark Provider Support ([9f29ebf](https://github.com/loonghao/vx/commit/9f29ebf34c29aaed9c1c94918ae3a4a40f198ff3))
* **rfc:** update Phase 2 progress in RFC 0032 ([c70a461](https://github.com/loonghao/vx/commit/c70a46127563370e3373f783144548c56458e89e))
* **rfc:** update Phase 2 status in RFC 0032 ([29e25c7](https://github.com/loonghao/vx/commit/29e25c76522013523eeda1a409de702cf1d31d3a))
* **rfc:** update RFC 0036 v0.3 - add Buck2 typed provider_field, load() module system, incremental analysis cache, declarative actions ([58ac77d](https://github.com/loonghao/vx/commit/58ac77d60cabacfdd451b926ae83548fc2d4a5a0))
* simplify AI agent configs, add vx wt/witr examples ([4479958](https://github.com/loonghao/vx/commit/4479958fdb7817f59552028a4edd1faf55cb374a))
* **skill:** add new vx capabilities to SKILL.md ([5d4df51](https://github.com/loonghao/vx/commit/5d4df51474f1fcbfff100df7c63e0ab7194f2c18))
* sync zh contributing.md and add zh fixes docs ([ef91ea0](https://github.com/loonghao/vx/commit/ef91ea050d611b1f3ce6a58f413182e6b4367e38))
* update cargo-build-optimization agent rule with implemented optimizations ([82294c3](https://github.com/loonghao/vx/commit/82294c3e08dfe3642fdef4f8d3dd2d6d622bd665))
* update documentation for self-update and worktrunk ([137941f](https://github.com/loonghao/vx/commit/137941f05bcae3d146bb3b70ad46cba67092c5e0))
* update media.md and witr.md with vx-org/mirrors download source ([#864](https://github.com/loonghao/vx/issues/864)) ([4a11abe](https://github.com/loonghao/vx/commit/4a11abe77d0c8bbfb10755aa3e2d166f67c726c5))
* update outdated version numbers in README.md ([#874](https://github.com/loonghao/vx/issues/874)) ([39b4c71](https://github.com/loonghao/vx/commit/39b4c71781a958483f59df32d2afec0d4349b0fb))
* update provider count 136 -&gt; 137 (add witr) ([#859](https://github.com/loonghao/vx/issues/859)) ([075a973](https://github.com/loonghao/vx/commit/075a973e4a92afa5d93820e86f96a650aba1b8b6))
* update provider count from 129/131 to 132 ([499d830](https://github.com/loonghao/vx/commit/499d8302cb989a7109a14e1203d69ab8ca4adcc8))
* update provider count from 129/131 to 132 across all documentation ([025c682](https://github.com/loonghao/vx/commit/025c682eeb828ac3f1e64054ba68b6994149bea4))
* update provider count from 132/135 to 136 across all docs ([5dc335c](https://github.com/loonghao/vx/commit/5dc335c86225da3bbda2a03e99e5ce539e5d9b04))
* update provider count from 132/135 to 136 across all docs ([e736c47](https://github.com/loonghao/vx/commit/e736c47f2afae3e7a9911a9e86cc1b23565d7bb3))
* update provider count from 136 to 137 across documentation ([#865](https://github.com/loonghao/vx/issues/865)) ([c60f84d](https://github.com/loonghao/vx/commit/c60f84d90f4b41046222bfac2c587350d5371973))
* update provider count to 129 in AGENTS.md ([82b6a24](https://github.com/loonghao/vx/commit/82b6a243371d605ddf9c33965aaea0f6083451f6))
* update provider count to 131 in AGENTS.md ([ea47fd0](https://github.com/loonghao/vx/commit/ea47fd03f81e512745a417097d2b60874a58a4dc))
* update provider count to 132 and add conda/micromamba/mamba ([eef8af6](https://github.com/loonghao/vx/commit/eef8af681f11631004df19b70e90fac51985cf90))
* update provider count to 132 and add conda/micromamba/mamba to overview ([102d59a](https://github.com/loonghao/vx/commit/102d59ad65f2a870186b4aeacb1a8120b0d29c62))
* update provider count to 137 (add witr) ([8afd778](https://github.com/loonghao/vx/commit/8afd778c119b992a99c0e2604fa87916783fc43a))
* update rules to reflect provider.star migration ([3813f82](https://github.com/loonghao/vx/commit/3813f82a7dc7fd33257cafbc6eb1ea6635634ef1))
* update Rust version requirement in contributing.md ([160dcdd](https://github.com/loonghao/vx/commit/160dcdd0be3b99bc12b8e18beb92602ad2f09f0f))
* update Rust version requirement to 1.93+ (Edition 2024) in contributing.md ([e8e85d0](https://github.com/loonghao/vx/commit/e8e85d00f26e733a1eef81c860e10c5c50f33b90))
* update Rust version to 1.95.0+ and fix RFC count ([#875](https://github.com/loonghao/vx/issues/875)) ([843f734](https://github.com/loonghao/vx/commit/843f734cde9bc232759662039ab92d49270e4706))
* update tools overview and quality documentation ([80d1427](https://github.com/loonghao/vx/commit/80d1427772ef9e958f65db24587904f696e29bd2))
* update version to 0.8.32 and provider count to 129 ([3094821](https://github.com/loonghao/vx/commit/30948218e4f7347f2cec04365392666dcd3e3d0b))
* update version to v0.8.35 and provider count to 136 ([70f2615](https://github.com/loonghao/vx/commit/70f261552a25a12b609019837755655da95ce126))
* update version to v0.8.35 and provider count to 136 ([fddf63a](https://github.com/loonghao/vx/commit/fddf63a8382e0b18abf39eb64dd3db60ff17eab8))
* update version to v0.8.36 and provider count to 136 ([f973ff0](https://github.com/loonghao/vx/commit/f973ff02d28f2a51b2c63d83450a56c54aa56e85))
* update version to v0.8.36 and provider count to 136 ([ebd928b](https://github.com/loonghao/vx/commit/ebd928bb2d29ce230f29ff6c505c9c0702a56ada))
* update version to v0.8.37 ([149b0ba](https://github.com/loonghao/vx/commit/149b0ba0b988c1606de0a2406466cb83c30a5852))
* update version to v0.8.37 in AGENTS.md and README.md ([4a323eb](https://github.com/loonghao/vx/commit/4a323ebc10d21e0c2aef4bbdd8b7fb62c04506f0))
* update version to v0.8.39 ([d0c622b](https://github.com/loonghao/vx/commit/d0c622beb3d5976fe02109afb367e4613c44afe4))
* update version to v0.8.39 in AGENTS.md and README.md ([bb0d11e](https://github.com/loonghao/vx/commit/bb0d11e27ce7b53fbeb802f609768cfb1a4c727c))

## [0.8.39](https://github.com/loonghao/vx/compare/v0.8.38...v0.8.39) (2026-05-22)


### Bug Fixes

* **deps:** update rust crate starlark_derive to 0.14 ([82c274b](https://github.com/loonghao/vx/commit/82c274b29d5dd4deb80e2318dba7cdf97fea83e7))

## [0.8.38](https://github.com/loonghao/vx/compare/v0.8.37...v0.8.38) (2026-05-22)


### Bug Fixes

* avoid msvc repair for unrelated commands ([bb6b292](https://github.com/loonghao/vx/commit/bb6b29289687a620b4aa6a56ce9bf5f549aef6d1))
* **deps:** update rust crate toon-format to 0.5 ([bb23093](https://github.com/loonghao/vx/commit/bb23093796635b88b8de85cc5db526952664ee9e))


### Documentation

* add missing tools (actrun, ctlptl, gws) to documentation ([b8cf5ac](https://github.com/loonghao/vx/commit/b8cf5ac01c8b2e2a3f77c1a3a54955fb76f1d0b0))
* add missing tools to documentation ([da73262](https://github.com/loonghao/vx/commit/da73262c8fa9efbcf59c4ada715b1ea4e82772c4))
* fix dead links in docs build ([c18fa9b](https://github.com/loonghao/vx/commit/c18fa9b98737a67bb751a28729b96058b48458d0))
* update version to v0.8.37 ([149b0ba](https://github.com/loonghao/vx/commit/149b0ba0b988c1606de0a2406466cb83c30a5852))
* update version to v0.8.37 in AGENTS.md and README.md ([4a323eb](https://github.com/loonghao/vx/commit/4a323ebc10d21e0c2aef4bbdd8b7fb62c04506f0))

## [0.8.37](https://github.com/loonghao/vx/compare/v0.8.36...v0.8.37) (2026-05-20)


### Features

* add witr provider (137 providers total) ([e74d708](https://github.com/loonghao/vx/commit/e74d70837ff3452ae156f463735dff779d267ad9))


### Bug Fixes

* correct RFC count (40+ -&gt; 50+) and update Rust version badge (1.93+ -&gt; 1.95.0+) ([#879](https://github.com/loonghao/vx/issues/879)) ([a400f57](https://github.com/loonghao/vx/commit/a400f57db7fcd01763f1d15656b5da9dd1e6f404))
* ffmpeg use Gyan.dev mirror, witr only override download_url ([10a7a50](https://github.com/loonghao/vx/commit/10a7a5007e506a42f7a46c007c9332ae8204c2ad))
* **ffmpeg:** use system_install only (remove unreliable GitHub downloads) ([1f8780a](https://github.com/loonghao/vx/commit/1f8780adbe1df7c4cafaf5a6213cf1a2663b7d58))
* **ffmpeg:** use vx-org/mirrors with BtbN static builds (win64+linux64+linuxarm64) ([77d741c](https://github.com/loonghao/vx/commit/77d741c7f7f8a69358a70f9fa13d614d8bb9c81d))
* handle rust toolchain versions in path selection ([585353e](https://github.com/loonghao/vx/commit/585353ea73647b1a4c05d1c6fe02cc5e729fc25c))
* **providers:** use vx-org/mirrors for ffmpeg and witr downloads ([ac4dc6a](https://github.com/loonghao/vx/commit/ac4dc6a82dff09cc6cb21e236d50f71d4d9ab4ce))
* remove unused variables in witr/provider.star ([ef0d871](https://github.com/loonghao/vx/commit/ef0d87125f5d2a6c1371ae56cd6a95dfa45208da))
* use system_install for ffmpeg Linux, fix witr install_layout ([293a4f5](https://github.com/loonghao/vx/commit/293a4f5b621312be21fd5baaf6c1b3020c11b0eb))
* **witr:** correct __type__ key in install_layout (double underscores) ([b287fe5](https://github.com/loonghao/vx/commit/b287fe520f6c3df04a3babd887172110d50591fe))
* **witr:** correct version pattern and binary path in provider.star ([b6683e1](https://github.com/loonghao/vx/commit/b6683e1576ee1f943f7e6f78232588e5b4545f21))
* **witr:** override install_layout with correct type ([6bf7bbd](https://github.com/loonghao/vx/commit/6bf7bbd63a5e12730e79351f24282053e85c9e7c))
* **witr:** rewrite provider without template to handle direct binaries correctly ([dcd08cc](https://github.com/loonghao/vx/commit/dcd08ccdd5051e4de05cc743ef8ceb33a1362be7))
* **witr:** use 'binary' type for direct binaries (Linux/macOS) ([b37b591](https://github.com/loonghao/vx/commit/b37b591952902505e3220cfd087b9f0e79211860))


### Documentation

* add complete Supported Tools section to llms-full.txt ([19c656d](https://github.com/loonghao/vx/commit/19c656dfaf612ab3a1ce6d0d6ed404231d04f8dd))
* add critical rules section to AGENTS.md for AI agents ([#869](https://github.com/loonghao/vx/issues/869)) ([c0e1470](https://github.com/loonghao/vx/commit/c0e14703c83c61f5e68ea67093de3094a81d0bce))
* add latest RFCs (0037, 0039, 0040) to llms-full.txt ([#873](https://github.com/loonghao/vx/issues/873)) ([6fecbd6](https://github.com/loonghao/vx/commit/6fecbd653ac6b4994e3e06c2f0ce8d5f3d539ab3))
* add more tool examples to AGENTS.md ([358ca9f](https://github.com/loonghao/vx/commit/358ca9fb72e0d223ddfbdd2f9b61ce1747367fc9))
* add self-update command documentation with channel support ([c8c413e](https://github.com/loonghao/vx/commit/c8c413ef488d6e75a0ca0c83129a9d654978bda3))
* add worktrunk (wt) tool documentation ([721a00b](https://github.com/loonghao/vx/commit/721a00b964c4616ae64d66f82286eda2617fd92d))
* fix duplicate Critical Rules sections in AGENTS.md ([#870](https://github.com/loonghao/vx/issues/870)) ([3c0fac2](https://github.com/loonghao/vx/commit/3c0fac2787788431a928019082dc7e3769372fba))
* fix syntax errors in other.md and quality.md ([#866](https://github.com/loonghao/vx/issues/866)) ([779b15c](https://github.com/loonghao/vx/commit/779b15cc4fd78259a1388beb5119202b44812b65))
* optimize AGENTS.md as progressive disclosure map ([#868](https://github.com/loonghao/vx/issues/868)) ([abf8fbc](https://github.com/loonghao/vx/commit/abf8fbc33bbcfba7f81c33b528eb918af26c85d3))
* simplify AI agent configs, add vx wt/witr examples ([4479958](https://github.com/loonghao/vx/commit/4479958fdb7817f59552028a4edd1faf55cb374a))
* update documentation for self-update and worktrunk ([137941f](https://github.com/loonghao/vx/commit/137941f05bcae3d146bb3b70ad46cba67092c5e0))
* update media.md and witr.md with vx-org/mirrors download source ([#864](https://github.com/loonghao/vx/issues/864)) ([4a11abe](https://github.com/loonghao/vx/commit/4a11abe77d0c8bbfb10755aa3e2d166f67c726c5))
* update outdated version numbers in README.md ([#874](https://github.com/loonghao/vx/issues/874)) ([39b4c71](https://github.com/loonghao/vx/commit/39b4c71781a958483f59df32d2afec0d4349b0fb))
* update provider count 136 -&gt; 137 (add witr) ([#859](https://github.com/loonghao/vx/issues/859)) ([075a973](https://github.com/loonghao/vx/commit/075a973e4a92afa5d93820e86f96a650aba1b8b6))
* update provider count from 136 to 137 across documentation ([#865](https://github.com/loonghao/vx/issues/865)) ([c60f84d](https://github.com/loonghao/vx/commit/c60f84d90f4b41046222bfac2c587350d5371973))
* update provider count to 137 (add witr) ([8afd778](https://github.com/loonghao/vx/commit/8afd778c119b992a99c0e2604fa87916783fc43a))
* update Rust version requirement in contributing.md ([160dcdd](https://github.com/loonghao/vx/commit/160dcdd0be3b99bc12b8e18beb92602ad2f09f0f))
* update Rust version requirement to 1.93+ (Edition 2024) in contributing.md ([e8e85d0](https://github.com/loonghao/vx/commit/e8e85d00f26e733a1eef81c860e10c5c50f33b90))
* update Rust version to 1.95.0+ and fix RFC count ([#875](https://github.com/loonghao/vx/issues/875)) ([843f734](https://github.com/loonghao/vx/commit/843f734cde9bc232759662039ab92d49270e4706))
* update version to v0.8.36 and provider count to 136 ([f973ff0](https://github.com/loonghao/vx/commit/f973ff02d28f2a51b2c63d83450a56c54aa56e85))
* update version to v0.8.36 and provider count to 136 ([ebd928b](https://github.com/loonghao/vx/commit/ebd928bb2d29ce230f29ff6c505c9c0702a56ada))

## [0.8.36](https://github.com/loonghao/vx/compare/v0.8.35...v0.8.36) (2026-05-04)


### Documentation

* update version to v0.8.35 and provider count to 136 ([70f2615](https://github.com/loonghao/vx/commit/70f261552a25a12b609019837755655da95ce126))
* update version to v0.8.35 and provider count to 136 ([fddf63a](https://github.com/loonghao/vx/commit/fddf63a8382e0b18abf39eb64dd3db60ff17eab8))

## [0.8.35](https://github.com/loonghao/vx/compare/v0.8.34...v0.8.35) (2026-05-03)


### Bug Fixes

* **ci:** ensure Release workflow triggers even when release is created from update-pr job ([7259bc6](https://github.com/loonghao/vx/commit/7259bc6fab2b45aad62fc4c7c583577f6157209f))

## [0.8.34](https://github.com/loonghao/vx/compare/v0.8.33...v0.8.34) (2026-05-03)


### Features

* add age and sops providers, update project analyzer frameworks ([f048f10](https://github.com/loonghao/vx/commit/f048f1000b6471d905b143bb6aae8c9ea832fd93))
* add worktrunk provider (132 tools total) ([#839](https://github.com/loonghao/vx/issues/839)) ([79c179d](https://github.com/loonghao/vx/commit/79c179d1a20ceed3b58335763ea74d61eb7f8c79))
* **providers:** add conda provider with micromamba, conda and mamba ([94f1aec](https://github.com/loonghao/vx/commit/94f1aec09ccc9a363a883d830fe29816f43d6a0f))


### Bug Fixes

* **providers:** correct install_layout strip_prefix and download_url ([3cad3c9](https://github.com/loonghao/vx/commit/3cad3c97c4578b5ca228f3765b60ea7f823dedbd))


### Code Refactoring

* **provider:** conda use provider.star only (remove Rust code) ([4ac74a6](https://github.com/loonghao/vx/commit/4ac74a66261ce771619e7120c449e522cdaae6a5))


### Documentation

* add age and sops to tools overview and CHANGELOG ([9f8dff3](https://github.com/loonghao/vx/commit/9f8dff34dca7271d80aa83f9ee61266fb0254a51))
* update provider count from 129/131 to 132 ([499d830](https://github.com/loonghao/vx/commit/499d8302cb989a7109a14e1203d69ab8ca4adcc8))
* update provider count from 129/131 to 132 across all documentation ([025c682](https://github.com/loonghao/vx/commit/025c682eeb828ac3f1e64054ba68b6994149bea4))
* update provider count from 132/135 to 136 across all docs ([5dc335c](https://github.com/loonghao/vx/commit/5dc335c86225da3bbda2a03e99e5ce539e5d9b04))
* update provider count from 132/135 to 136 across all docs ([e736c47](https://github.com/loonghao/vx/commit/e736c47f2afae3e7a9911a9e86cc1b23565d7bb3))
* update provider count to 131 in AGENTS.md ([ea47fd0](https://github.com/loonghao/vx/commit/ea47fd03f81e512745a417097d2b60874a58a4dc))
* update provider count to 132 and add conda/micromamba/mamba ([eef8af6](https://github.com/loonghao/vx/commit/eef8af681f11631004df19b70e90fac51985cf90))
* update provider count to 132 and add conda/micromamba/mamba to overview ([102d59a](https://github.com/loonghao/vx/commit/102d59ad65f2a870186b4aeacb1a8120b0d29c62))

## [0.8.33](https://github.com/loonghao/vx/compare/v0.8.32...v0.8.33) (2026-04-30)


### Features

* **providers:** add age encryption tool provider
* **providers:** add sops secrets management provider
* **project-analyzer:** add framework detection for Bun, Deno, Nix, Zig


### Performance Improvements

* add cargo build optimization agent rule ([f042114](https://github.com/loonghao/vx/commit/f042114000e11bd3fc94fc3c585aac1baf8511a8))
* optimize workspace compilation settings ([af082f7](https://github.com/loonghao/vx/commit/af082f777eaa521534181a924060bcd35e67c48f))


### Code Refactoring

* merge vx-core into vx-runtime-core ([d91989e](https://github.com/loonghao/vx/commit/d91989e3d1bd667106e43c0b0f0dfe41885bf862))


### Documentation

* update cargo-build-optimization agent rule with implemented optimizations ([82294c3](https://github.com/loonghao/vx/commit/82294c3e08dfe3642fdef4f8d3dd2d6d622bd665))
* update provider count to 129 in AGENTS.md ([82b6a24](https://github.com/loonghao/vx/commit/82b6a243371d605ddf9c33965aaea0f6083451f6))
* update tools overview and quality documentation ([80d1427](https://github.com/loonghao/vx/commit/80d1427772ef9e958f65db24587904f696e29bd2))
* update version to 0.8.32 and provider count to 129 ([3094821](https://github.com/loonghao/vx/commit/30948218e4f7347f2cec04365392666dcd3e3d0b))

## [0.8.32](https://github.com/loonghao/vx/compare/v0.8.31...v0.8.32) (2026-04-25)


### Features

* bridge global install commands to vx package shim workflow ([0d7ecf2](https://github.com/loonghao/vx/commit/0d7ecf289dbd637c708bfc7c2271cd931aad5571))
* **cli:** bridge global install commands to vx package shims ([c753951](https://github.com/loonghao/vx/commit/c7539518170f28b62c5f8ec5dc330e741808e60e))
* **cli:** enable direct global command shims in vx bin dir ([98131e4](https://github.com/loonghao/vx/commit/98131e429a8c9b7be83c7ee12c15975267ff83b2))


### Documentation

* **agent:** add cross-language global install contract and fix RFC links ([2d6f0a8](https://github.com/loonghao/vx/commit/2d6f0a80535086f2dee2df6f476b42fb2c9e9cce))

## [0.8.31](https://github.com/loonghao/vx/compare/v0.8.30...v0.8.31) (2026-04-18)


### Features

* **cli:** overhaul vx add/remove with format-preserving edits ([c391912](https://github.com/loonghao/vx/commit/c39191214ea7b8782347ae52b1dd7726f5d2c667))

## [0.8.30](https://github.com/loonghao/vx/compare/v0.8.29...v0.8.30) (2026-04-17)


### Code Refactoring

* unify progress bars and restructure docs progressively ([#812](https://github.com/loonghao/vx/issues/812)) ([1889cf4](https://github.com/loonghao/vx/commit/1889cf44718ac98bff4b07ab2d96d62ab822fcb1))

## [0.8.29](https://github.com/loonghao/vx/compare/v0.8.28...v0.8.29) (2026-04-16)


### Features

* change non-TTY default output from JSON to Toon, disable CDN by default ([cc46de7](https://github.com/loonghao/vx/commit/cc46de76e29412fc4cd9cbc726587ed7ad7c1dba))


### Bug Fixes

* **ci:** switch apt mirror to azure.archive.ubuntu.com for cross builds ([344e043](https://github.com/loonghao/vx/commit/344e04310761b34ce16443854c6f06c6c9c9ae91))
* **docker:** switch apt mirror to azure.archive.ubuntu.com in Dockerfiles and test workflow ([c95f733](https://github.com/loonghao/vx/commit/c95f7335208129fdc988bce102338225d44b0ef9))
* git uses MinGit ZIP on Windows, rust toolchain defaults to stable ([da6405d](https://github.com/loonghao/vx/commit/da6405d17c38cc39a29a46136ef2fb734d4cc95a))

## [0.8.28](https://github.com/loonghao/vx/compare/v0.8.27...v0.8.28) (2026-04-15)


### Features

* add FilterLevel enum (Light/Normal/Aggressive) for compact output ([#804](https://github.com/loonghao/vx/issues/804)) ([876731f](https://github.com/loonghao/vx/commit/876731f8cb495f403d66e1aeeb9a3ab4c3ea94bb))

## [0.8.27](https://github.com/loonghao/vx/compare/v0.8.26...v0.8.27) (2026-04-14)


### Features

* add vx-output-filter crate for compact subprocess output ([#802](https://github.com/loonghao/vx/issues/802)) ([ba69f04](https://github.com/loonghao/vx/commit/ba69f0402b90b318a90510ec4459d99337aaa5c3))


### Bug Fixes

* resolve merge conflict in release manifest ([deab703](https://github.com/loonghao/vx/commit/deab7036e02c58b42e68a99b0d62f5db013fc70f))

## [0.8.26](https://github.com/loonghao/vx/compare/v0.8.25...v0.8.26) (2026-04-14)


### Features

* **auto-improve:** squash merge auto-improve branch ([2797ede](https://github.com/loonghao/vx/commit/2797ede7ae83210a15606321ceab7e8775389b8c))


### Bug Fixes

* correct cmake macOS download URL and jq environment key ([e47d36b](https://github.com/loonghao/vx/commit/e47d36b39a3846a5585aefa16c173975b9e89b46))
* correct download URLs for grpcurl, k3d, kind, and duckdb providers ([108e290](https://github.com/loonghao/vx/commit/108e290654d06867bae9f837301d5a8c96cf09a0))
* **deps:** update rust crate hashbrown-986da7b5efc2b80e to 0.17 - abandoned ([d934ad1](https://github.com/loonghao/vx/commit/d934ad179813669f0c343585019676eb32a82913))
* **engine:** ctx.install_dir now points to actual install location ([19237c4](https://github.com/loonghao/vx/commit/19237c415bbb75c195f9300f6ed91e6202c46e2f))
* **installer:** apply binary rename fix to RealInstaller.download_with_layout ([925d6d5](https://github.com/loonghao/vx/commit/925d6d54f963dcd1ee4bb96209eef984a356827c))
* **installer:** fix PortableGit .7z.exe not recognized as archive + stop version fallback on layout errors ([d62adba](https://github.com/loonghao/vx/commit/d62adba7b119807c5f35d70a93403f1b8ece2272))
* **paths:** detect unified runtime store versions ([574c8ff](https://github.com/loonghao/vx/commit/574c8ff581b5d9e7f2b844d1cb1c35e175281a59))
* **provider:** correct grpcurl version check ([7c917ef](https://github.com/loonghao/vx/commit/7c917ef4e660127a5c8fa343fc421dba8eb38093))
* **providers:** fix 5 more provider bugs (round 2) + add tar.bz2 support ([88874cd](https://github.com/loonghao/vx/commit/88874cdf2884f62cfc8dce9f3a56623842c9559a))
* **providers:** fix 5 provider bugs from auto-improve branch ([c6938cf](https://github.com/loonghao/vx/commit/c6938cf760762d1c874280dc8f8f66086fdfa1c3))
* **providers:** fix binary rename, grpcurl macOS, duckdb macOS, nerdctl platform ([4a07643](https://github.com/loonghao/vx/commit/4a076431b43b45e00e2e1e38862008f8700fce66))
* **provider:** use gnu rustup triples on linux ([a1984e2](https://github.com/loonghao/vx/commit/a1984e21a8de4fcc4eca1ef1dd573467915399ac))
* remove platform subdir from install path, fix providers ([c80f726](https://github.com/loonghao/vx/commit/c80f726c6b9a5b54ea573bbbde0d2ec83528036a))
* resolve CI failures for lefthook, grpcurl, and kustomize providers ([3bcc0ec](https://github.com/loonghao/vx/commit/3bcc0ecd4c1eaf0cabb76bd83b0eb95801b6033f))
* resolve merge conflict in release manifest ([deab703](https://github.com/loonghao/vx/commit/deab7036e02c58b42e68a99b0d62f5db013fc70f))
* **resolver:** resolve bundled runtime fallback executable ([099ff92](https://github.com/loonghao/vx/commit/099ff92fc7d1f87f3b9d7857d6f7d4bd9811b25c))
* **resolver:** stop re-installing system-managed runtimes on every vx invocation ([83d239b](https://github.com/loonghao/vx/commit/83d239b29af6304e7dfee21ae6b48a8575200220))
* **runtime:** preserve bundled command prefixes ([821e779](https://github.com/loonghao/vx/commit/821e779c2f1f96d9dfa6a7c5fa43a393aaf90d85))
* **runtime:** satisfy clippy collapsible-if lint ([51dc8f4](https://github.com/loonghao/vx/commit/51dc8f4d2eb6f79b011bd1d18f9f486b35701973))
* **rust:** stop re-installing rust on every vx cargo invocation ([988858c](https://github.com/loonghao/vx/commit/988858c665d1faebe267bfba8d2b1a76c26e468a))
* **tests:** resolve latest unit test failures ([5c9684f](https://github.com/loonghao/vx/commit/5c9684fbc7c4102a596ea0b69cc5f25044e93586))
* **uv:** route uvx through uv tool run ([7b65a63](https://github.com/loonghao/vx/commit/7b65a63ed28d15b158b7c500a56dcf567e97dff7))
* wire Starlark post_extract hooks into ManifestDrivenRuntime and fix bundled runtime detection ([0cab93c](https://github.com/loonghao/vx/commit/0cab93c60355e1ea12fc9b332a41e1fbf18d46d8))

## [0.8.25](https://github.com/loonghao/vx/compare/v0.8.24...v0.8.25) (2026-04-10)


### Bug Fixes

* git Windows exe path, rust bundled store mismatch, lock multi-platform URLs + perf optimizations ([#787](https://github.com/loonghao/vx/issues/787)) ([f208ef5](https://github.com/loonghao/vx/commit/f208ef535b434e3eab4d0e047ac7a61e164806d1))


## [0.8.24](https://github.com/loonghao/vx/compare/v0.8.23...v0.8.24) (2026-04-09)


### Features

* **ecosystem_aliases:** route ecosystem:package to dedicated provider binary ([e7ccfb4](https://github.com/loonghao/vx/commit/e7ccfb438fda1eff06f5c55c00642389a57dbbad))
* **providers:** add cargo-audit provider ([dc2734a](https://github.com/loonghao/vx/commit/dc2734a1157ac168b3c12115811c1b3459c4308a))
* **providers:** add cargo-nextest and cargo-deny providers ([a484a8b](https://github.com/loonghao/vx/commit/a484a8b99c7266924bdf5511d7eab24ce542b3ee))
* **providers:** add grpcurl provider + update provider count to 114 ([906d97e](https://github.com/loonghao/vx/commit/906d97ea0c45496f2bb43d74426cf9b879403d11))
* **providers:** add kind and k3d providers ([ea33834](https://github.com/loonghao/vx/commit/ea338348cd3e53f350a1cc0f78fc5f708c5090f4))
* **routing:** prefer dedicated provider over cargo install for ecosystem:package ([1cefc75](https://github.com/loonghao/vx/commit/1cefc75f8e8a0ef4a8b845a2aff2994c33eb0ab8))


### Bug Fixes

* **cargo-audit:** remove unused rust_triple import (lint) ([48a1a87](https://github.com/loonghao/vx/commit/48a1a87ea8b90c1634c0b41db33fd3040e0ca38d))
* **cleanup:** fix compile errors from ecosystem_aliases feature ([d89bc38](https://github.com/loonghao/vx/commit/d89bc38489f7f670d738d199b866027cf7965dff))
* **console:** use eprintln for progress output to avoid stdout contamination ([7ebf3e7](https://github.com/loonghao/vx/commit/7ebf3e7dc4db23a13927b9e65d4ee5c93618d2ac))
* **docs:** fix broken doctests in vx-console and vx-starlark ([58d88e8](https://github.com/loonghao/vx/commit/58d88e8ac48cd2e9ad73e607b9e5bbc15282f8c9))
* **providers:** add fetch_versions_with_tag_prefix to layout mock + fix cargo-deny Windows ([eea4d7e](https://github.com/loonghao/vx/commit/eea4d7e813170e8a9e6e05ba4f36e3fbdcd01282))
* **providers:** fix cargo-nextest macOS triple and cargo-audit test assertions ([86186d9](https://github.com/loonghao/vx/commit/86186d9cfd1c5c53af16b64ca352ef6a597280aa))
* **providers:** fix download URLs for cargo-audit, cargo-nextest, and deno ([66555ed](https://github.com/loonghao/vx/commit/66555ed3e2a642a9965e209f3b597e033435d8bc))
* **tests:** fix 14 failing tests across multiple crates ([5069772](https://github.com/loonghao/vx/commit/5069772a3a7a9aa9edca8a8581acdb00d696b7e9))
* **tests:** fix cargo-deny Windows URL test and add missing provider tests ([aaae704](https://github.com/loonghao/vx/commit/aaae704b513bcf018ed115e1ba1443a9a9a8da31))


### Code Refactoring

* **providers:** simplify providers to use standard templates ([0085259](https://github.com/loonghao/vx/commit/008525996aebffc16cdd56422efeece7f03ee1aa))


### Documentation

* **cleanup:** sync provider count from 105 to 111 across all docs ([3ffd6af](https://github.com/loonghao/vx/commit/3ffd6aff8f9051e36c26dcfd88cb509850d233c7))

## [0.8.23](https://github.com/loonghao/vx/compare/v0.8.22...v0.8.23) (2026-04-08)


### Bug Fixes

* **providers:** fix download URL bugs in git, xmake, and ollama ([#777](https://github.com/loonghao/vx/issues/777)) ([0287fff](https://github.com/loonghao/vx/commit/0287fff7a6e7f86b8a8cd07aa06be004158678fc))

## [0.8.22](https://github.com/loonghao/vx/compare/v0.8.21...v0.8.22) (2026-04-07)


### Features

* **providers:** add tokei provider + triage stale issues ([f1077a1](https://github.com/loonghao/vx/commit/f1077a15b5224bf289af15f67f0ee710063cb29b))

## [0.8.21](https://github.com/loonghao/vx/compare/v0.8.20...v0.8.21) (2026-04-07)


### Features

* propagate explicit version from bundled runtime to parent dependency ([#766](https://github.com/loonghao/vx/issues/766)) ([b48abe6](https://github.com/loonghao/vx/commit/b48abe6aaceec92455830c2476770a4657fecd65))


### Bug Fixes

* gracefully resolve numeric version hints for pure opaque providers ([4891b84](https://github.com/loonghao/vx/commit/4891b84942b85972e53e129937545b4e311ff633))
* resolve sha2 LowerHex compile errors and upgrade GitHub Actions ([d148868](https://github.com/loonghao/vx/commit/d148868e133136b9d33627d0a793dadffa9ee5af))
* **vx-config:** fix sha2 GenericArray LowerHex compile error ([0c60d1b](https://github.com/loonghao/vx/commit/0c60d1b3f88f6378aa7d80eeb49f5f63d2c2abb4))


### Documentation

* optimize agent docs for v0.8.20 — add Copilot instructions, expand AI agent support to 17+ ([#762](https://github.com/loonghao/vx/issues/762)) ([95f30f4](https://github.com/loonghao/vx/commit/95f30f43c4d91aa4da37ba0e826ca06728f5cd3b))

## [0.8.20](https://github.com/loonghao/vx/compare/v0.8.19...v0.8.20) (2026-04-05)


### Documentation

* enhance AI agent ecosystem with 15+ agent support ([#749](https://github.com/loonghao/vx/issues/749)) ([61bb0cb](https://github.com/loonghao/vx/commit/61bb0cb06a8097333af0f26227f88b797d83bcb4))

## [0.8.19](https://github.com/loonghao/vx/compare/v0.8.18...v0.8.19) (2026-04-05)


### Features

* **rfc-0040:** implement version_info() for toolchain version indirection ([8771443](https://github.com/loonghao/vx/commit/8771443299c5dbbf6c2160ea48c2f7aa5c9af4c1))


### Bug Fixes

* **ci:** handle skipped/cancelled jobs in CI Success gate ([18ebc6c](https://github.com/loonghao/vx/commit/18ebc6c2242ec56e580b71f0b377336b82860af0))
* **cli:** fix vx check system_fallback and vx lock for installed tools ([06b47b0](https://github.com/loonghao/vx/commit/06b47b0368a0952d213286a6ddd726b6df56eee4))
* handle JSON output in where command e2e tests ([8ea99c4](https://github.com/loonghao/vx/commit/8ea99c4aa03a98ff8cc066ab47a16e9a6c4c8696))


### Documentation

* add CLAUDE.md, .cursor/rules/*.mdc, and improve AI agent documentation ([#747](https://github.com/loonghao/vx/issues/747)) ([6dc6884](https://github.com/loonghao/vx/commit/6dc688452b21d68318743a0ca94f7b38d1fb9928))

## [0.8.18](https://github.com/loonghao/vx/compare/v0.8.17...v0.8.18) (2026-04-03)


### Features

* **cli:** Agent DX improvements for AI agents ([cc805e0](https://github.com/loonghao/vx/commit/cc805e0e285f1640961e1a9ac0f4802b243c7658))


### Bug Fixes

* **tests:** fix output_tests and info_tests failures in non-TTY CI ([c3ab0bd](https://github.com/loonghao/vx/commit/c3ab0bd30dfaa5737eb3846a203ee4bcdb47dfb1))

## [0.8.17](https://github.com/loonghao/vx/compare/v0.8.16...v0.8.17) (2026-04-02)


### Bug Fixes

* **ci:** replace curl POST with clawhub CLI in sync-skills workflow ([94ab959](https://github.com/loonghao/vx/commit/94ab959d6b2b1bf53dc41e33b6274428a2fb4938))


### Documentation

* improve AI agent documentation ecosystem ([#741](https://github.com/loonghao/vx/issues/741)) ([298f340](https://github.com/loonghao/vx/commit/298f340eb0007ec36c3830eed1961bbe8051d78e))

## [0.8.16](https://github.com/loonghao/vx/compare/v0.8.15...v0.8.16) (2026-04-01)


### Features

* add 11 new providers (mise, gitleaks, biome, lazydocker, k9s, gping, watchexec, duf, trippy, sd, actionlint) ([7874ac8](https://github.com/loonghao/vx/commit/7874ac821a2d5798910a3d33807f35050d3d2b29))
* add 7 high-priority developer tool providers (lazygit, delta, hyperfine, zoxide, atuin, chezmoi, eza) ([b221a45](https://github.com/loonghao/vx/commit/b221a45f46e13c6b45d17adf7de32323f65cb923))
* add 7 new providers (tealdeer, dust, xh, bottom, trivy, zellij, dive) ([5aa0e2d](https://github.com/loonghao/vx/commit/5aa0e2da9af225ca010ad18e1d852f5292d0fde3))
* add helix and yazi providers ([acf70c3](https://github.com/loonghao/vx/commit/acf70c3940812116cf5fb85ac65e389cde028262))


### Bug Fixes

* **ci:** sanitize provider cache keys ([11e46fe](https://github.com/loonghao/vx/commit/11e46fe6fe5f2bce70bac911beea15cc35dde42f))
* **eza:** add platform_constraint to skip macOS in CI tests ([65c9571](https://github.com/loonghao/vx/commit/65c9571af9fea6a48ac3599e501c357279cf2e84))
* **gcloud:** update starlark test to use __type field ([2a35711](https://github.com/loonghao/vx/commit/2a357115eea69195be4285e73e00bdbf9747f684))
* import env_prepend from env.star instead of provider.star ([f63a814](https://github.com/loonghao/vx/commit/f63a814c395205c95fed8f87149e42df11991dab))
* **mise:** avoid strip_prefix on Windows to prevent Access Denied errors ([a955566](https://github.com/loonghao/vx/commit/a95556696b2be579f3750e5c383d90e3142c79d0))
* **mise:** update unit tests to match new install_layout implementation ([4d2946f](https://github.com/loonghao/vx/commit/4d2946fcb121edee0518079390d1a2026aae3745))
* **mise:** use strip_prefix='mise/bin' on Windows to avoid shim detection error ([5c06c76](https://github.com/loonghao/vx/commit/5c06c765dcb6634d3ed8adc7618e81ff49c34d92))
* **providers:** fix dust and eza macOS download URL 404 ([5eb5b20](https://github.com/loonghao/vx/commit/5eb5b201ae34359cf7a8a29e0cc4b555f03247b2))
* **providers:** fix dust version pattern and tealdeer binary rename ([3f3cd31](https://github.com/loonghao/vx/commit/3f3cd31ec6e83b75d6858a4a1a446539e5672ecc))
* **providers:** fix gcloud get_execute_path and terraform fetch_versions ([5c2505d](https://github.com/loonghao/vx/commit/5c2505da487877fbe0331f7704bd83403d061853))
* **providers:** resolve CI issues for new provider batch ([2b229ab](https://github.com/loonghao/vx/commit/2b229ab21a012fff3d200997ab892000431eff50))
* **providers:** resolve tealdeer and mise install layouts ([ed6b6b6](https://github.com/loonghao/vx/commit/ed6b6b630910eec5374c5eff820ab5d88f1f8a56))
* **watchexec:** remove unused load imports to pass provider static lint ([e4da30a](https://github.com/loonghao/vx/commit/e4da30ab193eba7dc1f809554f10f1b09a2324e6))
* **watchexec:** use .zip on Windows, .tar.xz on Linux/macOS ([6c8e978](https://github.com/loonghao/vx/commit/6c8e978fd60c424bb91d3da4b8898ab7a65d0d01))


### Documentation

* enhance AI agent documentation and sync skills ([#736](https://github.com/loonghao/vx/issues/736)) ([20affa6](https://github.com/loonghao/vx/commit/20affa63f35c70ac51858d96192df2320ff22d5b))
* enhance AI agent documentation with decision framework, MCP guide, and version fixes ([#732](https://github.com/loonghao/vx/issues/732)) ([90774ee](https://github.com/loonghao/vx/commit/90774eeb518ec046eae7c570bb316f2ad52f9f11))

## [0.8.15](https://github.com/loonghao/vx/compare/v0.8.14...v0.8.15) (2026-03-30)


### Bug Fixes

* clippy useless_vec warnings in tests ([0028d2b](https://github.com/loonghao/vx/commit/0028d2b9cfcfe79f7e4a0de8625c481f6c84a70e))
* Python install fails due to version_date cache key mismatch ([9ab1ea4](https://github.com/loonghao/vx/commit/9ab1ea4b25e83d87daf5823b3e871a5fa4a95ff2))
* Rust ecosystem passthrough for rustc versions in resolve_version ([a18548a](https://github.com/loonghao/vx/commit/a18548abbf479a075218f77c5cb7b4866fef1742))

## [0.8.14](https://github.com/loonghao/vx/compare/v0.8.13...v0.8.14) (2026-03-28)


### Bug Fixes

* auto-fetch versions when version_date cache miss in download_url ([fa81b5f](https://github.com/loonghao/vx/commit/fa81b5fba84da4632c1068f112ab8880dd44342c))

## [0.8.13](https://github.com/loonghao/vx/compare/v0.8.12...v0.8.13) (2026-03-28)


### Features

* add well-known Python version fallback for python-build-standalone ([35f85fd](https://github.com/loonghao/vx/commit/35f85fddc8c9fcf3957c5c4847c6e6a80a26b608))


### Bug Fixes

* preserve Rust MSRV in vx.toml and enable passthrough for Rust ecosystem ([1ded9c9](https://github.com/loonghao/vx/commit/1ded9c98c0e740e17f50df4b239abbec2a11c040))

## [0.8.12](https://github.com/loonghao/vx/compare/v0.8.11...v0.8.12) (2026-03-28)


### Bug Fixes

* **ci:** split release-please into two jobs to fix tag creation ([7dd72cf](https://github.com/loonghao/vx/commit/7dd72cfa6760528d8305ed264bd1fb9b9d70a20e))

## [0.8.11](https://github.com/loonghao/vx/compare/v0.8.10...v0.8.11) (2026-03-28)


### Bug Fixes

* Add 'if: !startsWith(github.event.head_commit.message, chore: release)' guards to skip these workflows when the push is a release commit. ([d32009f](https://github.com/loonghao/vx/commit/d32009f24555fadde638248c3a17ff0ebb5db644))
* **ci:** include Cargo.lock in workspace-hack commit step ([0e75aab](https://github.com/loonghao/vx/commit/0e75aab10237ad9521727751b567aca9792392a5))
* **ci:** prevent duplicate release-please PRs on release merge ([d32009f](https://github.com/loonghao/vx/commit/d32009f24555fadde638248c3a17ff0ebb5db644)), closes [#713](https://github.com/loonghao/vx/issues/713)
* **dist:** exclude vx-star-metadata from cargo-dist release artifacts ([2c30069](https://github.com/loonghao/vx/commit/2c3006951d57bea85b8391628704437872ea9e1a))


### Code Refactoring

* improve code quality - replace unwrap() and eprintln! with proper error handling ([a2c4c0b](https://github.com/loonghao/vx/commit/a2c4c0b05ec9abebf0cead84e61b70f058f80b62))

## [0.8.10](https://github.com/loonghao/vx/compare/v0.8.9...v0.8.10) (2026-03-28)


### Bug Fixes

* **ci:** exclude vx-msbuild-bridge from cargo-dist & improve skills sync ([2ca24a3](https://github.com/loonghao/vx/commit/2ca24a3962f66295da4022feee6ca13b8aa98698))
* **ci:** replace remaining vx run cargo scripts with direct vx cargo calls ([948d26a](https://github.com/loonghao/vx/commit/948d26a40081e726abd86aa12b56b4f922bc2deb))
* **ci:** use vx cargo prefix in justfile recipes for CI compatibility ([5984668](https://github.com/loonghao/vx/commit/5984668e65c61beea025e9471374fac30e442050))
* make E2E version list tests resilient to transient network errors ([99d8eba](https://github.com/loonghao/vx/commit/99d8eba57182fb1cd6273599c260cc9b50f103df))


### Code Refactoring

* improve code safety by eliminating unsafe unwrap calls ([d8778c7](https://github.com/loonghao/vx/commit/d8778c72834f4e9deb619400b69318b0209209f2))
* use LazyLock for regex compilation and improve error handling ([c72a14c](https://github.com/loonghao/vx/commit/c72a14c8196cca2865dbe69af3f9e363ad7e818b))


### Documentation

* improve AI agent documentation and fix version inconsistencies ([#710](https://github.com/loonghao/vx/issues/710)) ([1f22ea4](https://github.com/loonghao/vx/commit/1f22ea4e2b25e5184ea7cc60219baab5d7ddbc0b))

## [0.8.9](https://github.com/loonghao/vx/compare/v0.8.8...v0.8.9) (2026-03-26)


### Documentation

* improve agent documentation for better AI discoverability ([#701](https://github.com/loonghao/vx/issues/701)) ([75eadff](https://github.com/loonghao/vx/commit/75eadff95031a1f5cd69828ce89997eac8daf75b))

## [0.8.8](https://github.com/loonghao/vx/compare/v0.8.7...v0.8.8) (2026-03-26)


### Bug Fixes

* resolve Python PYTHONHOME mismatch ([#696](https://github.com/loonghao/vx/issues/696)), improve version pagination, unify skills ([fbadc74](https://github.com/loonghao/vx/commit/fbadc745050d6ff951ea4822ad65eda807f344ca))

## [0.8.7](https://github.com/loonghao/vx/compare/v0.8.6...v0.8.7) (2026-03-26)


### Bug Fixes

* add a ound flag so END block only prints when the match block did not. Also add head -1 safety to ensure only one line is captured. ([15ae81f](https://github.com/loonghao/vx/commit/15ae81f944b6e77f370cc00336bf2a4a1e39fd40))
* **install:** prevent awk double-output in resolve_latest_version ([15ae81f](https://github.com/loonghao/vx/commit/15ae81f944b6e77f370cc00336bf2a4a1e39fd40))
* **install:** skip releases without binary assets in version resolution ([ff7438a](https://github.com/loonghao/vx/commit/ff7438a2491346cbfe1c0bf941b1f206615957a4))
* **release:** disable sccache rustc-wrapper in release workflow ([b202568](https://github.com/loonghao/vx/commit/b202568208b421a9eaa1e7f76a44d9900b58181a))

## [0.8.6](https://github.com/loonghao/vx/compare/v0.8.5...v0.8.6) (2026-03-26)


### Bug Fixes

* **deps:** update rust crate anstream to v1 ([b837fcd](https://github.com/loonghao/vx/commit/b837fcdbca1344095e65f1523c65bec4c01d04bd))

## [0.8.5](https://github.com/loonghao/vx/compare/v0.8.4...v0.8.5) (2026-03-26)


### Features

* add build cache providers (sccache, ccache, buildcache) ([62dbacb](https://github.com/loonghao/vx/commit/62dbacb7d67aafc5a0cd2208e4543c444992d923))
* add dynamic package_prefixes from provider.star metadata ([2bfddd2](https://github.com/loonghao/vx/commit/2bfddd219b381c10c63099ed8fcabf0dfb4ad66b))
* add llvm/conan/xmake/wix providers and enhance msvc/msbuild for C++/C# build automation ([cfc336a](https://github.com/loonghao/vx/commit/cfc336af1f1b64cb404353253ba5d48f1dad3b91))
* add new oneshot runner ecosystems and update CLI help/docs ([eb78070](https://github.com/loonghao/vx/commit/eb78070dd4e5f1f7a188f9a032bd46e3f1a0bce1))
* add Nx and Turborepo cache providers, fix CI sccache issue ([fdb59a3](https://github.com/loonghao/vx/commit/fdb59a3dd83dacbb7edb57259363e271da867b4e))
* add starlark provider support with bash, curl, meson, openssl, pre-commit, release-please, rez, systemctl, xcodebuild providers ([4d0ff41](https://github.com/loonghao/vx/commit/4d0ff418b0efb9aaccd9d2740d8e3436f0c4bc35))
* add vx skill for ClawHub publication ([#638](https://github.com/loonghao/vx/issues/638)) ([0430588](https://github.com/loonghao/vx/commit/0430588a71d78ff7901bf4edb5ac31dbae9301f7))
* land provider and CLI improvements ([f173ca7](https://github.com/loonghao/vx/commit/f173ca7859a08d3b019d45046fab6064a21e870b))
* **list:** sort tool list alphabetically (a-z) ([46147f9](https://github.com/loonghao/vx/commit/46147f93f0866a758972524d648b0080dc2e769f))
* **resolver:** implement version priority with vx.lock support ([37cda5e](https://github.com/loonghao/vx/commit/37cda5e5d8dac4635003d6c0db3b047323f51c86))
* **rfc-0037:** implement ProviderHandle unified facade for CLI commands ([8864cd1](https://github.com/loonghao/vx/commit/8864cd16fbf3bced1770166d84df5515311c415c))
* wire provider dynamic deps and fix install routing ([d60c8b9](https://github.com/loonghao/vx/commit/d60c8b97182a2f3445703a10b52f12ad472f8cfc))


### Bug Fixes

* **7zip:** fix executable name and system_paths to point to binary file ([845419a](https://github.com/loonghao/vx/commit/845419a5d5884bbe00fbb3c68bc880e2289a56e3))
* add is_version_installable and prepare_execution for bundled runtimes ([daf6a66](https://github.com/loonghao/vx/commit/daf6a66046d6a52e014fc0daca8082d46ecceebe))
* add recursive search for bundled executables and remove wrong fallback ([5db6505](https://github.com/loonghao/vx/commit/5db65050952402709cc3c01e6ba533fe35a8a5b7))
* add workspace-hack deps and remove invalid CI cache parameter ([fed823d](https://github.com/loonghao/vx/commit/fed823de6ca6e8eb4105de7820e85160b2fa8b67))
* address provider CI regressions ([b0d6658](https://github.com/loonghao/vx/commit/b0d6658bb96d9bb95b5887d13e6669312df278e4))
* **ai:** fix skills format and install all 5 skills on setup ([83bf579](https://github.com/loonghao/vx/commit/83bf57939ac92774106abbe680daa9fe78931c9c))
* align starlark mock signatures with stdlib and fix provider tests ([8a8be08](https://github.com/loonghao/vx/commit/8a8be084061faaf1b01ba487ab82c7352d505454))
* **cache:** skip NeedsInstall results in resolution cache; extend TTL to 24h ([0cd8c36](https://github.com/loonghao/vx/commit/0cd8c364445189dc8d38cd8dde4ae334f91978ba))
* **ci:** add sccache setup to all CI workflows ([67237a6](https://github.com/loonghao/vx/commit/67237a685b901fd80c8ebbd99e88897a30fa89cc))
* **ci:** add sccache setup to benchmark workflow ([dbaefc8](https://github.com/loonghao/vx/commit/dbaefc8a279c38df310b2da41629dc7ac0d776b3))
* **ci:** fix discovery parser and CI skip list for Linux/macOS failures ([5d4b834](https://github.com/loonghao/vx/commit/5d4b8348bcd83512e011d8ee9aa9e738ad04f088))
* **ci:** improve sccache path handling on Windows ([7412df4](https://github.com/loonghao/vx/commit/7412df4895b16b9866e83a11c60e92ed1a410623))
* **ci:** increase Windows timeout to prevent CI failures ([313b008](https://github.com/loonghao/vx/commit/313b008dd616b9b18c108a7e210c2c3013679327))
* **ci:** install sccache in quick-test job ([4e32e1a](https://github.com/loonghao/vx/commit/4e32e1acfcd03cdbc499f413182975903f0d5a33))
* **ci:** resolve required check name conflict blocking PR merges ([8f92a54](https://github.com/loonghao/vx/commit/8f92a54e82ee4772364efb40e82a464d667d1def))
* **ci:** skip wix and xmake in CI tests ([bccc34a](https://github.com/loonghao/vx/commit/bccc34a1c0c2f150570392699b5906bf7a97866b))
* **ci:** use system cargo directly instead of vx cargo ([0b45f66](https://github.com/loonghao/vx/commit/0b45f6665845c2f3356c867dbe004790d15e7dbf))
* ensure rust targets are installed in CI ([9bffec3](https://github.com/loonghao/vx/commit/9bffec3aa24b600c7aa44eab403e53dda45713d9))
* exclude vx-star-metadata from cargo-hakari workspace-hack ([2a8bb5f](https://github.com/loonghao/vx/commit/2a8bb5ff6ffb995046670095da0b8b0f3d332733))
* fix PSReadLine cursor positioning issue in PowerShell prompt ([e802fee](https://github.com/loonghao/vx/commit/e802fee8e36ffcdbd774179c03153b8464dbea44))
* fix system_install providers and starlark test assertions (round 5) ([772edbd](https://github.com/loonghao/vx/commit/772edbd39371c01b279c46b8d99e6fbf2db6ab45))
* flatten InstallLayout JSON so manifest_runtime can read strip_prefix ([ab4fea7](https://github.com/loonghao/vx/commit/ab4fea7bf54048a00035de3a4630c79c87d152dc))
* hadolint asset name separator and uvx bundled_with support ([65a2fad](https://github.com/loonghao/vx/commit/65a2fad4c80b7feb0ac7572c7ae6c1d072824059))
* handle VX_VERSION=latest in install scripts ([5351963](https://github.com/loonghao/vx/commit/5351963159e0abee0a93498d694778dfc8ce9e7e))
* improve installer fallback and mirror release support ([a56b239](https://github.com/loonghao/vx/commit/a56b239c8b00e1df09ed8c0568c85ac022a7685e))
* inject parent runtime env for bundled runtimes (npm/node PATH issue) ([0b3de77](https://github.com/loonghao/vx/commit/0b3de7724f670c8e37f4f09ea0057246772ad89e))
* inject parent runtime PATH for bundled runtimes via spec env_config ([06ca8aa](https://github.com/loonghao/vx/commit/06ca8aa8a75d15a6e78689ff3daac36559bceec1))
* **just:** correct version_pattern to match 'just X.Y' output ([942e6a3](https://github.com/loonghao/vx/commit/942e6a32a53fd9e8dcb615053fa2a5163a9049bc))
* **lint:** resolve provider.star lint issues ([aaa95a3](https://github.com/loonghao/vx/commit/aaa95a31ad00e07c03059afc7ef3c8a628d8a88e))
* **manifest-runtime:** override resolve_version to return 'system' for system tools ([0e0c101](https://github.com/loonghao/vx/commit/0e0c101b9598502f76e685ab5a7b5eefb065957c))
* prevent bundled runtime executable misresolution (npm-&gt;node) ([12d5d50](https://github.com/loonghao/vx/commit/12d5d50cee8ab95976b84d6e01ef047d58c2a0f5))
* propagate locked version to bundled runtime dependencies ([1fa037f](https://github.com/loonghao/vx/commit/1fa037faa473e077cec32eb3247ac8b4e72fd5e6))
* **providers:** resolve CI download URL failures ([7391695](https://github.com/loonghao/vx/commit/7391695b724fae5c971353e1cdc29e62918b89fa))
* remove BOM from all provider.star files and improve star syntax checker ([dbafa40](https://github.com/loonghao/vx/commit/dbafa40eee01a867e35011bc79ed3d433e16efc0))
* remove unused loads and fix lint issues in provider.star files ([dc83b6f](https://github.com/loonghao/vx/commit/dc83b6f138d76f22d924b8ce87fa54a06ef92208))
* repair provider test resolution and platform gating ([8dc3aa5](https://github.com/loonghao/vx/commit/8dc3aa55c40dcbcdc2d1426cd0f545603f0b7587))
* replace all ctx dict access with struct attribute access in provider star files ([f1e93f1](https://github.com/loonghao/vx/commit/f1e93f150e2bb486abaccdd6942f0a4533908d56))
* replace all ctx.http.get_json with fetch_json_versions descriptors in provider.star files ([7bcdd33](https://github.com/loonghao/vx/commit/7bcdd333b81a1e2cb5bdbb5eb5816984488f38c0))
* resolve .cmd executables for bundled runtimes on Windows ([0442c91](https://github.com/loonghao/vx/commit/0442c91ff6f7704db4f65a3ab568ba9a256586a6))
* resolve CI errors ([de08ab2](https://github.com/loonghao/vx/commit/de08ab29ac0a7e9a0c04350722ad464ec9b997d0))
* resolve CI errors ([90a1a7c](https://github.com/loonghao/vx/commit/90a1a7cac4ede8305ff154f2f5cb65c236e47ff5))
* resolve CI failures for imagemagick, ffmpeg, rez, bash, make, nasm ([d900aae](https://github.com/loonghao/vx/commit/d900aaed962ce47705dd419d620815d725135834))
* resolve CI failures for yq, wix, xmake, vcpkg providers ([afee387](https://github.com/loonghao/vx/commit/afee38790239bd65ec19e9ebdc1c4781974ffc5b))
* resolve compiler errors in test files ([fe12f86](https://github.com/loonghao/vx/commit/fe12f8697ff085080a00dcc847f434e7d73263ee))
* resolve Linux CI failures for ffplay/ffprobe/gofmt/lld/xmake/yq ([f379ab2](https://github.com/loonghao/vx/commit/f379ab25b746ba94a17051cee6abb7ba9a2c61f0))
* resolve macOS CI failures for ffmpeg and imagemagick ([10949b0](https://github.com/loonghao/vx/commit/10949b0590b8716c16c3ae544a0992c2ebdb2e9e))
* **runtime:** check vx store first in ManifestDrivenRuntime.is_installed() and installed_versions() ([fe803e3](https://github.com/loonghao/vx/commit/fe803e370376fef347c1acd10608e15819152bf7))
* stabilize test suite and version constraint parsing ([a3ae77a](https://github.com/loonghao/vx/commit/a3ae77af823e28a49f265afd063000f0647be2a2))
* **starlark:** lower provider loading log level from info to debug ([96aed89](https://github.com/loonghao/vx/commit/96aed893d275aac78d693f1a110efc8d6d5d971e))
* **starlark:** register all 14 stdlib modules in loader ([583b16b](https://github.com/loonghao/vx/commit/583b16b892104bc9059cc521eeb8796baa6dc873))
* temp_dir unbound variable in install.sh and uv strip_prefix ([bf9cef4](https://github.com/loonghao/vx/commit/bf9cef4410700f1134b5edd4e34c70c9eb55097a))
* **tests:** rewrite all provider runtime_tests to use create_provider() API ([a0edcfc](https://github.com/loonghao/vx/commit/a0edcfcf181f65372b47c7403839f480a85469db))
* **ui:** show Installing feedback during auto-install to avoid perceived hang ([45fbd39](https://github.com/loonghao/vx/commit/45fbd394f390c6e23f9b39209cca22a914eec4fe))
* unblock remaining CI regressions ([c2c7b91](https://github.com/loonghao/vx/commit/c2c7b91b245017939755249b683288e9d0c41b51))
* use bin/bash.exe for git-bash instead of git-bash.exe --attach ([7e72af2](https://github.com/loonghao/vx/commit/7e72af28c8cda9d860729a1adc5e365271aca6ee))
* use child version for bundled proxy runtime installation ([02af11c](https://github.com/loonghao/vx/commit/02af11cc35f29d1b1021f07577311d7ff2ff5218))
* use struct attribute access ctx.platform.os instead of dict access ctx[platform][os] in stdlib star files ([ea14d6c](https://github.com/loonghao/vx/commit/ea14d6c3640c35d07f8ddb6f1d20b834726881a4))
* **versions:** case-insensitive Ecosystem deserialization for vx.lock compatibility ([bc33606](https://github.com/loonghao/vx/commit/bc33606fa8db60af5fac771b830f27e9a8642f97))
* **windows:** resolve OS error 193 when executing bundled runtimes (npm/npx) ([f7329c9](https://github.com/loonghao/vx/commit/f7329c9d4f4f8bea2b403a392359146b40e5fcc7))


### Code Refactoring

* **cli:** update commands and test utilities for runtime refactoring ([d2640d8](https://github.com/loonghao/vx/commit/d2640d8f22436f37417efcce1373314a2ca3ae31))
* **env,version-fetcher:** eliminate platform/version utils duplication ([1a1e4d4](https://github.com/loonghao/vx/commit/1a1e4d4b73b9cdccaabfaef62cdb8ef0aebebbc9))
* extract vx-star-metadata crate and eliminate Box::leak usage ([153d77a](https://github.com/loonghao/vx/commit/153d77acb7bd95d865548473a1a17f57c2775220))
* optimize provider.star files using stdlib templates ([71126bc](https://github.com/loonghao/vx/commit/71126bc5322565185b2557c18eac0800251cc894))
* **providers:** replace all hand-written permissions dicts with stdlib helpers ([a3d66b1](https://github.com/loonghao/vx/commit/a3d66b1e441a45a6c955120b500096ca8f4a14b1))
* replace bare .unwrap() with descriptive .expect() in production code ([eb151eb](https://github.com/loonghao/vx/commit/eb151eb3ee8acce69abd1b0fe085fc01545811ca))
* **resolver:** integrate ResolutionCache into execution pipeline ([4b9c0ca](https://github.com/loonghao/vx/commit/4b9c0ca532e70e61b380a025d7c66aa79143f2bf))
* **runtime-core:** remove dead Runtime trait and provider machinery ([56dbf1b](https://github.com/loonghao/vx/commit/56dbf1b6168591261a68c3e46bcb14f10fae2d3f))
* **runtime:** split runtime.rs into module and add ISP sub-traits ([3ebd68f](https://github.com/loonghao/vx/commit/3ebd68fb20469a4789c40cbcd9ae36212a568903))
* simplify all providers to PROVIDER_STAR only, remove redundant create_provider and star_metadata ([b232a8f](https://github.com/loonghao/vx/commit/b232a8fe7a76fbf62b9b39fa7e7e143a8ebe80c0))
* split tests to tests/ dir, extract bridge/builder modules, remove metadata indirection, fix clippy warnings ([84f6454](https://github.com/loonghao/vx/commit/84f645478bf1ca4c1abe9ef9667b82e2c56b4025))


### Documentation

* add Starlark Providers advanced guide (bilingual) ([c649320](https://github.com/loonghao/vx/commit/c6493208d74687e4e29c6c0be097a666a67a1009))
* improve agent knowledge - update provider count to 78, enhance AGENTS.md, sync skills ([#687](https://github.com/loonghao/vx/issues/687)) ([7185a30](https://github.com/loonghao/vx/commit/7185a30d42d0f6fc5ea1c9685387363e1b6aba88))
* improve agent knowledge - update provider.star docs, fix tool counts, add creating-provider guide ([dcff435](https://github.com/loonghao/vx/commit/dcff43556fd7258478394af8c5dd3d623e2d9f1b))
* update rules to reflect provider.star migration ([3813f82](https://github.com/loonghao/vx/commit/3813f82a7dc7fd33257cafbc6eb1ea6635634ef1))

## [0.8.4](https://github.com/loonghao/vx/compare/v0.8.3...v0.8.4) (2026-02-20)


### Features

* **starlark:** add github.star stdlib + jj provider.star migration ([2666e9c](https://github.com/loonghao/vx/commit/2666e9c35d48962caa4943614fe422d7e7a886b3))
* **starlark:** complete provider.star migration and fix stdlib ctx access ([eb9c882](https://github.com/loonghao/vx/commit/eb9c8821c16458edfcbe61d2650485abf798cef9))
* **vx-starlark:** implement Phase 2 Starlark execution engine ([46ace14](https://github.com/loonghao/vx/commit/46ace140ae17d909b12ace6b1f1df51a180cf2dd))
* **vx-starlark:** Phase 2 - integrate starlark-rust execution engine ([61b2fd3](https://github.com/loonghao/vx/commit/61b2fd3c167aa851c84c12ffe3ce48efa179591f))


### Bug Fixes

* fix workspace-hack hakari section markers and regenerate dependencies ([f41c6c6](https://github.com/loonghao/vx/commit/f41c6c694d102f6a39492987d3de16190dc9093a))
* **justfile:** fix test-pkgs recipe to not duplicate -p flag ([cbfc402](https://github.com/loonghao/vx/commit/cbfc402a896bbb0e40ee6352fc3383a32d4bed24))
* **vx-provider-jj:** strip v prefix from version tags to prevent double-v in download URL ([e0b13eb](https://github.com/loonghao/vx/commit/e0b13eb8f7c9995682eb3024d798c2fed8ef2288))
* **where:** use executable_name() instead of runtime name for exe lookup ([610310f](https://github.com/loonghao/vx/commit/610310fbba23d21940214cc0c820730fcc57882c))


### Code Refactoring

* **build:** remove legacy provider.toml support, simplify build.rs and registry.rs ([b0dd935](https://github.com/loonghao/vx/commit/b0dd935619a4454c65aec137cef79a202d9bc44b))
* **vx-starlark:** replace path-based cache with content-hash incremental analysis cache ([23c9918](https://github.com/loonghao/vx/commit/23c9918382c93a9081ffeb8b5dbbcfaf52a2e19e))


### Documentation

* **rfc:** add RFC 0036 - Starlark Provider Support ([9f29ebf](https://github.com/loonghao/vx/commit/9f29ebf34c29aaed9c1c94918ae3a4a40f198ff3))
* **rfc:** update RFC 0036 v0.3 - add Buck2 typed provider_field, load() module system, incremental analysis cache, declarative actions ([58ac77d](https://github.com/loonghao/vx/commit/58ac77d60cabacfdd451b926ae83548fc2d4a5a0))

## [0.8.3](https://github.com/loonghao/vx/compare/v0.8.2...v0.8.3) (2026-02-19)


### Features

* **ai:** implement RFC 0035 AI integration optimization ([7ab320c](https://github.com/loonghao/vx/commit/7ab320c0898678806829dcdc81673abab0453ce7))


### Bug Fixes

* change internal rustup/toolchain debug logs to trace level ([bdbbe93](https://github.com/loonghao/vx/commit/bdbbe938cdc463e7f6ef15f9f321077625a8305c))
* **ci:** remove max-versions-to-keep from winget-releaser ([21fa5f7](https://github.com/loonghao/vx/commit/21fa5f768cedcc332238c90fc8db5633abc798c4))
* **cli:** add --toon shortcut flag for TOON output format ([0a5cc91](https://github.com/loonghao/vx/commit/0a5cc9118fde4cc06dc6cd8cc428138cc49f0f8b))
* **tests:** add missing OutputFormat argument to handle_list calls ([0756270](https://github.com/loonghao/vx/commit/07562706ae789b2483a2f8a4023f34f3b6e3e824))


### Performance Improvements

* optimize test and build configuration ([7228164](https://github.com/loonghao/vx/commit/7228164be85faaac1bc9585ab9e0a5b1d7c73544))


### Documentation

* add llms.txt and llms-full.txt following llmstxt.org protocol ([6499b09](https://github.com/loonghao/vx/commit/6499b096a8eb0ad733ebe4b9af352ef32edd5c60))
* add pre-commit hooks documentation (EN/ZH) and update contributing guides ([770a4f5](https://github.com/loonghao/vx/commit/770a4f535d89f905cd17a3913db349be98d506dc))
* **cargo:** add fast build optimizations inspired by Bevy ([09c2311](https://github.com/loonghao/vx/commit/09c2311e8c06e26146c4b8d261a85e34fa196488))
* sync zh contributing.md and add zh fixes docs ([ef91ea0](https://github.com/loonghao/vx/commit/ef91ea050d611b1f3ce6a58f413182e6b4367e38))

## [0.8.2](https://github.com/loonghao/vx/compare/v0.8.1...v0.8.2) (2026-02-18)


### Features

* **build:** add rust-lld linker for faster builds (RFC 0032 Phase 1) ([60d7a17](https://github.com/loonghao/vx/commit/60d7a171600a9b22bc455c63fcf7e7b87e79b198))
* **build:** add vx-runtime-core and vx-runtime-archive to workspace dependencies ([c0e13bd](https://github.com/loonghao/vx/commit/c0e13bdddb0d38e2d83bd3ef4f6c1b6971ae1844))
* **build:** create vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([5c2a118](https://github.com/loonghao/vx/commit/5c2a118f06c8471994f7012ff8423ba74d9c9589))
* **build:** integrate vx-runtime-core and vx-runtime-archive (RFC 0032 Phase 2) ([cb49cb2](https://github.com/loonghao/vx/commit/cb49cb2d2fb0f609dcaddcc8230862cbed5a9788))


### Bug Fixes

* **ci:** remove lld linker on macOS due to compatibility issues ([1f283d7](https://github.com/loonghao/vx/commit/1f283d76140a64b910a94f0196a5b897462934fd))
* **macos:** make sevenz-rust optional to fix macOS build ([5042c58](https://github.com/loonghao/vx/commit/5042c582152491d3c8ac2844de7ed9edc56db988))


### Performance Improvements

* implement cargo-hakari workspace-hack + runtime/config refactoring ([aa28ce3](https://github.com/loonghao/vx/commit/aa28ce330be7605cc107a3159654327ddc5c6415))


### Documentation

* **rfc-0032:** update Plan D (hakari implemented), Plan E/F tracking status ([ad96225](https://github.com/loonghao/vx/commit/ad96225c7aaba5ac07f6979458468239dbf928a7))
* **rfc:** update Phase 2 progress in RFC 0032 ([c70a461](https://github.com/loonghao/vx/commit/c70a46127563370e3373f783144548c56458e89e))
* **rfc:** update Phase 2 status in RFC 0032 ([29e25c7](https://github.com/loonghao/vx/commit/29e25c76522013523eeda1a409de702cf1d31d3a))

## [0.8.1](https://github.com/loonghao/vx/compare/v0.8.0...v0.8.1) (2026-02-17)


### Features

* add vcpkg provider for C++ dependency management ([fc6f317](https://github.com/loonghao/vx/commit/fc6f31739bee912800ec7e73ebb2c336dc786b9b))
* use RuntimeContext for install_options instead of env vars (Phase 1) ([96c98a2](https://github.com/loonghao/vx/commit/96c98a2958412defb173f393e0143e206ba96bec))


### Bug Fixes

* auto-disable Spectre mitigation in MSBuild bridge when libs are missing ([a4e1623](https://github.com/loonghao/vx/commit/a4e16232dba6c32e4b6a0f230bb34156a9bd8007))
* clean extraction markers before re-installing missing MSVC components ([96cc05a](https://github.com/loonghao/vx/commit/96cc05a830f56677df52af39b4c5969bee30a138))
* collapse nested if statements to satisfy clippy ([0338d30](https://github.com/loonghao/vx/commit/0338d3059bb232ecbe2f333ab2a3833774ab7493))
* ensure MSVC Spectre component integrity check for already-installed companion tools ([3da828c](https://github.com/loonghao/vx/commit/3da828cdaf68862b2f413a16f097fdefd6875233))
* prevent repeated MSVC component re-installation when Spectre libs unavailable ([9c93a0c](https://github.com/loonghao/vx/commit/9c93a0c2d8b6d25cee51fe588974147bdef67c0a))
* resolve clippy warnings and test assertion ([484f35c](https://github.com/loonghao/vx/commit/484f35c19d946bf1931bd64b6999b3921980f6df))
* switch macOS FFmpeg download source from evermeet.cx to osxexperts.net ([1fbd926](https://github.com/loonghao/vx/commit/1fbd926e8dfbd6316c742d1778def3ccdcec0e2a))

## [0.8.0](https://github.com/loonghao/vx/compare/v0.7.13...v0.8.0) (2026-02-15)


### ⚠ BREAKING CHANGES

* migrate providers, add bridge system, fix Windows env injection
* migrate providers, add bridge system, fix Windows env injection

### Bug Fixes

* **docs:** escape angle brackets in RFC 0033 headings ([2c76f06](https://github.com/loonghao/vx/commit/2c76f06b3104ea316a01d25f829f90ee0c118f30))
* inject companion tools environment variables for vx.toml co-configured tools ([7fe198d](https://github.com/loonghao/vx/commit/7fe198d4b8e7e07008caf631b0b77df8abfaa422)), closes [#582](https://github.com/loonghao/vx/issues/582)
* relax MSVC prepare_environment validation to only check cl.exe existence ([9ca374d](https://github.com/loonghao/vx/commit/9ca374d119a05a14bdff9cc199584dee7bb6c866))
* remove unstable as_str() usage in environment.rs ([18d34e2](https://github.com/loonghao/vx/commit/18d34e26c2ee48072d06af8efc4ffcc20a90dc80))
* **test:** add missing package_alias field in manifest_registry_tests ([df2bbec](https://github.com/loonghao/vx/commit/df2bbeca7c89d36ecb01afe9d4618124ae409495))
* **test:** add missing package_alias field in ProviderMeta test ([10b7d1e](https://github.com/loonghao/vx/commit/10b7d1e7b58c7a66fc7959e904ec0967e4153f72))
* **test:** skip package_alias providers in CI tests ([46b9676](https://github.com/loonghao/vx/commit/46b967622af540ba62f13b0f1f7189bebf167181))


### Code Refactoring

* migrate providers, add bridge system, fix Windows env injection ([b3decdb](https://github.com/loonghao/vx/commit/b3decdbc93649e5214dc434e341e073a6c445e13))
* migrate providers, add bridge system, fix Windows env injection ([a2b9c0c](https://github.com/loonghao/vx/commit/a2b9c0c732f5910eb9aea148eb2b660ab7d7bf8b))


### Documentation

* add Package Alias documentation (EN + ZH) ([2d2be12](https://github.com/loonghao/vx/commit/2d2be1233d741f93d54081b713ca0870f3d87771))
* add RFC 0032 and RFC 0033, add opencode skills ([370eb15](https://github.com/loonghao/vx/commit/370eb158d41858328ca5c5ce77b91f037eef1a1f))
* clarify companion tool injection applies to all tools, not just Node.js ([9ec51f1](https://github.com/loonghao/vx/commit/9ec51f135b5fc526b07930ffe6744986bab9f012))
* **skill:** add new vx capabilities to SKILL.md ([5d4df51](https://github.com/loonghao/vx/commit/5d4df51474f1fcbfff100df7c63e0ab7194f2c18))

## [0.7.13](https://github.com/loonghao/vx/compare/v0.7.12...v0.7.13) (2026-02-14)


### Features

* add hadolint (Dockerfile linter) provider ([c9036c6](https://github.com/loonghao/vx/commit/c9036c6b4d521c7de00fa7113f7b937f889a1b9f))


### Bug Fixes

* add fzf to manifest registry and use source-built vx in docker CI ([d35c0a5](https://github.com/loonghao/vx/commit/d35c0a5d24e547a961dc2d36c30e4255b1f69588))
* hadolint provider executable layout and static registry ([70ff5d6](https://github.com/loonghao/vx/commit/70ff5d605d1590bbe4acc40684058409e5170410))
* lower rust-version to 1.93.0 for CI compatibility ([5c71b09](https://github.com/loonghao/vx/commit/5c71b094f5eb1d3570fa00230b4ddbf5eb495d88))
* MSVC env vars conflict with node-gyp ([#573](https://github.com/loonghao/vx/issues/573)) ([ba511fb](https://github.com/loonghao/vx/commit/ba511fbae4b3a28cf90783fbc03a80e27bfab94c))
* platform-aware executable fallback in ResolvedLayout ([3a79a16](https://github.com/loonghao/vx/commit/3a79a161af6a0dfb1d4b90ade1a9be261d22a8c7))
* reinstall runtime when executable missing from cached install ([9f894aa](https://github.com/loonghao/vx/commit/9f894aaca1969fff6870bc2098e037be19faf693))

## [0.7.12](https://github.com/loonghao/vx/compare/v0.7.11...v0.7.12) (2026-02-14)


### Bug Fixes

* add executable permission to shell scripts and update RFC status ([97ddb10](https://github.com/loonghao/vx/commit/97ddb100845e655446cb56343386ecbfe00e4ada))

## [0.7.11](https://github.com/loonghao/vx/compare/v0.7.10...v0.7.11) (2026-02-14)


### Features

* add prek provider and fix clippy for Rust 1.93 ([5d56ed5](https://github.com/loonghao/vx/commit/5d56ed5ea3d300e4baac139dd8c57d2e6e9b5ce9))
* add version fallback on installation verification failure ([866cf58](https://github.com/loonghao/vx/commit/866cf58e0327cf8c8273448021a3ef4f21f9b2dc))
* platform-aware tool filtering in vx.toml ([ed0afce](https://github.com/loonghao/vx/commit/ed0afce5f1b302c099eb22380dd86ddf56b05ec5))


### Bug Fixes

* add download retry, fix clippy lint, enhance pre-commit hooks ([e1b67fd](https://github.com/loonghao/vx/commit/e1b67fde0824487ce4d373bc9ff26b05aa0d37c0))
* add missing platform subdirectory in executable path resolution ([2810fa7](https://github.com/loonghao/vx/commit/2810fa7309ef4c38e2e39c4b999f782987e460bb))
* **ci:** ensure vx for security audit ([ed1a3fc](https://github.com/loonghao/vx/commit/ed1a3fc1fe7d360dce4c4b7a26ced88ac39eb5b3))
* **ci:** preserve sccache env in isolation ([e46fa69](https://github.com/loonghao/vx/commit/e46fa696bb509a8528471301bfa280086848bcc5))
* cross-platform lock file URL mismatch & enhance pre-commit hooks ([3e10939](https://github.com/loonghao/vx/commit/3e10939a36ee2bbd7ce23caff49a8d98f5eee5c3))
* disable axoupdater and turbo-cdn for aarch64-pc-windows-msvc ([3a442b6](https://github.com/loonghao/vx/commit/3a442b65de233827e10641985b1652ce81e49ac8))
* improve install.sh version detection and fallback logic ([f738047](https://github.com/loonghao/vx/commit/f7380473f79ef64b90e83d77ba4a0f8db340dd7e))
* **msvc:** selective env injection to avoid node-gyp conflicts ([#573](https://github.com/loonghao/vx/issues/573)) ([852f811](https://github.com/loonghao/vx/commit/852f8113df92324d571d68f23cb330a581c9d58d))
* prek archive strip and imagemagick platform directory issues ([e13b5dc](https://github.com/loonghao/vx/commit/e13b5dc9abf4bcbaf16c2f6879aec6a201d58cf6))
* **prek:** add executable_layout to trigger strip_prefix during install ([f22c4ca](https://github.com/loonghao/vx/commit/f22c4ca202f618aa008f7a861873b99b256bd504))
* **prek:** remove pre-commit alias causing non-deterministic registry conflict ([f2d4349](https://github.com/loonghao/vx/commit/f2d43492ee899b52ffc0e430e8c69982ba8c39b2))
* remove rustup from bundled tool fallback and run cargo fmt ([f97004b](https://github.com/loonghao/vx/commit/f97004b1f92de207244b591af3463cfadadafc96))
* skip releases without assets when rate limited ([e6f4e15](https://github.com/loonghao/vx/commit/e6f4e1571d131fa8fe0c84a8219edeeaac5c0cb6))
* upgrade deps for aarch64-windows cross-compile and add msvc-kit lzma workaround ([a516acd](https://github.com/loonghao/vx/commit/a516acd204f0038e6b3f2caf1f76ed4a030f3f79))
* upgrade msvc-kit to 0.2.9 and remove local patch ([4f550c7](https://github.com/loonghao/vx/commit/4f550c7f81f4da9ff0e498189a373b27af5981f1))
* use aws-lc-rs instead of ring for aarch64-pc-windows-msvc cross-compilation ([84b5050](https://github.com/loonghao/vx/commit/84b5050ed559aec889c459dc7170a933685e7572))
* use portable grep/sed for release tag extraction ([9bc9adf](https://github.com/loonghao/vx/commit/9bc9adf5df3669a3797cd8c3a9348823f7d91dc0))
* **vx-paths:** mark extern clonefile as unsafe ([81bfb90](https://github.com/loonghao/vx/commit/81bfb9034f4bb9cde193d296d949aa4c663a0853))
* wrap env::set_var/remove_var in unsafe blocks for Rust 1.83+ compatibility ([b948b5f](https://github.com/loonghao/vx/commit/b948b5f9fab880c121cd0974d0bdc29ebc71cd00))


### Performance Improvements

* **ci:** add sccache and fix fmt ordering ([e0bd6e0](https://github.com/loonghao/vx/commit/e0bd6e0d16c8a61cd7749a3463e5549c151cf197))

## [0.7.10](https://github.com/loonghao/vx/compare/v0.7.9...v0.7.10) (2026-02-12)


### Features

* add .NET/C# project analyzer and x-cmd provider ([dc166de](https://github.com/loonghao/vx/commit/dc166de2b7c1f8fb65567a26e81556a1303a0bc5))
* add actrun provider and .pkg format support ([41fb59b](https://github.com/loonghao/vx/commit/41fb59b3ade2cbc911b83726ac6a9d3e075ee6b4))
* add region-aware mirror support for all major providers ([cf5e349](https://github.com/loonghao/vx/commit/cf5e3490218d6034265523472a7fed7b31cb3265))
* **cli:** add skills provider and ai setup command ([b044d0f](https://github.com/loonghao/vx/commit/b044d0f5c8eff1ef1c1f4662241086e4e7b93bee))
* **installer:** add .pkg and .msi archive format support with executable flattening ([31ce917](https://github.com/loonghao/vx/commit/31ce917b9bdd1c1cc082af71ca724d4e54a2d7ff))
* unified structured output (RFC 0031) and Tier 1 provider expansion (RFC 0030) ([f213725](https://github.com/loonghao/vx/commit/f2137255f45b181a5da7a028e0ca0d05af2f0247))


### Bug Fixes

* **ci:** remove skills provider crate entirely ([1747125](https://github.com/loonghao/vx/commit/1747125f5fb35f4ea4444c8210f8a3271f850d5f))
* filter empty-assets GitHub releases and add aarch64-windows target ([bf042b5](https://github.com/loonghao/vx/commit/bf042b508d993c9445761f950337853c11cf4968))
* rename global --format to --output-format to avoid conflict with Dev command, run cargo fmt ([1f45915](https://github.com/loonghao/vx/commit/1f45915ce35406d1039e26689b7aa9e6aa0343c5))
* resolve CDN test race conditions and CI detection issues ([c280222](https://github.com/loonghao/vx/commit/c280222ca089c60b951f766bdc7a5901133cc0e9))
* resolve region_tests race conditions and clippy warnings ([8ab846a](https://github.com/loonghao/vx/commit/8ab846a16a52af4d04ff2797e7fc3b19e36d8e9d))
* resolve runtime test failures for bat/fd/ripgrep/fzf providers ([efed8bc](https://github.com/loonghao/vx/commit/efed8bcfcb8dfc50c306ce3eb5b3e4ffba8d83fc))
* resolve unused import and fmt issues ([e21c2ee](https://github.com/loonghao/vx/commit/e21c2ee70cf1925fd1512656d88ea57eb990c868))
* simplify boolean expression in cdn_tests.rs ([a052837](https://github.com/loonghao/vx/commit/a052837737f25d4977a97acd3a45a384a99cb5a7))
* x-cmd install() override and C# deep project detection ([ca379e8](https://github.com/loonghao/vx/commit/ca379e8439d8e1dba59f2218d60c98ccbcd90b44))


### Code Refactoring

* **cli:** remove skills from runtime registry, use vx npx instead ([743e3e8](https://github.com/loonghao/vx/commit/743e3e868e08a1c8400df15c4d5e6dbab7c104a2))


### Documentation

* add RFC 0030 provider expansion plan ([5dd4866](https://github.com/loonghao/vx/commit/5dd4866e28f27930053be02903c6253e6bbad718))
* add RFC 0031 unified structured output (--json global support + TOON) ([e6b46af](https://github.com/loonghao/vx/commit/e6b46afc281dc2b4d75499ad195596f5e62b5e49))

## [0.7.9](https://github.com/loonghao/vx/compare/v0.7.8...v0.7.9) (2026-02-10)


### Bug Fixes

* restore WinGet auto-publish support for cargo-dist releases ([6a00199](https://github.com/loonghao/vx/commit/6a00199635a0f7a8f82d92fb9ec360f194d66194))

## [0.7.8](https://github.com/loonghao/vx/compare/v0.7.7...v0.7.8) (2026-02-10)


### Bug Fixes

* add versioned artifact copies in release workflow and installer script fallback ([ae2d375](https://github.com/loonghao/vx/commit/ae2d3759614b6410a82e16aeca6f33955fb59e22))
* apply cargo fmt formatting ([428b1af](https://github.com/loonghao/vx/commit/428b1aff4f6e1a8ef0bb6a36cf820200f755d0a8))
* cargo fmt formatting issues ([278030d](https://github.com/loonghao/vx/commit/278030deafffe66c35a0dbdebf5d216230526c8a))
* create legacy compatibility release (vx-v{ver}) for old binary self-update ([cadfabb](https://github.com/loonghao/vx/commit/cadfabbb072e7f86dc50b1efe82901561a40b182))
* optimize self-update display and support cargo-dist versioned artifacts ([aad90ea](https://github.com/loonghao/vx/commit/aad90eaa424fd08692d5228ce301b006db59d228))

## [0.7.7](https://github.com/loonghao/vx/compare/v0.7.6...v0.7.7) (2026-02-08)


### Bug Fixes

* skip CI/CodeQL/Benchmark workflows on release commits ([1ae348a](https://github.com/loonghao/vx/commit/1ae348a79e96d82b22bf562727239c462cf4068f))

## [0.7.6](https://github.com/loonghao/vx/compare/v0.7.5...v0.7.6) (2026-02-08)


### Features

* add dagu workflow engine provider ([8f4ad20](https://github.com/loonghao/vx/commit/8f4ad203c5e8cd78f21064850b9ed37e898cb76b))


### Bug Fixes

* increase macOS benchmark thresholds for setup dry-run ([9bada95](https://github.com/loonghao/vx/commit/9bada954d760423d80148166f97b5f9aa86c7f47))

## [0.7.5](https://github.com/loonghao/vx/compare/v0.7.4...v0.7.5) (2026-02-08)


### Features

* support runtime::executable syntax and detection system_paths ([062e51d](https://github.com/loonghao/vx/commit/062e51d5dc49cb550dcc8dbbc37fa8a37e775b73))


### Bug Fixes

* msvc provider manifest parsing and detection paths ([e1eef54](https://github.com/loonghao/vx/commit/e1eef54ed3a35ea93a2d9bca1efe76f78dfb66f1))
* use per-layer tracing filter to prevent debug log leaking in normal mode ([23ec8f7](https://github.com/loonghao/vx/commit/23ec8f7f6aecfdd2df583a8f67df79742975e213))

## [0.7.4](https://github.com/loonghao/vx/compare/v0.7.3...v0.7.4) (2026-02-08)


### Features

* add vx-metrics crate with HTML report and metrics CLI command ([4b78068](https://github.com/loonghao/vx/commit/4b780687623f5c31cbc7cb8785edb684693cc647))
* integrate axoupdater for cargo-dist receipt-based self-update ([511eed6](https://github.com/loonghao/vx/commit/511eed6194323fdc8e316317604ca4766edfeecc))


### Bug Fixes

* pass InstallResult.executable_path through pipeline, add tests ([9866965](https://github.com/loonghao/vx/commit/98669654873d6670fa458054af6ad70dd02ee00b))
* redirect all log output to stderr in install scripts ([88a1b68](https://github.com/loonghao/vx/commit/88a1b686a815dac30b93b4a63efb15f8633ae8d4))
* resolve broken pipe and unbound variable errors in install scripts ([0a5fbe8](https://github.com/loonghao/vx/commit/0a5fbe821729448a6b8990d9091050d38f8e6590))
* support multi-tag format fallback for v0.6.x to v0.7.x self-update ([733ca65](https://github.com/loonghao/vx/commit/733ca65e31b49d866acf0632901b0f48cb23a191))
* use forward slashes for Windows binary path in test-action.yml ([daa7147](https://github.com/loonghao/vx/commit/daa714740a238536fbdf883aadfe82007fd69874))


### Performance Improvements

* add executable path caching with bincode serialization ([db2cf38](https://github.com/loonghao/vx/commit/db2cf3856dd29346efb0e907edca8f35d5290cad))
* lazy-load providers on demand instead of all at startup ([469f5ce](https://github.com/loonghao/vx/commit/469f5ceeb7a6cf0bf4b28b296cb248105f9add07))


### Code Refactoring

* migrate inline tests to tests/ directory for plan.rs and ensure.rs ([2cb1524](https://github.com/loonghao/vx/commit/2cb1524536402324d3875db1e8a158fc9bd29cf0))
* simplify action.yml and improve test-action.yml coverage ([e441615](https://github.com/loonghao/vx/commit/e441615cb765ba89335bb76f5a395e667271a757))

## [0.7.3](https://github.com/loonghao/vx/compare/v0.7.2...v0.7.3) (2026-02-07)


### Bug Fixes

* gracefully skip Homebrew publish when HOMEBREW_TAP_TOKEN is not set ([003ac49](https://github.com/loonghao/vx/commit/003ac49724a303a247756e79217c105fb56041c2))
* use HOMEBREW_TAP_GITHUB_TOKEN for Homebrew formula publishing ([56b7306](https://github.com/loonghao/vx/commit/56b730648f22564566cf870eec47b46546bc700f))

## [0.7.2](https://github.com/loonghao/vx/compare/v0.7.1...v0.7.2) (2026-02-07)


### Features

* **resolver:** implement execution pipeline architecture (RFC 0029) ([f968f5c](https://github.com/loonghao/vx/commit/f968f5c1be3dc44b32e8d12036b8ce0710682965))
* RFC 0029 Phase 1-3 + vx info docs (EN/ZH) ([15095a2](https://github.com/loonghao/vx/commit/15095a283462bc121b02a33065bfc83669b65365))


### Bug Fixes

* clippy lint - use struct update syntax for ExecutionConfig ([7d4a3e1](https://github.com/loonghao/vx/commit/7d4a3e1df638b0b1dc94431597edf5fb1920a32c))
* **deps:** update rust crate which to v8 ([90450b4](https://github.com/loonghao/vx/commit/90450b498b02261712331978c557501cdda8022e))
* update integration test to match structured error messages ([c7259bb](https://github.com/loonghao/vx/commit/c7259bb0e8e9fa42ed95bf883b2c00727feb3b32))


### Documentation

* Added docs/cli/info.md, docs/zh/cli/info.md, updated sidebar ([15095a2](https://github.com/loonghao/vx/commit/15095a283462bc121b02a33065bfc83669b65365))

## [0.7.1](https://github.com/loonghao/vx/compare/v0.7.0...v0.7.1) (2026-02-06)


### Bug Fixes

* **ci:** trigger release workflow via workflow_dispatch after release-please creates tag ([63790c1](https://github.com/loonghao/vx/commit/63790c184c65f2a20b4760f50d748c15c446a09e))

## [0.7.0](https://github.com/loonghao/vx/compare/v0.6.31...v0.7.0) (2026-02-06)


### ⚠ BREAKING CHANGES

* **self-update:** None - all changes are backward compatible

### Features

* add binary download support for rust/rustup and improve CI coverage ([3ae9600](https://github.com/loonghao/vx/commit/3ae9600e9345deb7a0cf2287f5cc2c5457e8a669))
* add C++ analyzer and refactor language modules ([b9fdb7a](https://github.com/loonghao/vx/commit/b9fdb7a65dfdff1f2ad27c2c2a06b197f00c0efd))
* add GitHub auth and unified version fetcher ([e2b946d](https://github.com/loonghao/vx/commit/e2b946d32a3a64d38ab3abafdb5a40a82d564faf))
* add GitHub CLI (gh) provider ([e3dd7f4](https://github.com/loonghao/vx/commit/e3dd7f45a1bcaed9611d39afa730b1de512583f0))
* add jsDelivr CDN fallback for GitHub releases API ([43ea611](https://github.com/loonghao/vx/commit/43ea61155e0a9548434b07e2704e8ce3e5e9b174))
* add layered executable path API ([256d508](https://github.com/loonghao/vx/commit/256d5083b9c22e6ec8874f899358ffc74d1ddc2c))
* add meson and make providers, fix git and yasm issues ([1203ae9](https://github.com/loonghao/vx/commit/1203ae93cdf022c8c43b86d7890525852ed1b4a8))
* add package manager install support and improve static binary handling ([7c812bc](https://github.com/loonghao/vx/commit/7c812bc4c088b058f50ce4070a4703f5f11db87b)), closes [#389](https://github.com/loonghao/vx/issues/389)
* add pipx provider and fix rust/rustup issues ([e7b2f99](https://github.com/loonghao/vx/commit/e7b2f998ed7238d8d3458e4e7de09cf4a68cd2a8))
* add pre_run hooks ([b444422](https://github.com/loonghao/vx/commit/b444422b51fce679c279edac5d43b47f2ba7aab8))
* add provider.toml manifests for meson and make ([f914d91](https://github.com/loonghao/vx/commit/f914d91d6a61e7893c9779d6b1f8fc96fc7eabf3))
* add RFCs for platform-aware providers and system tool discovery ([65357b8](https://github.com/loonghao/vx/commit/65357b8b691704c5defb395f3607ad0783aef2ac))
* add Runtime::store_name() method for consistent store path resolution ([682a47f](https://github.com/loonghao/vx/commit/682a47f92b16018137b1362090ab381a6c8f47dd))
* add silent MSI installation for AWS CLI on Windows ([7f5e7af](https://github.com/loonghao/vx/commit/7f5e7af31ff371a198c7971811adabd40af157e0))
* add static linking for Linux/macOS and optimize build speed ([a0b624f](https://github.com/loonghao/vx/commit/a0b624f0756a4f9fde4408485a7c7535d28f795f))
* add system tool providers and AI-native development documentation ([#368](https://github.com/loonghao/vx/issues/368)) ([5cb347f](https://github.com/loonghao/vx/commit/5cb347fffaa1f4538a9e69fb6569a6e8e3395266))
* add version syntax and dependency constraints support ([c06c309](https://github.com/loonghao/vx/commit/c06c3091d71512bb04456fb81c85152d2834eef1))
* add vx-migration crate ([b34a1e0](https://github.com/loonghao/vx/commit/b34a1e03c68353c22597f9e6e360f407e364a391))
* add vx-setup crate for setup pipeline and CI support ([63939b7](https://github.com/loonghao/vx/commit/63939b7de08ae0a663d4ddd70c13834e4d36316c))
* add Windows long path support and fix macOS/Linux installer compatibility ([e077ce8](https://github.com/loonghao/vx/commit/e077ce8217ae734811a35d1319db3256cda073cd))
* adopt cargo-dist for release workflow and fix tag pattern ([c2d55df](https://github.com/loonghao/vx/commit/c2d55dfa50880e4873fe645349576d50a7cfe0ec))
* **ci:** optimize CI pipeline with crate-level change detection ([5c26d8b](https://github.com/loonghao/vx/commit/5c26d8bb128d73e06e066794bd8b5c1ac257c6a6))
* **ci:** simplify release workflow with cargo-dist style ([5210e8b](https://github.com/loonghao/vx/commit/5210e8b0f013f23ff1aaac3f64a338c801bde473))
* **cli:** enhance vx dev with --info option and improved status display ([e732512](https://github.com/loonghao/vx/commit/e732512f69c0c2167a0ca4545f5834d174e04bda))
* **cli:** implement RFC 0013 manifest-driven registration ([5f00f11](https://github.com/loonghao/vx/commit/5f00f117512f92db4e398e08965896be0f4a9658))
* **cli:** implement RFC 0020 Phase 2 - modular command structure ([f844015](https://github.com/loonghao/vx/commit/f844015e35b97ada8d78d04ed4135d4dbcbd3927))
* **cli:** integrate lock file with sync and install commands ([a93ce06](https://github.com/loonghao/vx/commit/a93ce065dda71b6d7ee6ee9ce7793a6717f47451))
* **cli:** support installing multiple tools at once ([00ee20f](https://github.com/loonghao/vx/commit/00ee20fe9a939ebf229a7bdf0f1ca9d78c4ed72b))
* complete architecture improvements (Phase 0-4) ([d0d9fbd](https://github.com/loonghao/vx/commit/d0d9fbd5e8ba119203ef6d06718d449f3ec3ca66))
* **docker:** add tools image with pre-installed uv, ruff, and node ([1a92cd1](https://github.com/loonghao/vx/commit/1a92cd12c952881f4397f29ecc5ad93a0d7d622c))
* **env:** add default inherit_system_vars for all providers ([c4e453a](https://github.com/loonghao/vx/commit/c4e453a870449ba1e9d24eab76745cb7c7ce2e3b))
* **executor:** auto-sync uv dependencies before uv run ([75627cb](https://github.com/loonghao/vx/commit/75627cb809991af1a68cbfa2a22ef6832bc1c5d9))
* expand platform support for multiple architectures and libc variants ([283b995](https://github.com/loonghao/vx/commit/283b995795172d3b8fe8b98f071c960adf0732ae))
* **extension:** complete phase 2 with error handling and 81 tests ([48cfbcd](https://github.com/loonghao/vx/commit/48cfbcdd8fca06fa2dbc622357b4eb7d2d4e44c6))
* **extension:** implement vx extension system ([a23dccb](https://github.com/loonghao/vx/commit/a23dccbfd5d8e00d9eb8abdc6d73dd681bc18656))
* **extension:** phase 3 and 4 ([157fbd7](https://github.com/loonghao/vx/commit/157fbd703b22838a4281517ec79f8b22d8178b67))
* **imagemagick:** add system_deps for platform-specific package managers ([131e558](https://github.com/loonghao/vx/commit/131e5587ec64f14a28f755f805185a6ff68cac2c))
* **imagemagick:** improve error messages and add e2e tests ([5d1ace2](https://github.com/loonghao/vx/commit/5d1ace2faec3d51e1066655411a78220196e6c8c))
* implement global package management with cross-language isolation ([b1e873b](https://github.com/loonghao/vx/commit/b1e873bd951c4d23937bd8886fc12a1ec3356f7f))
* implement RFC 0020 & 0021 - system package manager integration and manifest-driven runtimes ([322f624](https://github.com/loonghao/vx/commit/322f624ab9d9f6a405cc19c03190297834e1b1fd))
* implement version solver ([ab9f99d](https://github.com/loonghao/vx/commit/ab9f99d11ac846b75587d2bd9293a0bed814029b))
* improve download timeout handling for large files ([f4ea792](https://github.com/loonghao/vx/commit/f4ea7921bbcbc1cf50079d98bb66fcc32fd8480e))
* improve network timeout handling and progress reporting ([a730b52](https://github.com/loonghao/vx/commit/a730b52e15cf3f2c4b1101e967b6c3e3c9b26e56))
* **installer:** add .tar.zst (Zstandard) format support ([48761b8](https://github.com/loonghao/vx/commit/48761b88098bd14bf72f3934fd1e23c488ee64a8))
* **installer:** add 7z archive format support in RealInstaller ([c9b291e](https://github.com/loonghao/vx/commit/c9b291e020a3d69f427304c2554301743b4705be))
* **manifest:** add provider.toml for all remaining providers ([d389553](https://github.com/loonghao/vx/commit/d38955345c8c996b36ac4258afb3fec3153649eb))
* **manifest:** implement provider override mechanism and add documentation ([aa0aa3f](https://github.com/loonghao/vx/commit/aa0aa3f9b59b74024b82a631be1d761f3217db11))
* **manifest:** implement RFC 0012 - Provider Manifest system ([8c23cd8](https://github.com/loonghao/vx/commit/8c23cd8c076d05d68298258c1d791c3fe99ff271))
* **manifest:** implement RFC 0018 Phase 2 user experience features ([5717bb0](https://github.com/loonghao/vx/commit/5717bb0bbad0cc26a248b77dc92285759086a17f))
* **msvc:** implement environment variable injection for MSVC runtime ([1fed486](https://github.com/loonghao/vx/commit/1fed486a8145108932d0d8f4455c779307283900)), closes [#353](https://github.com/loonghao/vx/issues/353)
* **project-analyzer:** add Electron and Tauri framework detection ([2c84210](https://github.com/loonghao/vx/commit/2c8421023f0d58efa95d270323ba85f856c4a99a))
* **provider:** add .NET SDK provider ([68b4196](https://github.com/loonghao/vx/commit/68b4196de76992d97358a4f118160636cd2dada4))
* **provider:** add release-please provider ([ee4c934](https://github.com/loonghao/vx/commit/ee4c93405736f4272e61c8ae5ce4831d8c65fbad))
* **providers:** add ImageMagick provider ([d617818](https://github.com/loonghao/vx/commit/d6178180926ae9c8c5e59126c774bce575eb3543))
* **providers:** add inherit_system_vars to additional providers ([de3488c](https://github.com/loonghao/vx/commit/de3488c50f2d0092cabd838887ae88cabfebe2be))
* **providers:** add jq provider with binary layout support ([fc9aa90](https://github.com/loonghao/vx/commit/fc9aa904c5115adcef407b100e24b06cbcd3271d))
* **providers:** add nasm and yasm assembler providers ([fe6e6ba](https://github.com/loonghao/vx/commit/fe6e6bab0e6510a1d99ca12447dbb8c6bff1c56a))
* **providers:** add winget and nuget providers with system install strategies ([fd2629d](https://github.com/loonghao/vx/commit/fd2629d9ed3587ccf7b03a552e5df1e720a1953f))
* **python:** add Python 3.7 support (Windows only via Python.org embeddable) ([824623e](https://github.com/loonghao/vx/commit/824623e55818744c4ebb7bd31063d60a39e24e06))
* **python:** add Python provider using python-build-standalone ([fc465a5](https://github.com/loonghao/vx/commit/fc465a5b8463222ef3c335ec88647e1a132fd580))
* **resolver:** add subprocess PATH inheritance for vx-managed tools ([74b6d21](https://github.com/loonghao/vx/commit/74b6d21cde34ee5ca8f07d49d0b70a9f087596ba))
* **resolver:** implement lock file mechanism ([0214aa7](https://github.com/loonghao/vx/commit/0214aa7c9b209b907239960874d709d9e81c29e4))
* **resolver:** implement RFC 0026 unified runtime provider relationships ([3c3a96b](https://github.com/loonghao/vx/commit/3c3a96b7e92620a52485717a3e4fa7bb3dd53504))
* **resolver:** prioritize project vx.toml tool versions in subprocess PATH ([fa5f18c](https://github.com/loonghao/vx/commit/fa5f18ce1a4ab5afeca7339979939703a583010d))
* **resolver:** resolution cache pipeline and unified cache-mode ([4d95563](https://github.com/loonghao/vx/commit/4d955634bd06f128c1d1b425d9c4393013492ad9))
* **runtime:** add plugin system with dynamic provider loading - Add plugin.rs with PluginLoader and ProviderLoader trait - Export plugin module in vx-runtime lib.rs - Add load_from_manifests method to ManifestRegistry - Add set_provider_loader method to ProviderRegistry - Add libloading dependency for dynamic library loading ([ec1df78](https://github.com/loonghao/vx/commit/ec1df78fe015fa73f7b984937e9c8197720c24c4))
* **runtime:** add pre_run hook for provider-specific setup ([e059a61](https://github.com/loonghao/vx/commit/e059a6196e88ddc1fe311ef5b1f824234cd46e7d))
* **runtime:** load constraints from embedded provider manifests ([ec9f933](https://github.com/loonghao/vx/commit/ec9f933db791e1bf87a782b40865f22a74af9eed))
* **runtime:** use indicatif for download progress and add E2E CDN tests ([7287824](https://github.com/loonghao/vx/commit/7287824f1945dca4c83b2b4036e3bb67200eef66))
* **self-update:** enhance with progress bar, checksum verification, and version selection ([2013c11](https://github.com/loonghao/vx/commit/2013c112463b5b9cd4fc3c64cfb513d46295e3f4))
* **shim:** implement RFC 0027 implicit package execution with auto-install ([9e3fa3a](https://github.com/loonghao/vx/commit/9e3fa3a068b0582e7bd2bb84782764fbb83e07b9))
* **system-pm:** add silent installation support for Windows package managers ([00ead9d](https://github.com/loonghao/vx/commit/00ead9d510b05e44d41160da2149435eccf503ca))
* **system-pm:** prioritize winget on Windows (built-in on Win11) ([208c99f](https://github.com/loonghao/vx/commit/208c99f2218b26e77129335dcf3fa00dc1473b76))
* **test:** add --ci mode for full end-to-end testing ([6576953](https://github.com/loonghao/vx/commit/657695375172d4d371a468ed519f07f12bb604f5))
* **test:** add --vx-root and --temp-root for isolated CI testing ([fde4a2f](https://github.com/loonghao/vx/commit/fde4a2fd9883bb3748d42a88de769639284d46e9))
* **test:** add comprehensive vx test command for provider testing ([b7ed2e8](https://github.com/loonghao/vx/commit/b7ed2e8afb5b723080b2fca1a2943d00c5724037))
* vx-args ([dace989](https://github.com/loonghao/vx/commit/dace98932dd5d76039f5febb43c20a14f4b8c41a))
* **vx-console:** add unified console output system ([3e891b9](https://github.com/loonghao/vx/commit/3e891b9125f6a82e44838c6cd7c7b24bea0067bb))
* **vx-console:** add unified console output system ([fb49b97](https://github.com/loonghao/vx/commit/fb49b97a74f9b0b3c411286c6df50871bce05b51))
* **vx-console:** implement P0-P2 features ([7b27925](https://github.com/loonghao/vx/commit/7b2792583da235bc6d1f83b50144f8f40176f6e1))
* **vx-env:** unify shell spawning with embedded assets ([9a03c5a](https://github.com/loonghao/vx/commit/9a03c5aed9e3d16142cb5d91b6e8f44cbb0e8a96))
* **vx-paths:** add debug logging for executable search ([2ebda23](https://github.com/loonghao/vx/commit/2ebda23f246906bd1b4715bdf551b425306b5e76))
* **vx-project-analyzer:** implement RFC 0003 project analyzer ([efc2ca0](https://github.com/loonghao/vx/commit/efc2ca0b0b3235151e41c7b2b3ea1a77226765e3))
* **vx-runtime:** implement RFC 0022 post-install normalization ([68e1cb9](https://github.com/loonghao/vx/commit/68e1cb93b9e7e41d0713c487b004e806a6aaa781))
* **vx-runtime:** integrate version resolver into Runtime trait ([f19fd10](https://github.com/loonghao/vx/commit/f19fd1047cfd55c8519f4591acfa4bdbee8365ce))
* **yarn:** use vx-managed Node.js for Yarn 2.x+ corepack ([31565df](https://github.com/loonghao/vx/commit/31565df7243d8ab152faf792afdeceff567d7303))


### Bug Fixes

* add backward compatibility for artifact naming in install scripts ([3e46de1](https://github.com/loonghao/vx/commit/3e46de1fa45bb1e8fa7e4445f79fb472ee26876a))
* add BundledCommand variant to InstallStrategyDef and fix nuget provider registration ([26afdff](https://github.com/loonghao/vx/commit/26afdffff92f63f9428664f5b284fde4c41d33a5))
* add BundledConfig to support RFC 0028 bundled runtime pattern ([abedc08](https://github.com/loonghao/vx/commit/abedc08b512727dba322103a694ce99ac05f8c94))
* add install() method to RustupRuntime for system rustup detection ([b6bdb3f](https://github.com/loonghao/vx/commit/b6bdb3fc86750edc68f78c1ce3fbf574d5c640de))
* add libc dependency for Unix-specific syscalls ([3e422ea](https://github.com/loonghao/vx/commit/3e422ea8c92bf590e166fd441ecb2c209eac53ea))
* add missing PathBuf import in services tests ([d7eed59](https://github.com/loonghao/vx/commit/d7eed5970b65a43c4b0a03424c35116d4e52bec6))
* add missing platform_paths field in BundledTool test ([2225547](https://github.com/loonghao/vx/commit/2225547dec06d5201b05a0e89a17073aff0d7e09))
* add missing subdir field and ignore crates dead links in docs ([8438853](https://github.com/loonghao/vx/commit/84388535f7c7db3edde897af5ab2bfd00321dbb6))
* add on.push trigger for CodeQL to show alerts in Security tab ([a3b704d](https://github.com/loonghao/vx/commit/a3b704d4f195ee1b14c671f11746aac57be81863))
* add serial_test to prevent env var race conditions in tests ([49eb42b](https://github.com/loonghao/vx/commit/49eb42b231c6793cb2a44f6f495bf96267eacf38))
* allow system tool fallback in isolation mode ([28dbb23](https://github.com/loonghao/vx/commit/28dbb23f706a786b33df11c1d0ae4dd107353060))
* AWS CLI and Windows self-update improvements ([97cb866](https://github.com/loonghao/vx/commit/97cb8663e5b4802a0e7e472235ba870222020640))
* AWS CLI version list and Windows MSI handling ([01d2009](https://github.com/loonghao/vx/commit/01d200942f71a807abcbb9adf8ebb82cdfdfc62e))
* **awscli:** correct Linux executable path to aws/dist/aws ([5d1f84a](https://github.com/loonghao/vx/commit/5d1f84a833120047282fd4be7d52702672fa12c9))
* **awscli:** use post_extract instead of post_install for MSI installation ([126d795](https://github.com/loonghao/vx/commit/126d7950187fbb32cc137e3003dd2d1f6fb46f37))
* batch update all remaining files to use Platform::new() ([bb0ca17](https://github.com/loonghao/vx/commit/bb0ca17ccb9303767cb9d5c99ca5ec7c52c29587))
* change default isolate to false for child process access to system tools ([0b5bba0](https://github.com/loonghao/vx/commit/0b5bba01d846500e4c82977e836b90fd5c341195))
* check both vx.toml and .vx.toml in workflow test ([2a3dafc](https://github.com/loonghao/vx/commit/2a3dafc66bab3bc25c1cfda1cd6dc0b5d6959c96))
* CI issues for brew, ffmpeg, and vscode providers ([353f4f7](https://github.com/loonghao/vx/commit/353f4f7b81c407c6a484835f7972d60dbbb84ee8))
* CI test issues for vscode, docker, and rcedit ([210fcbf](https://github.com/loonghao/vx/commit/210fcbf66a52f5c643edbd46119c1212d2792ffb))
* **ci:** correct release asset naming in Docker and package manager workflows ([625a959](https://github.com/loonghao/vx/commit/625a9598f1573ac7bd25d3da71dfcc5250a85728))
* **ci:** downgrade actions/checkout from v6 to v4 ([49dee31](https://github.com/loonghao/vx/commit/49dee3117e3a31e89c53dff6f97a1615048bde2f))
* **ci:** ensure system paths available for npm postinstall scripts ([91e4f51](https://github.com/loonghao/vx/commit/91e4f51a926bf42a10dd4fadcd24341ee6108bca))
* **ci:** fix Homebrew and Scoop publishing issues ([ab81dd6](https://github.com/loonghao/vx/commit/ab81dd6518e55e3ac979c5812175d9a46c17f475))
* **ci:** fix release workflow skipping and docker manifest creation ([4f3672d](https://github.com/loonghao/vx/commit/4f3672dd41575d8cd92d230db4e90335910d3645))
* **ci:** handle cancelled jobs and empty test_packages fallback ([c09e5ff](https://github.com/loonghao/vx/commit/c09e5ff1b342054098efd9e5ea42d649f3005bd7))
* **ci:** normalize version for WinGet to resolve version format issue ([70b3a33](https://github.com/loonghao/vx/commit/70b3a335fe066b3092b315bebe2795892065c3aa))
* **ci:** only skip spack on Windows, not all platforms ([e9522f4](https://github.com/loonghao/vx/commit/e9522f4fb1348660b1b704b86a954a39815bdd54))
* **ci:** only use releases with available assets ([e10afcd](https://github.com/loonghao/vx/commit/e10afcdb0fe89bbc23789a4c383322a464c49ab2))
* **ci:** pass GITHUB_TOKEN to vx test commands to avoid rate limits ([9a72d77](https://github.com/loonghao/vx/commit/9a72d7715961ab31a1efd23e1b16bd77873a6489))
* **ci:** preserve changelog and fix OpenSSL cross-compilation ([52dfbfd](https://github.com/loonghao/vx/commit/52dfbfdb8c59999b93abbeae474aa8a74898b924))
* **ci:** remove package-name from release-please config to fix tag pattern ([5b72bf2](https://github.com/loonghao/vx/commit/5b72bf2525ff01f45e8d23ca9f778a2da5a7b607))
* **ci:** remove tests for non-existent commands and ignore RFC dead links ([da29e22](https://github.com/loonghao/vx/commit/da29e2261941f2b6d712958e083d9263fcf4a6f5))
* **ci:** replace Ash258/Scoop-GithubActions with custom script ([eb960c2](https://github.com/loonghao/vx/commit/eb960c23b966527135a0d8a277bf13bd4ee33325))
* **ci:** replace Justintime50/homebrew-releaser with custom script ([7aa8441](https://github.com/loonghao/vx/commit/7aa844135d657b6e02ca558e6de6be65a653d9e8))
* **ci:** resolve doctest failure and integration test timeout ([be2104d](https://github.com/loonghao/vx/commit/be2104d6f55ad21e11a679ae82196fff0e084152))
* **ci:** resolve release workflow trigger issue ([e08dc7d](https://github.com/loonghao/vx/commit/e08dc7d77987b937e85917684f4265418d68f406))
* **ci:** resolve RPM build, Docker manifest, and release notes issues ([11169d2](https://github.com/loonghao/vx/commit/11169d27b15bcd07dae6bc117d0725a41ed1dd04))
* **ci:** restore corrupted workflow files and upgrade actions to v6 ([f0fbd40](https://github.com/loonghao/vx/commit/f0fbd4052cdb73d244eb2a0fc5011400b3750aa2))
* **ci:** use actions/setup-node instead of vx for docs build ([a45dff3](https://github.com/loonghao/vx/commit/a45dff3aff704350ad7acc3ee5dd35ba977845d0))
* **ci:** use cross-platform sha256 checksum and unify workspace versions ([31e8e5d](https://github.com/loonghao/vx/commit/31e8e5dd2daf02c0e7b0120a25e131e90c48bf17))
* **cli:** move error_str into windows-only cfg block ([0892cd4](https://github.com/loonghao/vx/commit/0892cd49872b85117763e8eb60840256b82bc1e2))
* **clippy:** move generate_dockerfile before tests and remove duplicate if branches ([af2962b](https://github.com/loonghao/vx/commit/af2962be1accef980b90d6ef2fce28983a401528))
* **cli:** update tests for new multi-tool install API ([7d0d2ed](https://github.com/loonghao/vx/commit/7d0d2ed711acf832c0afde94e96a261565f0c227))
* conditional import for ShellScript and fix test logic ([889995e](https://github.com/loonghao/vx/commit/889995e777b8ecb255d90cc70c1ed38c948ba37e))
* correct manifest syntax for awscli, azcli, and rust providers ([b37860d](https://github.com/loonghao/vx/commit/b37860d231c5bc5c36567754363691e4b0cb0be5))
* correct rm alias test to check for Remove instead of Uninstall ([0f5f147](https://github.com/loonghao/vx/commit/0f5f147849fdaa881c6f27b8ee4f0ff00fcdf05a))
* correct test file to use Vec instead of arrays ([5bd8e5f](https://github.com/loonghao/vx/commit/5bd8e5fc61acd635dfe700957b04e255d699e747))
* **deps:** update react monorepo to v19 ([ea68f12](https://github.com/loonghao/vx/commit/ea68f126baa83dfe710499ab5bc78f835bd0bacd))
* **deps:** update rust crate dirs to v6 ([e9d9a0f](https://github.com/loonghao/vx/commit/e9d9a0ffce5336c2746e91702a8a853631c209f4))
* **deps:** update rust crate libloading to 0.9 ([4593ea8](https://github.com/loonghao/vx/commit/4593ea8508d1f9afdbf5a0a9e8e0ae64d71032bc))
* **deps:** update rust crate turbo-cdn to 0.6 ([e864c99](https://github.com/loonghao/vx/commit/e864c994f90dd65d39aa1b506e81e9296e041b30))
* **deps:** update rust crate turbo-cdn to 0.8 ([9753c66](https://github.com/loonghao/vx/commit/9753c663f20038b88bf3cfcf5b8aa5290821293a))
* **deps:** update rust crate winreg to 0.55 ([67c5327](https://github.com/loonghao/vx/commit/67c53278e7f718c27ec0745ea215d6223714bad7))
* **docker:** add gcompat for glibc compatibility on Alpine ([d395244](https://github.com/loonghao/vx/commit/d39524466b17e8a7b668def27223a34f5cd6155a))
* **docker:** add libatomic1 for Node.js compatibility ([87ea2bd](https://github.com/loonghao/vx/commit/87ea2bd0acd25f41e322b81bf4ce89b46973b543))
* **docker:** add libc6-compat and verify binary before USER switch ([0f9d9eb](https://github.com/loonghao/vx/commit/0f9d9ebe4053e46a674e616cad65768e4a597f12))
* **docker:** use musl binaries for Alpine compatibility ([315af4f](https://github.com/loonghao/vx/commit/315af4f33d06aa661c9b9d76155de5b29f7d92e0))
* **docker:** use Ubuntu 24.04 for glibc 2.39 compatibility ([c637cf0](https://github.com/loonghao/vx/commit/c637cf027b9bea6f6a2d1c66dac80e958155af05))
* **docker:** use UID/GID 1001 to avoid conflict with existing ubuntu user ([da5b085](https://github.com/loonghao/vx/commit/da5b085dfb96a4d3cd44cea493c876e26b780c1a))
* enable cdn-acceleration for all targets (turbo-cdn 0.6 uses rustls) ([3dab167](https://github.com/loonghao/vx/commit/3dab1675cb9b57f624bd2928d52e18bf0c34991e))
* ensure Python 3.12 for pip package installation in CI ([a7e55f3](https://github.com/loonghao/vx/commit/a7e55f38b529755b5d5070509c493a1cf1b39db6))
* escape mustache syntax in docs for VitePress compatibility ([b453ecd](https://github.com/loonghao/vx/commit/b453ecd7c39eef0f18b4b13e824d67989ee6e2c5))
* **executor:** add executable existence check before execution ([c625ecc](https://github.com/loonghao/vx/commit/c625ecc51a8b5510c0947551bf8a089aed5f86fd))
* **executor:** add fallback for Yarn 2.x+ to auto-install Node.js ([2228e5e](https://github.com/loonghao/vx/commit/2228e5e6dd2c04f095680dc5370e79a633221dfe))
* **executor:** add platform check at execute() entry point ([4a20a18](https://github.com/loonghao/vx/commit/4a20a18fbeedac2c12cba33d1b0067c9fa9dc778))
* **executor:** check platform support before installing runtime ([96e0495](https://github.com/loonghao/vx/commit/96e049517960863956aafb4ebc043c945c7b0c02))
* **executor:** ensure essential system paths in build_command ([c1e7fb8](https://github.com/loonghao/vx/commit/c1e7fb8f32816c9419843593322c40c112002e7c))
* **executor:** ensure essential system paths in isolated mode ([d1ca27c](https://github.com/loonghao/vx/commit/d1ca27cc1e402496481e57a0de4944e458c99d36))
* **extension:** add missing subdir field to RemoteSource tests ([6aa9da1](https://github.com/loonghao/vx/commit/6aa9da123e16bd812cdd099120cf9c85ec659dd3))
* filter system PATH to include only essential directories in isolated mode ([60663c0](https://github.com/loonghao/vx/commit/60663c0a2ebf48b7474341896af449a1fcfacb22))
* fix code formatting and increase Windows benchmark thresholds ([15a2807](https://github.com/loonghao/vx/commit/15a2807eb553386521faf40442061d110da12818))
* fix remaining tests to use platform-specific directory structure ([de5b0b7](https://github.com/loonghao/vx/commit/de5b0b7e86ef08589ff57e85330d05e3d7c7539d))
* gate msvc provider on windows ([fbf2d9d](https://github.com/loonghao/vx/commit/fbf2d9d53889c03f62bb1bc3220919d805029c12))
* handle Ctrl+C exit gracefully and fix Java download URL detection ([1168fb9](https://github.com/loonghao/vx/commit/1168fb9108a6403e1471e718018212784ab78aab))
* **imagemagick:** add custom resolve_version for special version format ([f1c69eb](https://github.com/loonghao/vx/commit/f1c69eb1bbb0bfac814538d6de4330232f6f36d9))
* **imagemagick:** implement system package manager fallback for macOS/Windows ([40f23e9](https://github.com/loonghao/vx/commit/40f23e9cd87fc88d54caef83dd5301f8f773a91a))
* **imagemagick:** use package managers on Windows instead of direct download ([016b195](https://github.com/loonghao/vx/commit/016b19581d2f3d8941ebda2d859023da70f56481))
* improve HTTP error messages ([58f4e40](https://github.com/loonghao/vx/commit/58f4e406698eb68464c84d4de738c7be22a1635e))
* increase retry count and delay for network resilience ([1687749](https://github.com/loonghao/vx/commit/1687749e5075059a06dad02a720e18de23b64929))
* inherit_system_vars now properly passes all system vars to child processes ([a144bee](https://github.com/loonghao/vx/commit/a144beee3741fbd64f6f6429e690d07f24f9e859))
* **installer:** support versioned artifact naming format ([0449e2d](https://github.com/loonghao/vx/commit/0449e2d2b667a40c9733df31c1db283c8839b83a))
* **jq:** use tag_prefix for version parsing ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* make test-all-providers.sh compatible with Bash 3.x (macOS) ([8332946](https://github.com/loonghao/vx/commit/8332946e70b18e69a540d4cab6d786d6bfdf7415))
* **make:** use static versions and system package manager installation ([617baf1](https://github.com/loonghao/vx/commit/617baf1899a44a3abbf3013301efa98891dd95a1))
* **make:** use success_system_installed() for verification ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* **meson:** use PackageRuntime trait for pip-based installation ([42fa897](https://github.com/loonghao/vx/commit/42fa897f935a6eb2ade1621f2bee551e79768ad0))
* **msbuild:** suppress dead_code warning for non-Windows platforms ([92e6470](https://github.com/loonghao/vx/commit/92e6470061d4c7faea42a454b22b7c4512a1ca79))
* **node:** add post_extract hook to ensure npm/npx executable permissions ([e2dbc0c](https://github.com/loonghao/vx/commit/e2dbc0ca7539388aa41cc765b49f83a9f41e91dc))
* **nuget:** fix executable path for binary installation ([d7ea8af](https://github.com/loonghao/vx/commit/d7ea8af01c077fe75c9a1ed0e56d986ddcb4871c))
* pass version to Rust ecosystem dependencies during installation ([14ec0e1](https://github.com/loonghao/vx/commit/14ec0e1b54541aa6a5b3d824fae268caf3156022))
* pin Node.js to LTS v22 in Docker images ([a13b7e6](https://github.com/loonghao/vx/commit/a13b7e6ad42a4281fcde9ce43db5b749639ad107))
* **pip:** pass bin_name to install functions for correct binary verification ([f9cb7ef](https://github.com/loonghao/vx/commit/f9cb7ef67ebe01a4cfba9be92eac0c0b1ab006e7))
* PowerShell escaping and display issues in vx dev ([b4b86ef](https://github.com/loonghao/vx/commit/b4b86efbda8b73ef330c5c6fbcd40e75f80ad518))
* prefer gnu over musl for Linux targets in install scripts ([87b4af8](https://github.com/loonghao/vx/commit/87b4af898608bf9be9c2cb1fb5f3209884a6c001))
* prefix unused variable with underscore to fix warning ([10d576d](https://github.com/loonghao/vx/commit/10d576d4309feeed29f190932cc41a2be8a4c7ef))
* properly detect system dependency versions and find bin directories ([c232ff0](https://github.com/loonghao/vx/commit/c232ff08a9b2dec5ba63d8821d6c6196800cf73c))
* provide cross-platform stub for msvc provider ([3327e92](https://github.com/loonghao/vx/commit/3327e9254b3bc00bfa3ed0936ea05525415d0b66))
* **providers:** fix rust path calculation and make Windows support ([9f636d4](https://github.com/loonghao/vx/commit/9f636d41efebd5c9094708faac507db4a2df35f8))
* **providers:** implement strip_prefix for archive extraction ([67dcabe](https://github.com/loonghao/vx/commit/67dcabef5c0967fe21b1d13467944d873b147185))
* **providers:** platform gating, npm shims, and vscode/jq layouts ([56d0bb4](https://github.com/loonghao/vx/commit/56d0bb46ded371036aa4d515d7cca8fc7ec64ccb))
* **python:** correct filename format and add version resolution ([88ef92a](https://github.com/loonghao/vx/commit/88ef92aa0978fb8e37ffa0eef46e5c38deaf0c17))
* **python:** update release_date for EOL versions ([b1d9de3](https://github.com/loonghao/vx/commit/b1d9de3dcd7b2dfd87f3779cbf2d4b027f1e6e60))
* reject invalid version strings instead of treating as latest ([fd3411a](https://github.com/loonghao/vx/commit/fd3411a42c36b38e4d0378572f5434f6cd1d26d7))
* **release:** fix version extraction and rate limit handling ([05c281a](https://github.com/loonghao/vx/commit/05c281a2127737a816b5094aa9115dc16e7a58cf))
* remove dead links and format code ([7a56b7e](https://github.com/loonghao/vx/commit/7a56b7e6b4264c626b5caca4f697950d6e8f4db0))
* remove dead links in docs ([ba3a15c](https://github.com/loonghao/vx/commit/ba3a15caa84745812bc265971b41d6f47cd82e60))
* remove default() calls on unit structs for clippy lint ([c47a092](https://github.com/loonghao/vx/commit/c47a092274f09706870f5d37cca0282d1526fb14))
* remove duplicate profile config from .cargo/config.toml ([5a8d454](https://github.com/loonghao/vx/commit/5a8d454df2ebf743a4c68df4301ee85152fc4d15))
* remove needless borrow in platform redirection test ([b606f59](https://github.com/loonghao/vx/commit/b606f59323f8b3d9383f82c388522d6661a87827))
* remove redundant if_same_then_else in test ([81ba2fe](https://github.com/loonghao/vx/commit/81ba2fea3443aa233475e42d2dd1408425875a88))
* remove redundant tracing import in vscode provider ([c22961f](https://github.com/loonghao/vx/commit/c22961ff18e2a934f8abef2f6d28b2ef9ab76a54))
* remove unintended benchmark file and fix clippy error ([d3766ff](https://github.com/loonghao/vx/commit/d3766ff97010faec33491694d4ee38f72a78361a))
* remove unsupported crt-static for Linux gnu targets ([f5f9847](https://github.com/loonghao/vx/commit/f5f9847d1f5d2ac6c1bf7b8301f22b70d7c9706c))
* remove unused PermissionsExt import in bundle.rs ([4ebfd63](https://github.com/loonghao/vx/commit/4ebfd638206182ce1ae0d6d47b904da948bb4057))
* remove useless assert!(true) to fix clippy warning ([75f8cdf](https://github.com/loonghao/vx/commit/75f8cdf8da54e7e24633291db3266b1ad3f47453))
* remove yasm/pipx providers and fix Windows test hanging ([c814502](https://github.com/loonghao/vx/commit/c81450208beee49f9c39875831411f960c30faa3))
* replace non-existent vx stats command with vx cache info ([1c975ce](https://github.com/loonghao/vx/commit/1c975ce5cadc51d26cac688cc8cd64b507f8d68b))
* resolve CI failures on Windows ([6f7c5d9](https://github.com/loonghao/vx/commit/6f7c5d9d1139c32433660ccdd5b6ff532613f05d))
* resolve CI issues and improve platform-suffixed executable search ([47d3633](https://github.com/loonghao/vx/commit/47d363370879c67a630fabb37d459527cd3800de))
* resolve cl alias via msvc canonical runtime ([26ed20a](https://github.com/loonghao/vx/commit/26ed20a118a7dac2109d7faba4ba7beecb940e5f))
* resolve clippy error and pnpm test failures ([8e2e4f4](https://github.com/loonghao/vx/commit/8e2e4f4df3b316f920c106be86fb6482828c5e3c))
* resolve clippy field_reassign_with_default warnings ([6aa7896](https://github.com/loonghao/vx/commit/6aa78968e2f6b0d67ef830dc6fe503fff63722bf))
* resolve clippy for_kv_map warning in vx-cli setup ([fb01cec](https://github.com/loonghao/vx/commit/fb01cec0a5898b173597d206c387cad62be955fe))
* resolve clippy for_kv_map warning in vx-setup ([3d2de48](https://github.com/loonghao/vx/commit/3d2de480858adff79f2fe70e287889380ba6344c))
* resolve clippy needless_lifetimes warning ([b47e993](https://github.com/loonghao/vx/commit/b47e9936a623da8102479ec5317e3e8f47291ed9))
* resolve clippy unnecessary_get_then_check in tests ([063e5ce](https://github.com/loonghao/vx/commit/063e5cef37dddfda794fc8b6657af6845aeb9400))
* resolve clippy useless_vec warnings ([487ab66](https://github.com/loonghao/vx/commit/487ab66b61bd56db219c3641772f80319d32beaa))
* resolve clippy warnings (io_other_error, for_kv_map) ([e2c7b10](https://github.com/loonghao/vx/commit/e2c7b1026847d3e8c8162ad84d24430362919ddd))
* resolve clippy warnings (redundant closure, single match, collapsible if, assertions) ([46e3e1f](https://github.com/loonghao/vx/commit/46e3e1f6cdc2e77971d11f04be7fc414db358a31))
* resolve clippy warnings and doctest error ([7ebdf3e](https://github.com/loonghao/vx/commit/7ebdf3e729850015bc0fb7488bbab5ed42c5cc0c))
* resolve clippy warnings and fix test failures ([2463f79](https://github.com/loonghao/vx/commit/2463f79342f039d561bc983a0df7ba02a2469400))
* resolve clippy warnings and improve API design ([0d9b9bf](https://github.com/loonghao/vx/commit/0d9b9bf3591117a6e8896ad5bc9c1f2bb62de77a))
* resolve clippy warnings and improve code quality ([19ead74](https://github.com/loonghao/vx/commit/19ead7480c94b48b1db01291bb96eedf14488bff))
* resolve clippy warnings in vx-args parser ([ff3526e](https://github.com/loonghao/vx/commit/ff3526e52b63857e0f166d788e11c1465c30132e))
* resolve clippy warnings in vx-system-pm and test handler ([7824e97](https://github.com/loonghao/vx/commit/7824e97d44fe1e8e1f8f059647ebacae73d73b0a))
* resolve clippy warnings in vx-version-fetcher ([91fa10e](https://github.com/loonghao/vx/commit/91fa10e2361b6ff1c3dfcecf1d6793f233ddb26d))
* resolve compilation errors and lint issues in vx-system-pm ([d97cbec](https://github.com/loonghao/vx/commit/d97cbec48d5316178830f0dd7321a00fb6cc6c93))
* resolve compilation errors and update documentation ([5cf23f2](https://github.com/loonghao/vx/commit/5cf23f24ee8c5c959b74cb61b5c138996d0c306f))
* resolve compilation errors in tests and warnings ([197f3d2](https://github.com/loonghao/vx/commit/197f3d28a712d2db3d1314b604df3a71a01be6ed))
* resolve doc tests and lint issues ([5c6a4ab](https://github.com/loonghao/vx/commit/5c6a4abe2defa922c6c7bf7aacb0b2acc0f39e45))
* resolve lint warnings and update rust tests ([8d20ea7](https://github.com/loonghao/vx/commit/8d20ea7179b9367289a3945a1402abc5a10b4bfe))
* resolve msi.rs compilation errors on non-Windows platforms ([bf67aec](https://github.com/loonghao/vx/commit/bf67aec2474342d8ff50cf843d42580600441215))
* resolve PYTHONHOME template and self-update version comparison issues ([76cf1c3](https://github.com/loonghao/vx/commit/76cf1c3152fcac8f6a56bb9afb71063afef9dbcc))
* resolve remaining clippy errors and adjust benchmark thresholds ([2805010](https://github.com/loonghao/vx/commit/2805010dee055a82f9350ccf3464764d7ac4ef91))
* resolve remaining clippy warnings and flaky e2e tests ([271013f](https://github.com/loonghao/vx/commit/271013fc86d570b5f6bd159de44436fb08b789f7))
* resolve test failures and clippy warnings ([9b09ade](https://github.com/loonghao/vx/commit/9b09adee28147f3684aaf16f2a18e22a7549f707))
* resolve test failures and lint warnings ([781e423](https://github.com/loonghao/vx/commit/781e4236242a73d3a6213f544f8e30d6e546a8fb))
* resolve unused variable warnings in script_generator_tests ([064c986](https://github.com/loonghao/vx/commit/064c9868999cc2b6b042b7857aaee320fc9aa2ce))
* resolve Windows CI test failures ([8d184e2](https://github.com/loonghao/vx/commit/8d184e24f9affc94e8fa322a6f4d83fbf1eb7816))
* resolve Windows CI test failures and formatting issues ([edbe6f6](https://github.com/loonghao/vx/commit/edbe6f617800e77148084d39e66e1b2c55010d4f))
* **resolver:** version-specific deps and Windows path spaces ([d881395](https://github.com/loonghao/vx/commit/d8813956ac7213ff2ee504c51d1f8f24ddfc08d1))
* **runtime_root:** use is_file() instead of exists() for executable checks ([b629205](https://github.com/loonghao/vx/commit/b6292055ee66ade63509341c5e40e24a0b8a7c8e))
* **rust:** use tar.gz for Windows ([c29e480](https://github.com/loonghao/vx/commit/c29e480750298859c8293ee605f38aaa1abfb96a))
* **scripts:** move function definitions before usage in PowerShell script ([92ee92e](https://github.com/loonghao/vx/commit/92ee92ef060822b88e5f746778525f0546e8867d))
* **self-update:** support both legacy and versioned artifact naming ([c7e02df](https://github.com/loonghao/vx/commit/c7e02dff9d851de8493fc772174cad99b37bb670))
* **setup:** preserve boolean values in vx.toml when using vx add ([559fcb4](https://github.com/loonghao/vx/commit/559fcb40b7931ca1bb56b82aebda33d3bb4ec38f))
* simplify lock file version matching logic ([05a12b2](https://github.com/loonghao/vx/commit/05a12b274e7db13fdb7f9d05d3edc0de8b6f9d4f))
* **spack:** restrict to Unix platforms only (Linux/macOS) ([ce27acf](https://github.com/loonghao/vx/commit/ce27acf48fe9014c5a7494b9e17d434e2d016f4b))
* split PATH string before passing to join_paths ([893658e](https://github.com/loonghao/vx/commit/893658ec4b4f6facbede5912034e4ed0d274db3e))
* suppress dead_code warning in make provider ([ac5414e](https://github.com/loonghao/vx/commit/ac5414ed97fda20fe0cdf2d02edae0337bac7a78))
* **sync:** filter out non-vx tools from project analysis ([20ab704](https://github.com/loonghao/vx/commit/20ab7044a7f4183de339500bb02374493174cfd7))
* terraform API limit and add cache command ([86f726f](https://github.com/loonghao/vx/commit/86f726f0a59a8b8b8fd2511db5857c279fa61862))
* **test:** fix command parsing for quoted arguments ([5542b86](https://github.com/loonghao/vx/commit/5542b86092afdd4557786c6d729faa56a0ae644c))
* **test:** improve test command and add progress tracking ([5a01706](https://github.com/loonghao/vx/commit/5a017067f76e5e82f718419a92f1813d8367ddcf))
* **test:** improve test command logic and add --install mode ([e14c869](https://github.com/loonghao/vx/commit/e14c8694a88c40a101c64d1f66686ebb6112a0aa))
* **tests:** handle leading whitespace in release commit detection and use --inherit-env in CI ([83e6419](https://github.com/loonghao/vx/commit/83e641969b0cb0fa5ca1bf6f91113c70f1abd538))
* **tests:** increase config parse threshold for CI variability ([11ab726](https://github.com/loonghao/vx/commit/11ab726d3cd4c333706735ad2cc9230148072408))
* **tests:** prevent parent directory config search in e2e tests ([84086af](https://github.com/loonghao/vx/commit/84086af1e94b476b557e0b566d169de5fd12e27b))
* **test:** use executable_path from InstallResult for system installs ([10cebb1](https://github.com/loonghao/vx/commit/10cebb117a1728b4fc739a24e201cf8bd1b1330e))
* update boundary e2e tests to use correct CLI commands ([c675b00](https://github.com/loonghao/vx/commit/c675b0039cfca35105656969e7b82a25cc813fbc))
* update ffmpeg and zig tests to use Platform::new() ([33a4472](https://github.com/loonghao/vx/commit/33a447223005585d5d9514e9fc3e5248c757c5ec))
* update file_rename migration tests for current behavior ([42d3b01](https://github.com/loonghao/vx/commit/42d3b01a2683c9e49de0dadadc2fe2fd72118682))
* update gcloud provider tests to match implementation ([f568151](https://github.com/loonghao/vx/commit/f568151027b23c8488dfaabdedcde1ad93a5978f))
* update imagemagick, awscli, spack tests to use Platform::new() ([47cc69f](https://github.com/loonghao/vx/commit/47cc69fb0574cc196906341be6d2a84346ff803f))
* update Java provider tests to match post_extract flattening ([d0c29f0](https://github.com/loonghao/vx/commit/d0c29f02a070bbd115c14cb7469ff5deae745030))
* update migration integration tests for file-rename no-op behavior ([ed1278e](https://github.com/loonghao/vx/commit/ed1278ef8d075cb9026ddc2bfc7cab3318f90a2c))
* update more test files to use Platform::new() ([a44bc82](https://github.com/loonghao/vx/commit/a44bc8252e5b8b91734447fc0697ebb1684a8abc))
* update project_context tests to use cache commands ([9d0e475](https://github.com/loonghao/vx/commit/9d0e475d973328990b0a6490fd310911fa6610c5))
* update Python provider test to expect 2 runtimes (python, pip) ([cc396ac](https://github.com/loonghao/vx/commit/cc396aced10a4aaa36e5bb74528512f2ae596d21))
* update release workflow tag trigger pattern to match v* tags ([685d72d](https://github.com/loonghao/vx/commit/685d72d8b7fcba115355b21f85487239996f8442))
* update remaining test files to use Platform::new() constructor ([3fba1de](https://github.com/loonghao/vx/commit/3fba1de2ae85634d859bc8ababc15871f51a2669))
* update tests to match current API signatures ([6f49c2b](https://github.com/loonghao/vx/commit/6f49c2b4cf297f311fd86c5924307d0ab3c47274))
* update tests to use vx.toml (new format) instead of .vx.toml ([9a214f6](https://github.com/loonghao/vx/commit/9a214f64c3eaad66ab27cc14b90a2d3fa2016a24))
* update VSCode provider test for Linux executable path ([1b37932](https://github.com/loonghao/vx/commit/1b379324bdee93b00e821a86680038b2fb05c83c))
* use 'uv self version' instead of 'uv --version'\n\nUV's version command is 'uv self version', not 'uv --version'.\nFixes CI test failures. ([9db4c09](https://github.com/loonghao/vx/commit/9db4c09260238027e916023c5bf6fb53520a87e9))
* use BTreeMap for deterministic lock file ordering ([d3ec5b2](https://github.com/loonghao/vx/commit/d3ec5b2479fdc98afac3d1a2544fe48737d139cf))
* use is_empty() instead of len() &lt; 1 in jq provider ([4e611a8](https://github.com/loonghao/vx/commit/4e611a87078ca275d5343234428b55b539fad22f))
* use JSON text format instead of bincode for serde_json::Value caching ([b36ed89](https://github.com/loonghao/vx/commit/b36ed893c3a7d3d874820e685ddd0665eb64d486))
* use or_default() instead of or_insert_with(PathBuf::new) ([f52a2e5](https://github.com/loonghao/vx/commit/f52a2e5cff0abd446fe237f70e362dbbfbff8a40))
* use runtime.name() instead of runtime_name for store paths ([d29e363](https://github.com/loonghao/vx/commit/d29e3635b482bf7db64e571bec477a5bb3ab4534))
* use rustls instead of native-tls for musl cross-compilation ([780106e](https://github.com/loonghao/vx/commit/780106e69b0fb04fbea5f6546578b0375da8de3f))
* use string comparison instead of Path::new in filter_system_path ([3254d35](https://github.com/loonghao/vx/commit/3254d3522db503b32cdbc1aeaa79ed37f7349e3a))
* use synchronous filesystem scan for vx tools PATH building ([db0be5d](https://github.com/loonghao/vx/commit/db0be5d6b032c881b71b9b7e0d4eb4b5c927f280))
* use system cargo/rustc paths when using system rustup ([05351ba](https://github.com/loonghao/vx/commit/05351ba5e8870eca70706a9b5f0b9e297b00021f))
* use vec! macro instead of push for clippy lint ([2e06ba5](https://github.com/loonghao/vx/commit/2e06ba59e6d3fd17385b84f357e6c471eb6071d5))
* **vx-cache:** fix clippy warnings ([d76604d](https://github.com/loonghao/vx/commit/d76604d8beaadf1bb282f7e9e3c74d743a4af9e6))
* **vx-console:** add libc dependency for unix platform ([72e9a47](https://github.com/loonghao/vx/commit/72e9a472fba1a400c1cb52458950e2676bce8e0b))
* **vx-env:** fix PATH inheritance and nested bin directory detection ([727fe3d](https://github.com/loonghao/vx/commit/727fe3d229b64ae3c18f724cec89d80a74b7dc02))
* **vx-extension:** use std::io::Error::other for clippy ([9363106](https://github.com/loonghao/vx/commit/9363106bf6e8c1f3e690b6b179dbb54c187bf0d0))
* where command alias resolution and E2E test improvements ([2e70581](https://github.com/loonghao/vx/commit/2e705816c620d56f97f214293442964086ee6d1a))
* Windows CI shell syntax and skip problematic runtimes ([859ee2d](https://github.com/loonghao/vx/commit/859ee2da7a34cca81d614f09a66bd3ec87b42508))
* winget, nuget, and msbuild provider issues ([4cd0eae](https://github.com/loonghao/vx/commit/4cd0eae3d6ef3dff5ebcc2749047363da77ca59e))
* **yarn:** auto-install Node.js for Yarn 2.x+ via provided_by ([a8534e6](https://github.com/loonghao/vx/commit/a8534e6e2869a3e4c28ada7e41d9d0e696bc984c))


### Code Refactoring

* centralize cross-platform path utilities in vx-paths::platform ([6c676a6](https://github.com/loonghao/vx/commit/6c676a6be48098fb82e2cbe1c0bcf44e471ef069))
* **ci:** extract provider discovery to reusable scripts ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* **ci:** split provider tests into parallel jobs ([8a2c0b2](https://github.com/loonghao/vx/commit/8a2c0b2e122475a21bda1cffce6289fc5fb8ef8a))
* **cli:** consolidate commands and add common utilities ([a179d23](https://github.com/loonghao/vx/commit/a179d23ab19e4b85f67b552fb91d30f26c537b51))
* **cli:** modularize dev and env commands following RFC 0020 ([2872e13](https://github.com/loonghao/vx/commit/2872e13e9e11f7b9cdf54eec35a25c8360e9f883))
* **cli:** redesign add/remove commands for clarity ([6b6ccc5](https://github.com/loonghao/vx/commit/6b6ccc5a7ce4fa3bce787fb7ca41ab117d2b037f))
* **cli:** redesign cache subcommands for clarity ([12c5f0d](https://github.com/loonghao/vx/commit/12c5f0d1736b04fab386af5232752f2ff67264d7))
* **command:** use raw_arg for proper Windows cmd.exe handling ([bd8534b](https://github.com/loonghao/vx/commit/bd8534b29dc7907103255149b8744f56b7b569e9))
* **docker:** switch to Debian-based images for glibc compatibility ([1248d12](https://github.com/loonghao/vx/commit/1248d12cc00eb3b94f80e7ac0381e95e793f6219))
* improve architecture with security warnings and unified config ([538a747](https://github.com/loonghao/vx/commit/538a7475d4253fd3a51c68dd8174edf9765035ef))
* improve yarn provider corepack support ([591e91a](https://github.com/loonghao/vx/commit/591e91aff358f589664c29e2b26278ec1b423a53))
* **install:** remove legacy version compatibility from install scripts ([5e02441](https://github.com/loonghao/vx/commit/5e0244129ef03377e6a839b0932f876210ddd9b1))
* optimize version handling and add comprehensive regression tests ([43db813](https://github.com/loonghao/vx/commit/43db813df1ef2ac75cc388ac81d7379f22b13e4a))
* **runtime:** add check_platform_support() helper and platform utils ([75c61e3](https://github.com/loonghao/vx/commit/75c61e341e5c7e700167accc0a2d4ee054c0b3a9))
* **rust:** use system_install strategy via rustup ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* **script-parser:** redesign with extensible pattern provider architecture ([1482f76](https://github.com/loonghao/vx/commit/1482f766a872f94724639fdea8c6c282f6c8e68b))
* simplify .cargo/config.toml following uv's approach ([968709a](https://github.com/loonghao/vx/commit/968709a02417e25ff23d584310a6b81574efd8a4))
* use VersionFetcher interface for vscode provider ([f38ce20](https://github.com/loonghao/vx/commit/f38ce207d6dba5885970cb87b502fa9bca259a93))
* various improvements and fixes ([f321c3c](https://github.com/loonghao/vx/commit/f321c3c0bb058aeb623e4d906aa67420b93f2383))
* **vx-config:** introduce TomlWriter module for safe TOML generation ([733d564](https://github.com/loonghao/vx/commit/733d564d8812bcfd53ce8431104652a0e06a1bec))
* **vx-paths:** centralize config file constants and discovery functions ([87dbf54](https://github.com/loonghao/vx/commit/87dbf547967e13e9fa566c0d83744557576564fa))
* **vx-runtime:** split impls.rs into modular structure ([d351582](https://github.com/loonghao/vx/commit/d351582563b6960fbac1f88e067e1b035543714a))


### Documentation

* add comprehensive documentation for enhanced script system ([5cb347f](https://github.com/loonghao/vx/commit/5cb347fffaa1f4538a9e69fb6569a6e8e3395266))
* add llms.txt and llms-full.txt for LLM accessibility ([bc6420f](https://github.com/loonghao/vx/commit/bc6420f084ca5b040de64558c749442dbc8c4a13))
* add manifest-driven providers documentation (EN/ZH) ([7cbc622](https://github.com/loonghao/vx/commit/7cbc622d15cf209b0d185e92439f47f525458143))
* add plugin command documentation and update snapshots ([4af6248](https://github.com/loonghao/vx/commit/4af6248fbcb507aee2901cd30fdb1014144f14ca))
* add RFC 0027 implicit package execution documentation ([44f9671](https://github.com/loonghao/vx/commit/44f967119e6f82361d71f4b0fc37466dabe03512))
* add security documentation and update architecture docs ([6f7c1b6](https://github.com/loonghao/vx/commit/6f7c1b6ac36a6cd6570b4577a6e0c5db46a2a271))
* Add version management guide (EN/ZH) ([c06c309](https://github.com/loonghao/vx/commit/c06c3091d71512bb04456fb81c85152d2834eef1))
* add vx global command documentation (RFC 0025) ([c2d63f8](https://github.com/loonghao/vx/commit/c2d63f8e2832181d5a3854e703852b6585dcc838))
* add vx test command documentation (EN/ZH) ([143dd0f](https://github.com/loonghao/vx/commit/143dd0f2723189c3053e37a84493bfa7f06da757))
* complete RFC-0028 remaining tasks - Yarn 2.x+ corepack support ([7fa8f8c](https://github.com/loonghao/vx/commit/7fa8f8c5114d822d4638ec347d8f4db6edf5aeb2))
* fix dead links to non-existent network.md ([024f9be](https://github.com/loonghao/vx/commit/024f9beed9e6be0d1e4f50eb4a4314edcad70d66))
* rename .vx.toml to vx.toml across all documentation and code ([0eca931](https://github.com/loonghao/vx/commit/0eca93171ff895e8ac04d455947bda90c905292f))
* **rfc:** add design limitations and future improvements analysis ([a6b75da](https://github.com/loonghao/vx/commit/a6b75da171b1543c3e46e55a05aada71896bf2e2))
* **rfc:** add mainstream Rust CLI survey (Cargo, uv, ripgrep) to RFC 0009 ([5556a63](https://github.com/loonghao/vx/commit/5556a63210df0d337c1192582d50fc791c270002))
* **rfc:** add RFC 0009 unified console output system (vx-console) ([9450f6d](https://github.com/loonghao/vx/commit/9450f6d8a021609cec719bf149741c32f015ff21))
* **rfc:** add RFC 0013 - Manifest-Driven Provider Registration ([dff3ebc](https://github.com/loonghao/vx/commit/dff3ebc3ad2f328654ca4a4bc4ff6e67eb7d0627))
* **rfc:** enhance RFC 0013 with Provider vs Extension comparison and performance analysis ([1c302a5](https://github.com/loonghao/vx/commit/1c302a5631b942f184bdad86b2a7d0735ebe6852))
* **rfc:** integrate vx-migration framework and RuntimeMap dependency ordering ([eb447fa](https://github.com/loonghao/vx/commit/eb447fa67538a7c2e13c7c9e1ba2f8988c8f6df6))
* update GitHub Action examples to use [@main](https://github.com/main) and add auto-dependency docs ([4fe582c](https://github.com/loonghao/vx/commit/4fe582cde993018b5e9308368b493c9141ed54e8))
* update global and implicit-package-execution documentation ([1496cee](https://github.com/loonghao/vx/commit/1496ceedc55bc3e9705473f49cd17f4c7078426a))
* update Homebrew installation instructions ([1ff78b5](https://github.com/loonghao/vx/commit/1ff78b50d291880d3e7bfdac9e627f4a2869ac6e))
* update RFC status to reflect implementation progress ([9ebd753](https://github.com/loonghao/vx/commit/9ebd7533d7442fad22a8dde9b75d34861fae6fe3))

## [0.6.31](https://github.com/loonghao/vx/compare/vx-v0.6.30...vx-v0.6.31) (2026-02-06)


### Bug Fixes

* update release workflow tag trigger pattern to match v* tags ([685d72d](https://github.com/loonghao/vx/commit/685d72d8b7fcba115355b21f85487239996f8442))

## [0.6.30](https://github.com/loonghao/vx/compare/vx-v0.6.29...vx-v0.6.30) (2026-02-06)


### Features

* adopt cargo-dist for release workflow and fix tag pattern ([c2d55df](https://github.com/loonghao/vx/commit/c2d55dfa50880e4873fe645349576d50a7cfe0ec))

## [0.6.29](https://github.com/loonghao/vx/compare/vx-v0.6.28...vx-v0.6.29) (2026-02-06)


### Bug Fixes

* **ci:** resolve RPM build, Docker manifest, and release notes issues ([11169d2](https://github.com/loonghao/vx/commit/11169d27b15bcd07dae6bc117d0725a41ed1dd04))
* **ci:** use cross-platform sha256 checksum and unify workspace versions ([31e8e5d](https://github.com/loonghao/vx/commit/31e8e5dd2daf02c0e7b0120a25e131e90c48bf17))

## [0.6.28](https://github.com/loonghao/vx/compare/vx-v0.6.27...vx-v0.6.28) (2026-02-06)


### Features

* **ci:** simplify release workflow with cargo-dist style ([5210e8b](https://github.com/loonghao/vx/commit/5210e8b0f013f23ff1aaac3f64a338c801bde473))
* **providers:** add winget and nuget providers with system install strategies ([fd2629d](https://github.com/loonghao/vx/commit/fd2629d9ed3587ccf7b03a552e5df1e720a1953f))
* **yarn:** use vx-managed Node.js for Yarn 2.x+ corepack ([31565df](https://github.com/loonghao/vx/commit/31565df7243d8ab152faf792afdeceff567d7303))


### Bug Fixes

* add BundledCommand variant to InstallStrategyDef and fix nuget provider registration ([26afdff](https://github.com/loonghao/vx/commit/26afdffff92f63f9428664f5b284fde4c41d33a5))
* add BundledConfig to support RFC 0028 bundled runtime pattern ([abedc08](https://github.com/loonghao/vx/commit/abedc08b512727dba322103a694ce99ac05f8c94))
* **ci:** resolve doctest failure and integration test timeout ([be2104d](https://github.com/loonghao/vx/commit/be2104d6f55ad21e11a679ae82196fff0e084152))
* **executor:** add fallback for Yarn 2.x+ to auto-install Node.js ([2228e5e](https://github.com/loonghao/vx/commit/2228e5e6dd2c04f095680dc5370e79a633221dfe))
* **msbuild:** suppress dead_code warning for non-Windows platforms ([92e6470](https://github.com/loonghao/vx/commit/92e6470061d4c7faea42a454b22b7c4512a1ca79))
* **nuget:** fix executable path for binary installation ([d7ea8af](https://github.com/loonghao/vx/commit/d7ea8af01c077fe75c9a1ed0e56d986ddcb4871c))
* resolve clippy needless_lifetimes warning ([b47e993](https://github.com/loonghao/vx/commit/b47e9936a623da8102479ec5317e3e8f47291ed9))
* resolve clippy useless_vec warnings ([487ab66](https://github.com/loonghao/vx/commit/487ab66b61bd56db219c3641772f80319d32beaa))
* resolve compilation errors in tests and warnings ([197f3d2](https://github.com/loonghao/vx/commit/197f3d28a712d2db3d1314b604df3a71a01be6ed))
* resolve PYTHONHOME template and self-update version comparison issues ([76cf1c3](https://github.com/loonghao/vx/commit/76cf1c3152fcac8f6a56bb9afb71063afef9dbcc))
* **resolver:** version-specific deps and Windows path spaces ([d881395](https://github.com/loonghao/vx/commit/d8813956ac7213ff2ee504c51d1f8f24ddfc08d1))
* **runtime_root:** use is_file() instead of exists() for executable checks ([b629205](https://github.com/loonghao/vx/commit/b6292055ee66ade63509341c5e40e24a0b8a7c8e))
* use JSON text format instead of bincode for serde_json::Value caching ([b36ed89](https://github.com/loonghao/vx/commit/b36ed893c3a7d3d874820e685ddd0665eb64d486))
* winget, nuget, and msbuild provider issues ([4cd0eae](https://github.com/loonghao/vx/commit/4cd0eae3d6ef3dff5ebcc2749047363da77ca59e))
* **yarn:** auto-install Node.js for Yarn 2.x+ via provided_by ([a8534e6](https://github.com/loonghao/vx/commit/a8534e6e2869a3e4c28ada7e41d9d0e696bc984c))


### Code Refactoring

* **command:** use raw_arg for proper Windows cmd.exe handling ([bd8534b](https://github.com/loonghao/vx/commit/bd8534b29dc7907103255149b8744f56b7b569e9))
* improve yarn provider corepack support ([591e91a](https://github.com/loonghao/vx/commit/591e91aff358f589664c29e2b26278ec1b423a53))
* **install:** remove legacy version compatibility from install scripts ([5e02441](https://github.com/loonghao/vx/commit/5e0244129ef03377e6a839b0932f876210ddd9b1))
* optimize version handling and add comprehensive regression tests ([43db813](https://github.com/loonghao/vx/commit/43db813df1ef2ac75cc388ac81d7379f22b13e4a))


### Documentation

* complete RFC-0028 remaining tasks - Yarn 2.x+ corepack support ([7fa8f8c](https://github.com/loonghao/vx/commit/7fa8f8c5114d822d4638ec347d8f4db6edf5aeb2))

## [Unreleased]

### Bug Fixes

* **executor:** Fix `{install_dir}` template variable not expanded when version is not specified. The template expansion now falls back to scanning the filesystem for installed versions when no explicit version is provided. This fixes the `PYTHONHOME` environment variable issue where it was set to the literal string `{install_dir}` instead of the actual installation path.
* **self-update:** Fix version comparison logic to correctly handle various version formats (`vx-v0.6.27`, `v0.6.27`, `0.6.27`). The improved `extract_semver` function now supports two-part versions (e.g., `0.6`) with optional patch version defaulting to 0. This prevents incorrect "downgrade available" messages when the current version is actually newer than the CDN version.

### Documentation

* **cli:** Update self-update documentation to mention smart version comparison feature
* **guide:** Add template variables table to environment variables section in manifest-driven providers documentation

## [0.6.27](https://github.com/loonghao/vx/compare/vx-v0.6.26...vx-v0.6.27) (2026-02-02)


### Bug Fixes

* **deps:** update react monorepo to v19 ([ea68f12](https://github.com/loonghao/vx/commit/ea68f126baa83dfe710499ab5bc78f835bd0bacd))
* **deps:** update rust crate dirs to v6 ([e9d9a0f](https://github.com/loonghao/vx/commit/e9d9a0ffce5336c2746e91702a8a853631c209f4))

## [0.6.26](https://github.com/loonghao/vx/compare/vx-v0.6.25...vx-v0.6.26) (2026-02-02)


### Features

* **provider:** add .NET SDK provider ([68b4196](https://github.com/loonghao/vx/commit/68b4196de76992d97358a4f118160636cd2dada4))


### Bug Fixes

* **ci:** fix release workflow skipping and docker manifest creation ([4f3672d](https://github.com/loonghao/vx/commit/4f3672dd41575d8cd92d230db4e90335910d3645))
* fix code formatting and increase Windows benchmark thresholds ([15a2807](https://github.com/loonghao/vx/commit/15a2807eb553386521faf40442061d110da12818))

## [0.6.25](https://github.com/loonghao/vx/compare/vx-v0.6.24...vx-v0.6.25) (2026-02-02)


### Bug Fixes

* **ci:** ensure system paths available for npm postinstall scripts ([91e4f51](https://github.com/loonghao/vx/commit/91e4f51a926bf42a10dd4fadcd24341ee6108bca))
* **ci:** handle cancelled jobs and empty test_packages fallback ([c09e5ff](https://github.com/loonghao/vx/commit/c09e5ff1b342054098efd9e5ea42d649f3005bd7))
* **ci:** resolve release workflow trigger issue ([e08dc7d](https://github.com/loonghao/vx/commit/e08dc7d77987b937e85917684f4265418d68f406))
* **ci:** use actions/setup-node instead of vx for docs build ([a45dff3](https://github.com/loonghao/vx/commit/a45dff3aff704350ad7acc3ee5dd35ba977845d0))
* **executor:** ensure essential system paths in build_command ([c1e7fb8](https://github.com/loonghao/vx/commit/c1e7fb8f32816c9419843593322c40c112002e7c))
* **executor:** ensure essential system paths in isolated mode ([d1ca27c](https://github.com/loonghao/vx/commit/d1ca27cc1e402496481e57a0de4944e458c99d36))
* **tests:** handle leading whitespace in release commit detection and use --inherit-env in CI ([83e6419](https://github.com/loonghao/vx/commit/83e641969b0cb0fa5ca1bf6f91113c70f1abd538))

## [0.6.24](https://github.com/loonghao/vx/compare/vx-v0.6.23...vx-v0.6.24) (2026-02-01)


### Features

* **env:** add default inherit_system_vars for all providers ([c4e453a](https://github.com/loonghao/vx/commit/c4e453a870449ba1e9d24eab76745cb7c7ce2e3b))
* **providers:** add inherit_system_vars to additional providers ([de3488c](https://github.com/loonghao/vx/commit/de3488c50f2d0092cabd838887ae88cabfebe2be))
* **resolver:** implement RFC 0026 unified runtime provider relationships ([3c3a96b](https://github.com/loonghao/vx/commit/3c3a96b7e92620a52485717a3e4fa7bb3dd53504))
* **shim:** implement RFC 0027 implicit package execution with auto-install ([9e3fa3a](https://github.com/loonghao/vx/commit/9e3fa3a068b0582e7bd2bb84782764fbb83e07b9))


### Bug Fixes

* change default isolate to false for child process access to system tools ([0b5bba0](https://github.com/loonghao/vx/commit/0b5bba01d846500e4c82977e836b90fd5c341195))
* filter system PATH to include only essential directories in isolated mode ([60663c0](https://github.com/loonghao/vx/commit/60663c0a2ebf48b7474341896af449a1fcfacb22))
* inherit_system_vars now properly passes all system vars to child processes ([a144bee](https://github.com/loonghao/vx/commit/a144beee3741fbd64f6f6429e690d07f24f9e859))
* resolve clippy warnings and improve API design ([0d9b9bf](https://github.com/loonghao/vx/commit/0d9b9bf3591117a6e8896ad5bc9c1f2bb62de77a))
* split PATH string before passing to join_paths ([893658e](https://github.com/loonghao/vx/commit/893658ec4b4f6facbede5912034e4ed0d274db3e))
* use string comparison instead of Path::new in filter_system_path ([3254d35](https://github.com/loonghao/vx/commit/3254d3522db503b32cdbc1aeaa79ed37f7349e3a))


### Code Refactoring

* centralize cross-platform path utilities in vx-paths::platform ([6c676a6](https://github.com/loonghao/vx/commit/6c676a6be48098fb82e2cbe1c0bcf44e471ef069))


### Documentation

* add RFC 0027 implicit package execution documentation ([44f9671](https://github.com/loonghao/vx/commit/44f967119e6f82361d71f4b0fc37466dabe03512))
* add vx global command documentation (RFC 0025) ([c2d63f8](https://github.com/loonghao/vx/commit/c2d63f8e2832181d5a3854e703852b6585dcc838))
* update global and implicit-package-execution documentation ([1496cee](https://github.com/loonghao/vx/commit/1496ceedc55bc3e9705473f49cd17f4c7078426a))

## [0.6.23](https://github.com/loonghao/vx/compare/vx-v0.6.22...vx-v0.6.23) (2026-01-31)


### Bug Fixes

* **sync:** filter out non-vx tools from project analysis ([20ab704](https://github.com/loonghao/vx/commit/20ab7044a7f4183de339500bb02374493174cfd7))


### Code Refactoring

* **script-parser:** redesign with extensible pattern provider architecture ([1482f76](https://github.com/loonghao/vx/commit/1482f766a872f94724639fdea8c6c282f6c8e68b))

## [0.6.22](https://github.com/loonghao/vx/compare/vx-v0.6.21...vx-v0.6.22) (2026-01-30)


### Features

* implement global package management with cross-language isolation ([b1e873b](https://github.com/loonghao/vx/commit/b1e873bd951c4d23937bd8886fc12a1ec3356f7f))


### Bug Fixes

* add install() method to RustupRuntime for system rustup detection ([b6bdb3f](https://github.com/loonghao/vx/commit/b6bdb3fc86750edc68f78c1ce3fbf574d5c640de))
* PowerShell escaping and display issues in vx dev ([b4b86ef](https://github.com/loonghao/vx/commit/b4b86efbda8b73ef330c5c6fbcd40e75f80ad518))
* resolve unused variable warnings in script_generator_tests ([064c986](https://github.com/loonghao/vx/commit/064c9868999cc2b6b042b7857aaee320fc9aa2ce))
* use BTreeMap for deterministic lock file ordering ([d3ec5b2](https://github.com/loonghao/vx/commit/d3ec5b2479fdc98afac3d1a2544fe48737d139cf))
* use system cargo/rustc paths when using system rustup ([05351ba](https://github.com/loonghao/vx/commit/05351ba5e8870eca70706a9b5f0b9e297b00021f))

## [0.6.21](https://github.com/loonghao/vx/compare/vx-v0.6.20...vx-v0.6.21) (2026-01-29)


### Bug Fixes

* allow system tool fallback in isolation mode ([28dbb23](https://github.com/loonghao/vx/commit/28dbb23f706a786b33df11c1d0ae4dd107353060))
* fix remaining tests to use platform-specific directory structure ([de5b0b7](https://github.com/loonghao/vx/commit/de5b0b7e86ef08589ff57e85330d05e3d7c7539d))
* remove needless borrow in platform redirection test ([b606f59](https://github.com/loonghao/vx/commit/b606f59323f8b3d9383f82c388522d6661a87827))
* resolve clippy warnings and fix test failures ([2463f79](https://github.com/loonghao/vx/commit/2463f79342f039d561bc983a0df7ba02a2469400))
* resolve clippy warnings and improve code quality ([19ead74](https://github.com/loonghao/vx/commit/19ead7480c94b48b1db01291bb96eedf14488bff))
* resolve remaining clippy errors and adjust benchmark thresholds ([2805010](https://github.com/loonghao/vx/commit/2805010dee055a82f9350ccf3464764d7ac4ef91))
* resolve test failures and lint warnings ([781e423](https://github.com/loonghao/vx/commit/781e4236242a73d3a6213f544f8e30d6e546a8fb))
* simplify lock file version matching logic ([05a12b2](https://github.com/loonghao/vx/commit/05a12b274e7db13fdb7f9d05d3edc0de8b6f9d4f))

## [0.6.20](https://github.com/loonghao/vx/compare/vx-v0.6.19...vx-v0.6.20) (2026-01-23)


### Features

* **cli:** enhance vx dev with --info option and improved status display ([e732512](https://github.com/loonghao/vx/commit/e732512f69c0c2167a0ca4545f5834d174e04bda))
* **docker:** add tools image with pre-installed uv, ruff, and node ([1a92cd1](https://github.com/loonghao/vx/commit/1a92cd12c952881f4397f29ecc5ad93a0d7d622c))
* expand platform support for multiple architectures and libc variants ([283b995](https://github.com/loonghao/vx/commit/283b995795172d3b8fe8b98f071c960adf0732ae))
* **vx-env:** unify shell spawning with embedded assets ([9a03c5a](https://github.com/loonghao/vx/commit/9a03c5aed9e3d16142cb5d91b6e8f44cbb0e8a96))


### Bug Fixes

* batch update all remaining files to use Platform::new() ([bb0ca17](https://github.com/loonghao/vx/commit/bb0ca17ccb9303767cb9d5c99ca5ec7c52c29587))
* conditional import for ShellScript and fix test logic ([889995e](https://github.com/loonghao/vx/commit/889995e777b8ecb255d90cc70c1ed38c948ba37e))
* **docker:** add gcompat for glibc compatibility on Alpine ([d395244](https://github.com/loonghao/vx/commit/d39524466b17e8a7b668def27223a34f5cd6155a))
* **docker:** add libatomic1 for Node.js compatibility ([87ea2bd](https://github.com/loonghao/vx/commit/87ea2bd0acd25f41e322b81bf4ce89b46973b543))
* **docker:** add libc6-compat and verify binary before USER switch ([0f9d9eb](https://github.com/loonghao/vx/commit/0f9d9ebe4053e46a674e616cad65768e4a597f12))
* **docker:** use musl binaries for Alpine compatibility ([315af4f](https://github.com/loonghao/vx/commit/315af4f33d06aa661c9b9d76155de5b29f7d92e0))
* **docker:** use Ubuntu 24.04 for glibc 2.39 compatibility ([c637cf0](https://github.com/loonghao/vx/commit/c637cf027b9bea6f6a2d1c66dac80e958155af05))
* **docker:** use UID/GID 1001 to avoid conflict with existing ubuntu user ([da5b085](https://github.com/loonghao/vx/commit/da5b085dfb96a4d3cd44cea493c876e26b780c1a))
* **node:** add post_extract hook to ensure npm/npx executable permissions ([e2dbc0c](https://github.com/loonghao/vx/commit/e2dbc0ca7539388aa41cc765b49f83a9f41e91dc))
* pin Node.js to LTS v22 in Docker images ([a13b7e6](https://github.com/loonghao/vx/commit/a13b7e6ad42a4281fcde9ce43db5b749639ad107))
* remove redundant tracing import in vscode provider ([c22961f](https://github.com/loonghao/vx/commit/c22961ff18e2a934f8abef2f6d28b2ef9ab76a54))
* **tests:** increase config parse threshold for CI variability ([11ab726](https://github.com/loonghao/vx/commit/11ab726d3cd4c333706735ad2cc9230148072408))
* update ffmpeg and zig tests to use Platform::new() ([33a4472](https://github.com/loonghao/vx/commit/33a447223005585d5d9514e9fc3e5248c757c5ec))
* update imagemagick, awscli, spack tests to use Platform::new() ([47cc69f](https://github.com/loonghao/vx/commit/47cc69fb0574cc196906341be6d2a84346ff803f))
* update more test files to use Platform::new() ([a44bc82](https://github.com/loonghao/vx/commit/a44bc8252e5b8b91734447fc0697ebb1684a8abc))
* update remaining test files to use Platform::new() constructor ([3fba1de](https://github.com/loonghao/vx/commit/3fba1de2ae85634d859bc8ababc15871f51a2669))
* **vx-env:** fix PATH inheritance and nested bin directory detection ([727fe3d](https://github.com/loonghao/vx/commit/727fe3d229b64ae3c18f724cec89d80a74b7dc02))
* where command alias resolution and E2E test improvements ([2e70581](https://github.com/loonghao/vx/commit/2e705816c620d56f97f214293442964086ee6d1a))


### Code Refactoring

* **cli:** modularize dev and env commands following RFC 0020 ([2872e13](https://github.com/loonghao/vx/commit/2872e13e9e11f7b9cdf54eec35a25c8360e9f883))
* **docker:** switch to Debian-based images for glibc compatibility ([1248d12](https://github.com/loonghao/vx/commit/1248d12cc00eb3b94f80e7ac0381e95e793f6219))
* use VersionFetcher interface for vscode provider ([f38ce20](https://github.com/loonghao/vx/commit/f38ce207d6dba5885970cb87b502fa9bca259a93))


### Documentation

* add llms.txt and llms-full.txt for LLM accessibility ([bc6420f](https://github.com/loonghao/vx/commit/bc6420f084ca5b040de64558c749442dbc8c4a13))

## [0.6.19](https://github.com/loonghao/vx/compare/vx-v0.6.18...vx-v0.6.19) (2026-01-20)


### Features

* **system-pm:** add silent installation support for Windows package managers ([00ead9d](https://github.com/loonghao/vx/commit/00ead9d510b05e44d41160da2149435eccf503ca))

## [0.6.18](https://github.com/loonghao/vx/compare/vx-v0.6.17...vx-v0.6.18) (2026-01-20)


### Features

* add GitHub CLI (gh) provider ([e3dd7f4](https://github.com/loonghao/vx/commit/e3dd7f45a1bcaed9611d39afa730b1de512583f0))
* **imagemagick:** add system_deps for platform-specific package managers ([131e558](https://github.com/loonghao/vx/commit/131e5587ec64f14a28f755f805185a6ff68cac2c))
* **imagemagick:** improve error messages and add e2e tests ([5d1ace2](https://github.com/loonghao/vx/commit/5d1ace2faec3d51e1066655411a78220196e6c8c))
* **installer:** add .tar.zst (Zstandard) format support ([48761b8](https://github.com/loonghao/vx/commit/48761b88098bd14bf72f3934fd1e23c488ee64a8))
* **installer:** add 7z archive format support in RealInstaller ([c9b291e](https://github.com/loonghao/vx/commit/c9b291e020a3d69f427304c2554301743b4705be))
* **providers:** add ImageMagick provider ([d617818](https://github.com/loonghao/vx/commit/d6178180926ae9c8c5e59126c774bce575eb3543))
* **python:** add Python 3.7 support (Windows only via Python.org embeddable) ([824623e](https://github.com/loonghao/vx/commit/824623e55818744c4ebb7bd31063d60a39e24e06))
* **system-pm:** prioritize winget on Windows (built-in on Win11) ([208c99f](https://github.com/loonghao/vx/commit/208c99f2218b26e77129335dcf3fa00dc1473b76))
* **vx-runtime:** implement RFC 0022 post-install normalization ([68e1cb9](https://github.com/loonghao/vx/commit/68e1cb93b9e7e41d0713c487b004e806a6aaa781))


### Bug Fixes

* **imagemagick:** add custom resolve_version for special version format ([f1c69eb](https://github.com/loonghao/vx/commit/f1c69eb1bbb0bfac814538d6de4330232f6f36d9))
* **imagemagick:** implement system package manager fallback for macOS/Windows ([40f23e9](https://github.com/loonghao/vx/commit/40f23e9cd87fc88d54caef83dd5301f8f773a91a))
* **imagemagick:** use package managers on Windows instead of direct download ([016b195](https://github.com/loonghao/vx/commit/016b19581d2f3d8941ebda2d859023da70f56481))
* remove unused PermissionsExt import in bundle.rs ([4ebfd63](https://github.com/loonghao/vx/commit/4ebfd638206182ce1ae0d6d47b904da948bb4057))
* resolve clippy field_reassign_with_default warnings ([6aa7896](https://github.com/loonghao/vx/commit/6aa78968e2f6b0d67ef830dc6fe503fff63722bf))
* **test:** use executable_path from InstallResult for system installs ([10cebb1](https://github.com/loonghao/vx/commit/10cebb117a1728b4fc739a24e201cf8bd1b1330e))


### Code Refactoring

* various improvements and fixes ([f321c3c](https://github.com/loonghao/vx/commit/f321c3c0bb058aeb623e4d906aa67420b93f2383))

## [0.6.17](https://github.com/loonghao/vx/compare/vx-v0.6.16...vx-v0.6.17) (2026-01-18)


### Features

* add Windows long path support and fix macOS/Linux installer compatibility ([e077ce8](https://github.com/loonghao/vx/commit/e077ce8217ae734811a35d1319db3256cda073cd))
* **resolver:** prioritize project vx.toml tool versions in subprocess PATH ([fa5f18c](https://github.com/loonghao/vx/commit/fa5f18ce1a4ab5afeca7339979939703a583010d))


### Bug Fixes

* add missing platform_paths field in BundledTool test ([2225547](https://github.com/loonghao/vx/commit/2225547dec06d5201b05a0e89a17073aff0d7e09))
* replace non-existent vx stats command with vx cache info ([1c975ce](https://github.com/loonghao/vx/commit/1c975ce5cadc51d26cac688cc8cd64b507f8d68b))
* resolve doc tests and lint issues ([5c6a4ab](https://github.com/loonghao/vx/commit/5c6a4abe2defa922c6c7bf7aacb0b2acc0f39e45))
* resolve Windows CI test failures ([8d184e2](https://github.com/loonghao/vx/commit/8d184e24f9affc94e8fa322a6f4d83fbf1eb7816))

## [0.6.16](https://github.com/loonghao/vx/compare/vx-v0.6.15...vx-v0.6.16) (2026-01-17)


### Features

* add GitHub auth and unified version fetcher ([e2b946d](https://github.com/loonghao/vx/commit/e2b946d32a3a64d38ab3abafdb5a40a82d564faf))


### Bug Fixes

* resolve clippy warnings in vx-version-fetcher ([91fa10e](https://github.com/loonghao/vx/commit/91fa10e2361b6ff1c3dfcecf1d6793f233ddb26d))
* use is_empty() instead of len() &lt; 1 in jq provider ([4e611a8](https://github.com/loonghao/vx/commit/4e611a87078ca275d5343234428b55b539fad22f))

## [0.6.15](https://github.com/loonghao/vx/compare/vx-v0.6.14...vx-v0.6.15) (2026-01-17)


### Bug Fixes

* add on.push trigger for CodeQL to show alerts in Security tab ([a3b704d](https://github.com/loonghao/vx/commit/a3b704d4f195ee1b14c671f11746aac57be81863))
* use rustls instead of native-tls for musl cross-compilation ([780106e](https://github.com/loonghao/vx/commit/780106e69b0fb04fbea5f6546578b0375da8de3f))

## [0.6.14](https://github.com/loonghao/vx/compare/vx-v0.6.13...vx-v0.6.14) (2026-01-17)


### Features

* add binary download support for rust/rustup and improve CI coverage ([3ae9600](https://github.com/loonghao/vx/commit/3ae9600e9345deb7a0cf2287f5cc2c5457e8a669))
* add jsDelivr CDN fallback for GitHub releases API ([43ea611](https://github.com/loonghao/vx/commit/43ea61155e0a9548434b07e2704e8ce3e5e9b174))
* add meson and make providers, fix git and yasm issues ([1203ae9](https://github.com/loonghao/vx/commit/1203ae93cdf022c8c43b86d7890525852ed1b4a8))
* add package manager install support and improve static binary handling ([7c812bc](https://github.com/loonghao/vx/commit/7c812bc4c088b058f50ce4070a4703f5f11db87b)), closes [#389](https://github.com/loonghao/vx/issues/389)
* add pipx provider and fix rust/rustup issues ([e7b2f99](https://github.com/loonghao/vx/commit/e7b2f998ed7238d8d3458e4e7de09cf4a68cd2a8))
* add provider.toml manifests for meson and make ([f914d91](https://github.com/loonghao/vx/commit/f914d91d6a61e7893c9779d6b1f8fc96fc7eabf3))
* add Runtime::store_name() method for consistent store path resolution ([682a47f](https://github.com/loonghao/vx/commit/682a47f92b16018137b1362090ab381a6c8f47dd))
* add static linking for Linux/macOS and optimize build speed ([a0b624f](https://github.com/loonghao/vx/commit/a0b624f0756a4f9fde4408485a7c7535d28f795f))
* **cli:** implement RFC 0020 Phase 2 - modular command structure ([f844015](https://github.com/loonghao/vx/commit/f844015e35b97ada8d78d04ed4135d4dbcbd3927))
* implement RFC 0020 & 0021 - system package manager integration and manifest-driven runtimes ([322f624](https://github.com/loonghao/vx/commit/322f624ab9d9f6a405cc19c03190297834e1b1fd))
* improve download timeout handling for large files ([f4ea792](https://github.com/loonghao/vx/commit/f4ea7921bbcbc1cf50079d98bb66fcc32fd8480e))
* **providers:** add jq provider with binary layout support ([fc9aa90](https://github.com/loonghao/vx/commit/fc9aa904c5115adcef407b100e24b06cbcd3271d))
* **resolver:** add subprocess PATH inheritance for vx-managed tools ([74b6d21](https://github.com/loonghao/vx/commit/74b6d21cde34ee5ca8f07d49d0b70a9f087596ba))
* **test:** add --ci mode for full end-to-end testing ([6576953](https://github.com/loonghao/vx/commit/657695375172d4d371a468ed519f07f12bb604f5))
* **test:** add --vx-root and --temp-root for isolated CI testing ([fde4a2f](https://github.com/loonghao/vx/commit/fde4a2fd9883bb3748d42a88de769639284d46e9))
* **test:** add comprehensive vx test command for provider testing ([b7ed2e8](https://github.com/loonghao/vx/commit/b7ed2e8afb5b723080b2fca1a2943d00c5724037))
* **vx-paths:** add debug logging for executable search ([2ebda23](https://github.com/loonghao/vx/commit/2ebda23f246906bd1b4715bdf551b425306b5e76))


### Bug Fixes

* add libc dependency for Unix-specific syscalls ([3e422ea](https://github.com/loonghao/vx/commit/3e422ea8c92bf590e166fd441ecb2c209eac53ea))
* **awscli:** use post_extract instead of post_install for MSI installation ([126d795](https://github.com/loonghao/vx/commit/126d7950187fbb32cc137e3003dd2d1f6fb46f37))
* CI issues for brew, ffmpeg, and vscode providers ([353f4f7](https://github.com/loonghao/vx/commit/353f4f7b81c407c6a484835f7972d60dbbb84ee8))
* CI test issues for vscode, docker, and rcedit ([210fcbf](https://github.com/loonghao/vx/commit/210fcbf66a52f5c643edbd46119c1212d2792ffb))
* **ci:** downgrade actions/checkout from v6 to v4 ([49dee31](https://github.com/loonghao/vx/commit/49dee3117e3a31e89c53dff6f97a1615048bde2f))
* **ci:** only skip spack on Windows, not all platforms ([e9522f4](https://github.com/loonghao/vx/commit/e9522f4fb1348660b1b704b86a954a39815bdd54))
* **ci:** pass GITHUB_TOKEN to vx test commands to avoid rate limits ([9a72d77](https://github.com/loonghao/vx/commit/9a72d7715961ab31a1efd23e1b16bd77873a6489))
* **ci:** restore corrupted workflow files and upgrade actions to v6 ([f0fbd40](https://github.com/loonghao/vx/commit/f0fbd4052cdb73d244eb2a0fc5011400b3750aa2))
* correct manifest syntax for awscli, azcli, and rust providers ([b37860d](https://github.com/loonghao/vx/commit/b37860d231c5bc5c36567754363691e4b0cb0be5))
* **deps:** update rust crate turbo-cdn to 0.8 ([9753c66](https://github.com/loonghao/vx/commit/9753c663f20038b88bf3cfcf5b8aa5290821293a))
* **deps:** update rust crate winreg to 0.55 ([67c5327](https://github.com/loonghao/vx/commit/67c53278e7f718c27ec0745ea215d6223714bad7))
* ensure Python 3.12 for pip package installation in CI ([a7e55f3](https://github.com/loonghao/vx/commit/a7e55f38b529755b5d5070509c493a1cf1b39db6))
* **executor:** add executable existence check before execution ([c625ecc](https://github.com/loonghao/vx/commit/c625ecc51a8b5510c0947551bf8a089aed5f86fd))
* handle Ctrl+C exit gracefully and fix Java download URL detection ([1168fb9](https://github.com/loonghao/vx/commit/1168fb9108a6403e1471e718018212784ab78aab))
* **jq:** use tag_prefix for version parsing ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* make test-all-providers.sh compatible with Bash 3.x (macOS) ([8332946](https://github.com/loonghao/vx/commit/8332946e70b18e69a540d4cab6d786d6bfdf7415))
* **make:** use static versions and system package manager installation ([617baf1](https://github.com/loonghao/vx/commit/617baf1899a44a3abbf3013301efa98891dd95a1))
* **make:** use success_system_installed() for verification ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* **meson:** use PackageRuntime trait for pip-based installation ([42fa897](https://github.com/loonghao/vx/commit/42fa897f935a6eb2ade1621f2bee551e79768ad0))
* **pip:** pass bin_name to install functions for correct binary verification ([f9cb7ef](https://github.com/loonghao/vx/commit/f9cb7ef67ebe01a4cfba9be92eac0c0b1ab006e7))
* prefix unused variable with underscore to fix warning ([10d576d](https://github.com/loonghao/vx/commit/10d576d4309feeed29f190932cc41a2be8a4c7ef))
* **providers:** fix rust path calculation and make Windows support ([9f636d4](https://github.com/loonghao/vx/commit/9f636d41efebd5c9094708faac507db4a2df35f8))
* **providers:** implement strip_prefix for archive extraction ([67dcabe](https://github.com/loonghao/vx/commit/67dcabef5c0967fe21b1d13467944d873b147185))
* **providers:** platform gating, npm shims, and vscode/jq layouts ([56d0bb4](https://github.com/loonghao/vx/commit/56d0bb46ded371036aa4d515d7cca8fc7ec64ccb))
* remove duplicate profile config from .cargo/config.toml ([5a8d454](https://github.com/loonghao/vx/commit/5a8d454df2ebf743a4c68df4301ee85152fc4d15))
* remove unsupported crt-static for Linux gnu targets ([f5f9847](https://github.com/loonghao/vx/commit/f5f9847d1f5d2ac6c1bf7b8301f22b70d7c9706c))
* remove useless assert!(true) to fix clippy warning ([75f8cdf](https://github.com/loonghao/vx/commit/75f8cdf8da54e7e24633291db3266b1ad3f47453))
* remove yasm/pipx providers and fix Windows test hanging ([c814502](https://github.com/loonghao/vx/commit/c81450208beee49f9c39875831411f960c30faa3))
* resolve CI failures on Windows ([6f7c5d9](https://github.com/loonghao/vx/commit/6f7c5d9d1139c32433660ccdd5b6ff532613f05d))
* resolve clippy error and pnpm test failures ([8e2e4f4](https://github.com/loonghao/vx/commit/8e2e4f4df3b316f920c106be86fb6482828c5e3c))
* resolve clippy warnings in vx-system-pm and test handler ([7824e97](https://github.com/loonghao/vx/commit/7824e97d44fe1e8e1f8f059647ebacae73d73b0a))
* resolve compilation errors and lint issues in vx-system-pm ([d97cbec](https://github.com/loonghao/vx/commit/d97cbec48d5316178830f0dd7321a00fb6cc6c93))
* resolve lint warnings and update rust tests ([8d20ea7](https://github.com/loonghao/vx/commit/8d20ea7179b9367289a3945a1402abc5a10b4bfe))
* resolve msi.rs compilation errors on non-Windows platforms ([bf67aec](https://github.com/loonghao/vx/commit/bf67aec2474342d8ff50cf843d42580600441215))
* resolve remaining clippy warnings and flaky e2e tests ([271013f](https://github.com/loonghao/vx/commit/271013fc86d570b5f6bd159de44436fb08b789f7))
* **scripts:** move function definitions before usage in PowerShell script ([92ee92e](https://github.com/loonghao/vx/commit/92ee92ef060822b88e5f746778525f0546e8867d))
* suppress dead_code warning in make provider ([ac5414e](https://github.com/loonghao/vx/commit/ac5414ed97fda20fe0cdf2d02edae0337bac7a78))
* **test:** fix command parsing for quoted arguments ([5542b86](https://github.com/loonghao/vx/commit/5542b86092afdd4557786c6d729faa56a0ae644c))
* **test:** improve test command and add progress tracking ([5a01706](https://github.com/loonghao/vx/commit/5a017067f76e5e82f718419a92f1813d8367ddcf))
* **test:** improve test command logic and add --install mode ([e14c869](https://github.com/loonghao/vx/commit/e14c8694a88c40a101c64d1f66686ebb6112a0aa))
* update boundary e2e tests to use correct CLI commands ([c675b00](https://github.com/loonghao/vx/commit/c675b0039cfca35105656969e7b82a25cc813fbc))
* update gcloud provider tests to match implementation ([f568151](https://github.com/loonghao/vx/commit/f568151027b23c8488dfaabdedcde1ad93a5978f))
* update Java provider tests to match post_extract flattening ([d0c29f0](https://github.com/loonghao/vx/commit/d0c29f02a070bbd115c14cb7469ff5deae745030))
* update project_context tests to use cache commands ([9d0e475](https://github.com/loonghao/vx/commit/9d0e475d973328990b0a6490fd310911fa6610c5))
* update Python provider test to expect 2 runtimes (python, pip) ([cc396ac](https://github.com/loonghao/vx/commit/cc396aced10a4aaa36e5bb74528512f2ae596d21))
* update tests to match current API signatures ([6f49c2b](https://github.com/loonghao/vx/commit/6f49c2b4cf297f311fd86c5924307d0ab3c47274))
* update VSCode provider test for Linux executable path ([1b37932](https://github.com/loonghao/vx/commit/1b379324bdee93b00e821a86680038b2fb05c83c))
* use 'uv self version' instead of 'uv --version'\n\nUV's version command is 'uv self version', not 'uv --version'.\nFixes CI test failures. ([9db4c09](https://github.com/loonghao/vx/commit/9db4c09260238027e916023c5bf6fb53520a87e9))
* use runtime.name() instead of runtime_name for store paths ([d29e363](https://github.com/loonghao/vx/commit/d29e3635b482bf7db64e571bec477a5bb3ab4534))
* use synchronous filesystem scan for vx tools PATH building ([db0be5d](https://github.com/loonghao/vx/commit/db0be5d6b032c881b71b9b7e0d4eb4b5c927f280))
* Windows CI shell syntax and skip problematic runtimes ([859ee2d](https://github.com/loonghao/vx/commit/859ee2da7a34cca81d614f09a66bd3ec87b42508))


### Code Refactoring

* **ci:** extract provider discovery to reusable scripts ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* **ci:** split provider tests into parallel jobs ([8a2c0b2](https://github.com/loonghao/vx/commit/8a2c0b2e122475a21bda1cffce6289fc5fb8ef8a))
* **cli:** consolidate commands and add common utilities ([a179d23](https://github.com/loonghao/vx/commit/a179d23ab19e4b85f67b552fb91d30f26c537b51))
* **cli:** redesign cache subcommands for clarity ([12c5f0d](https://github.com/loonghao/vx/commit/12c5f0d1736b04fab386af5232752f2ff67264d7))
* **rust:** use system_install strategy via rustup ([55ca72b](https://github.com/loonghao/vx/commit/55ca72bd529c7bcdc1f44a0930a2034daf5d56e7))
* simplify .cargo/config.toml following uv's approach ([968709a](https://github.com/loonghao/vx/commit/968709a02417e25ff23d584310a6b81574efd8a4))
* **vx-runtime:** split impls.rs into modular structure ([d351582](https://github.com/loonghao/vx/commit/d351582563b6960fbac1f88e067e1b035543714a))


### Documentation

* add manifest-driven providers documentation (EN/ZH) ([7cbc622](https://github.com/loonghao/vx/commit/7cbc622d15cf209b0d185e92439f47f525458143))
* add vx test command documentation (EN/ZH) ([143dd0f](https://github.com/loonghao/vx/commit/143dd0f2723189c3053e37a84493bfa7f06da757))

## [0.6.13](https://github.com/loonghao/vx/compare/vx-v0.6.12...vx-v0.6.13) (2026-01-11)


### Features

* **providers:** add nasm and yasm assembler providers ([fe6e6ba](https://github.com/loonghao/vx/commit/fe6e6bab0e6510a1d99ca12447dbb8c6bff1c56a))


### Code Refactoring

* **vx-config:** introduce TomlWriter module for safe TOML generation ([733d564](https://github.com/loonghao/vx/commit/733d564d8812bcfd53ce8431104652a0e06a1bec))

## [0.6.12](https://github.com/loonghao/vx/compare/vx-v0.6.11...vx-v0.6.12) (2026-01-10)


### Features

* improve network timeout handling and progress reporting ([a730b52](https://github.com/loonghao/vx/commit/a730b52e15cf3f2c4b1101e967b6c3e3c9b26e56))
* **manifest:** implement RFC 0018 Phase 2 user experience features ([5717bb0](https://github.com/loonghao/vx/commit/5717bb0bbad0cc26a248b77dc92285759086a17f))


### Bug Fixes

* **vx-cache:** fix clippy warnings ([d76604d](https://github.com/loonghao/vx/commit/d76604d8beaadf1bb282f7e9e3c74d743a4af9e6))


### Documentation

* fix dead links to non-existent network.md ([024f9be](https://github.com/loonghao/vx/commit/024f9beed9e6be0d1e4f50eb4a4314edcad70d66))

## [0.6.11](https://github.com/loonghao/vx/compare/vx-v0.6.10...vx-v0.6.11) (2026-01-08)


### Features

* add system tool providers and AI-native development documentation ([#368](https://github.com/loonghao/vx/issues/368)) ([5cb347f](https://github.com/loonghao/vx/commit/5cb347fffaa1f4538a9e69fb6569a6e8e3395266))


### Documentation

* add comprehensive documentation for enhanced script system ([5cb347f](https://github.com/loonghao/vx/commit/5cb347fffaa1f4538a9e69fb6569a6e8e3395266))

## [0.6.10](https://github.com/loonghao/vx/compare/vx-v0.6.9...vx-v0.6.10) (2026-01-07)


### Features

* add RFCs for platform-aware providers and system tool discovery ([65357b8](https://github.com/loonghao/vx/commit/65357b8b691704c5defb395f3607ad0783aef2ac))
* add version syntax and dependency constraints support ([c06c309](https://github.com/loonghao/vx/commit/c06c3091d71512bb04456fb81c85152d2834eef1))
* **cli:** implement RFC 0013 manifest-driven registration ([5f00f11](https://github.com/loonghao/vx/commit/5f00f117512f92db4e398e08965896be0f4a9658))
* complete architecture improvements (Phase 0-4) ([d0d9fbd](https://github.com/loonghao/vx/commit/d0d9fbd5e8ba119203ef6d06718d449f3ec3ca66))
* **manifest:** add provider.toml for all remaining providers ([d389553](https://github.com/loonghao/vx/commit/d38955345c8c996b36ac4258afb3fec3153649eb))
* **manifest:** implement provider override mechanism and add documentation ([aa0aa3f](https://github.com/loonghao/vx/commit/aa0aa3f9b59b74024b82a631be1d761f3217db11))
* **manifest:** implement RFC 0012 - Provider Manifest system ([8c23cd8](https://github.com/loonghao/vx/commit/8c23cd8c076d05d68298258c1d791c3fe99ff271))
* **runtime:** add plugin system with dynamic provider loading - Add plugin.rs with PluginLoader and ProviderLoader trait - Export plugin module in vx-runtime lib.rs - Add load_from_manifests method to ManifestRegistry - Add set_provider_loader method to ProviderRegistry - Add libloading dependency for dynamic library loading ([ec1df78](https://github.com/loonghao/vx/commit/ec1df78fe015fa73f7b984937e9c8197720c24c4))
* **runtime:** load constraints from embedded provider manifests ([ec9f933](https://github.com/loonghao/vx/commit/ec9f933db791e1bf87a782b40865f22a74af9eed))


### Bug Fixes

* correct test file to use Vec instead of arrays ([5bd8e5f](https://github.com/loonghao/vx/commit/5bd8e5fc61acd635dfe700957b04e255d699e747))
* **deps:** update rust crate libloading to 0.9 ([4593ea8](https://github.com/loonghao/vx/commit/4593ea8508d1f9afdbf5a0a9e8e0ae64d71032bc))
* properly detect system dependency versions and find bin directories ([c232ff0](https://github.com/loonghao/vx/commit/c232ff08a9b2dec5ba63d8821d6c6196800cf73c))
* remove unintended benchmark file and fix clippy error ([d3766ff](https://github.com/loonghao/vx/commit/d3766ff97010faec33491694d4ee38f72a78361a))
* use or_default() instead of or_insert_with(PathBuf::new) ([f52a2e5](https://github.com/loonghao/vx/commit/f52a2e5cff0abd446fe237f70e362dbbfbff8a40))


### Code Refactoring

* improve architecture with security warnings and unified config ([538a747](https://github.com/loonghao/vx/commit/538a7475d4253fd3a51c68dd8174edf9765035ef))


### Documentation

* add security documentation and update architecture docs ([6f7c1b6](https://github.com/loonghao/vx/commit/6f7c1b6ac36a6cd6570b4577a6e0c5db46a2a271))
* Add version management guide (EN/ZH) ([c06c309](https://github.com/loonghao/vx/commit/c06c3091d71512bb04456fb81c85152d2834eef1))
* **rfc:** add RFC 0013 - Manifest-Driven Provider Registration ([dff3ebc](https://github.com/loonghao/vx/commit/dff3ebc3ad2f328654ca4a4bc4ff6e67eb7d0627))
* **rfc:** enhance RFC 0013 with Provider vs Extension comparison and performance analysis ([1c302a5](https://github.com/loonghao/vx/commit/1c302a5631b942f184bdad86b2a7d0735ebe6852))
* update RFC status to reflect implementation progress ([9ebd753](https://github.com/loonghao/vx/commit/9ebd7533d7442fad22a8dde9b75d34861fae6fe3))

## [0.6.9](https://github.com/loonghao/vx/compare/vx-v0.6.8...vx-v0.6.9) (2026-01-06)


### Features

* **cli:** support installing multiple tools at once ([00ee20f](https://github.com/loonghao/vx/commit/00ee20fe9a939ebf229a7bdf0f1ca9d78c4ed72b))
* **msvc:** implement environment variable injection for MSVC runtime ([1fed486](https://github.com/loonghao/vx/commit/1fed486a8145108932d0d8f4455c779307283900)), closes [#353](https://github.com/loonghao/vx/issues/353)
* **resolver:** resolution cache pipeline and unified cache-mode ([4d95563](https://github.com/loonghao/vx/commit/4d955634bd06f128c1d1b425d9c4393013492ad9))
* **vx-console:** add unified console output system ([3e891b9](https://github.com/loonghao/vx/commit/3e891b9125f6a82e44838c6cd7c7b24bea0067bb))


### Bug Fixes

* **ci:** normalize version for WinGet to resolve version format issue ([70b3a33](https://github.com/loonghao/vx/commit/70b3a335fe066b3092b315bebe2795892065c3aa))
* **cli:** update tests for new multi-tool install API ([7d0d2ed](https://github.com/loonghao/vx/commit/7d0d2ed711acf832c0afde94e96a261565f0c227))
* gate msvc provider on windows ([fbf2d9d](https://github.com/loonghao/vx/commit/fbf2d9d53889c03f62bb1bc3220919d805029c12))
* provide cross-platform stub for msvc provider ([3327e92](https://github.com/loonghao/vx/commit/3327e9254b3bc00bfa3ed0936ea05525415d0b66))
* remove default() calls on unit structs for clippy lint ([c47a092](https://github.com/loonghao/vx/commit/c47a092274f09706870f5d37cca0282d1526fb14))
* resolve CI issues and improve platform-suffixed executable search ([47d3633](https://github.com/loonghao/vx/commit/47d363370879c67a630fabb37d459527cd3800de))
* resolve cl alias via msvc canonical runtime ([26ed20a](https://github.com/loonghao/vx/commit/26ed20a118a7dac2109d7faba4ba7beecb940e5f))
* resolve Windows CI test failures and formatting issues ([edbe6f6](https://github.com/loonghao/vx/commit/edbe6f617800e77148084d39e66e1b2c55010d4f))
* use vec! macro instead of push for clippy lint ([2e06ba5](https://github.com/loonghao/vx/commit/2e06ba59e6d3fd17385b84f357e6c471eb6071d5))

## [0.6.8](https://github.com/loonghao/vx/compare/vx-v0.6.7...vx-v0.6.8) (2025-12-31)


### Features

* **vx-console:** implement P0-P2 features ([7b27925](https://github.com/loonghao/vx/commit/7b2792583da235bc6d1f83b50144f8f40176f6e1))


### Bug Fixes

* **release:** fix version extraction and rate limit handling ([05c281a](https://github.com/loonghao/vx/commit/05c281a2127737a816b5094aa9115dc16e7a58cf))
* **vx-console:** add libc dependency for unix platform ([72e9a47](https://github.com/loonghao/vx/commit/72e9a472fba1a400c1cb52458950e2676bce8e0b))

## [0.6.7](https://github.com/loonghao/vx/compare/vx-v0.6.6...vx-v0.6.7) (2025-12-31)


### Features

* **cli:** integrate lock file with sync and install commands ([a93ce06](https://github.com/loonghao/vx/commit/a93ce065dda71b6d7ee6ee9ce7793a6717f47451))
* **resolver:** implement lock file mechanism ([0214aa7](https://github.com/loonghao/vx/commit/0214aa7c9b209b907239960874d709d9e81c29e4))
* **vx-console:** add unified console output system ([fb49b97](https://github.com/loonghao/vx/commit/fb49b97a74f9b0b3c411286c6df50871bce05b51))


### Bug Fixes

* add missing subdir field and ignore crates dead links in docs ([8438853](https://github.com/loonghao/vx/commit/84388535f7c7db3edde897af5ab2bfd00321dbb6))
* **awscli:** correct Linux executable path to aws/dist/aws ([5d1f84a](https://github.com/loonghao/vx/commit/5d1f84a833120047282fd4be7d52702672fa12c9))
* **cli:** move error_str into windows-only cfg block ([0892cd4](https://github.com/loonghao/vx/commit/0892cd49872b85117763e8eb60840256b82bc1e2))
* **extension:** add missing subdir field to RemoteSource tests ([6aa9da1](https://github.com/loonghao/vx/commit/6aa9da123e16bd812cdd099120cf9c85ec659dd3))


### Documentation

* **rfc:** add design limitations and future improvements analysis ([a6b75da](https://github.com/loonghao/vx/commit/a6b75da171b1543c3e46e55a05aada71896bf2e2))
* **rfc:** add mainstream Rust CLI survey (Cargo, uv, ripgrep) to RFC 0009 ([5556a63](https://github.com/loonghao/vx/commit/5556a63210df0d337c1192582d50fc791c270002))
* **rfc:** add RFC 0009 unified console output system (vx-console) ([9450f6d](https://github.com/loonghao/vx/commit/9450f6d8a021609cec719bf149741c32f015ff21))
* **rfc:** integrate vx-migration framework and RuntimeMap dependency ordering ([eb447fa](https://github.com/loonghao/vx/commit/eb447fa67538a7c2e13c7c9e1ba2f8988c8f6df6))

## [0.6.6](https://github.com/loonghao/vx/compare/vx-v0.6.5...vx-v0.6.6) (2025-12-30)


### Features

* add silent MSI installation for AWS CLI on Windows ([7f5e7af](https://github.com/loonghao/vx/commit/7f5e7af31ff371a198c7971811adabd40af157e0))


### Bug Fixes

* AWS CLI and Windows self-update improvements ([97cb866](https://github.com/loonghao/vx/commit/97cb8663e5b4802a0e7e472235ba870222020640))
* AWS CLI version list and Windows MSI handling ([01d2009](https://github.com/loonghao/vx/commit/01d200942f71a807abcbb9adf8ebb82cdfdfc62e))
* terraform API limit and add cache command ([86f726f](https://github.com/loonghao/vx/commit/86f726f0a59a8b8b8fd2511db5857c279fa61862))

## [0.6.5](https://github.com/loonghao/vx/compare/vx-v0.6.4...vx-v0.6.5) (2025-12-30)


### Features

* add pre_run hooks ([b444422](https://github.com/loonghao/vx/commit/b444422b51fce679c279edac5d43b47f2ba7aab8))
* **executor:** auto-sync uv dependencies before uv run ([75627cb](https://github.com/loonghao/vx/commit/75627cb809991af1a68cbfa2a22ef6832bc1c5d9))
* implement version solver ([ab9f99d](https://github.com/loonghao/vx/commit/ab9f99d11ac846b75587d2bd9293a0bed814029b))
* **runtime:** add pre_run hook for provider-specific setup ([e059a61](https://github.com/loonghao/vx/commit/e059a6196e88ddc1fe311ef5b1f824234cd46e7d))
* **vx-runtime:** integrate version resolver into Runtime trait ([f19fd10](https://github.com/loonghao/vx/commit/f19fd1047cfd55c8519f4591acfa4bdbee8365ce))


### Bug Fixes

* improve HTTP error messages ([58f4e40](https://github.com/loonghao/vx/commit/58f4e406698eb68464c84d4de738c7be22a1635e))
* reject invalid version strings instead of treating as latest ([fd3411a](https://github.com/loonghao/vx/commit/fd3411a42c36b38e4d0378572f5434f6cd1d26d7))


### Documentation

* update GitHub Action examples to use [@main](https://github.com/main) and add auto-dependency docs ([4fe582c](https://github.com/loonghao/vx/commit/4fe582cde993018b5e9308368b493c9141ed54e8))

## [0.6.4](https://github.com/loonghao/vx/compare/vx-v0.6.3...vx-v0.6.4) (2025-12-30)


### Bug Fixes

* **python:** update release_date for EOL versions ([b1d9de3](https://github.com/loonghao/vx/commit/b1d9de3dcd7b2dfd87f3779cbf2d4b027f1e6e60))
* **rust:** use tar.gz for Windows ([c29e480](https://github.com/loonghao/vx/commit/c29e480750298859c8293ee605f38aaa1abfb96a))

## [0.6.3](https://github.com/loonghao/vx/compare/vx-v0.6.2...vx-v0.6.3) (2025-12-30)


### Features

* **python:** add Python provider using python-build-standalone ([fc465a5](https://github.com/loonghao/vx/commit/fc465a5b8463222ef3c335ec88647e1a132fd580))


### Bug Fixes

* **ci:** correct release asset naming in Docker and package manager workflows ([625a959](https://github.com/loonghao/vx/commit/625a9598f1573ac7bd25d3da71dfcc5250a85728))
* **python:** correct filename format and add version resolution ([88ef92a](https://github.com/loonghao/vx/commit/88ef92aa0978fb8e37ffa0eef46e5c38deaf0c17))

## [0.6.2](https://github.com/loonghao/vx/compare/vx-v0.6.1...vx-v0.6.2) (2025-12-30)


### Features

* **project-analyzer:** add Electron and Tauri framework detection ([2c84210](https://github.com/loonghao/vx/commit/2c8421023f0d58efa95d270323ba85f856c4a99a))

## [0.6.1](https://github.com/loonghao/vx/compare/vx-v0.6.0...vx-v0.6.1) (2025-12-30)


### Bug Fixes

* **ci:** preserve changelog and fix OpenSSL cross-compilation ([52dfbfd](https://github.com/loonghao/vx/commit/52dfbfdb8c59999b93abbeae474aa8a74898b924))
* enable cdn-acceleration for all targets (turbo-cdn 0.6 uses rustls) ([3dab167](https://github.com/loonghao/vx/commit/3dab1675cb9b57f624bd2928d52e18bf0c34991e))
* **installer:** support versioned artifact naming format ([0449e2d](https://github.com/loonghao/vx/commit/0449e2d2b667a40c9733df31c1db283c8839b83a))

## [0.6.0](https://github.com/loonghao/vx/compare/vx-v0.5.29...vx-v0.6.0) (2025-12-30)


### ⚠ BREAKING CHANGES

* **self-update:** None - all changes are backward compatible

### Features

* add layered executable path API ([256d508](https://github.com/loonghao/vx/commit/256d5083b9c22e6ec8874f899358ffc74d1ddc2c))
* **runtime:** use indicatif for download progress and add E2E CDN tests ([7287824](https://github.com/loonghao/vx/commit/7287824f1945dca4c83b2b4036e3bb67200eef66))
* **self-update:** enhance with progress bar, checksum verification, and version selection ([2013c11](https://github.com/loonghao/vx/commit/2013c112463b5b9cd4fc3c64cfb513d46295e3f4))


### Bug Fixes

* correct rm alias test to check for Remove instead of Uninstall ([0f5f147](https://github.com/loonghao/vx/commit/0f5f147849fdaa881c6f27b8ee4f0ff00fcdf05a))
* **setup:** preserve boolean values in vx.toml when using vx add ([559fcb4](https://github.com/loonghao/vx/commit/559fcb40b7931ca1bb56b82aebda33d3bb4ec38f))
* update file_rename migration tests for current behavior ([42d3b01](https://github.com/loonghao/vx/commit/42d3b01a2683c9e49de0dadadc2fe2fd72118682))
* update migration integration tests for file-rename no-op behavior ([ed1278e](https://github.com/loonghao/vx/commit/ed1278ef8d075cb9026ddc2bfc7cab3318f90a2c))


### Code Refactoring

* **cli:** redesign add/remove commands for clarity ([6b6ccc5](https://github.com/loonghao/vx/commit/6b6ccc5a7ce4fa3bce787fb7ca41ab117d2b037f))


### Documentation

* rename .vx.toml to vx.toml across all documentation and code ([0eca931](https://github.com/loonghao/vx/commit/0eca93171ff895e8ac04d455947bda90c905292f))

## [0.5.29](https://github.com/loonghao/vx/compare/vx-v0.5.28...vx-v0.5.29) (2025-12-29)


### Features

* add vx-setup crate for setup pipeline and CI support ([63939b7](https://github.com/loonghao/vx/commit/63939b7de08ae0a663d4ddd70c13834e4d36316c))
* **extension:** phase 3 and 4 ([157fbd7](https://github.com/loonghao/vx/commit/157fbd703b22838a4281517ec79f8b22d8178b67))
* vx-args ([dace989](https://github.com/loonghao/vx/commit/dace98932dd5d76039f5febb43c20a14f4b8c41a))


### Bug Fixes

* **deps:** update rust crate turbo-cdn to 0.6 ([e864c99](https://github.com/loonghao/vx/commit/e864c994f90dd65d39aa1b506e81e9296e041b30))
* escape mustache syntax in docs for VitePress compatibility ([b453ecd](https://github.com/loonghao/vx/commit/b453ecd7c39eef0f18b4b13e824d67989ee6e2c5))
* remove dead links in docs ([ba3a15c](https://github.com/loonghao/vx/commit/ba3a15caa84745812bc265971b41d6f47cd82e60))
* remove redundant if_same_then_else in test ([81ba2fe](https://github.com/loonghao/vx/commit/81ba2fea3443aa233475e42d2dd1408425875a88))
* resolve clippy for_kv_map warning in vx-cli setup ([fb01cec](https://github.com/loonghao/vx/commit/fb01cec0a5898b173597d206c387cad62be955fe))
* resolve clippy for_kv_map warning in vx-setup ([3d2de48](https://github.com/loonghao/vx/commit/3d2de480858adff79f2fe70e287889380ba6344c))
* resolve clippy unnecessary_get_then_check in tests ([063e5ce](https://github.com/loonghao/vx/commit/063e5cef37dddfda794fc8b6657af6845aeb9400))
* resolve clippy warnings (io_other_error, for_kv_map) ([e2c7b10](https://github.com/loonghao/vx/commit/e2c7b1026847d3e8c8162ad84d24430362919ddd))
* resolve clippy warnings in vx-args parser ([ff3526e](https://github.com/loonghao/vx/commit/ff3526e52b63857e0f166d788e11c1465c30132e))

## [0.5.28](https://github.com/loonghao/vx/compare/vx-v0.5.27...vx-v0.5.28) (2025-12-29)


### Features

* add C++ analyzer and refactor language modules ([b9fdb7a](https://github.com/loonghao/vx/commit/b9fdb7a65dfdff1f2ad27c2c2a06b197f00c0efd))
* add vx-migration crate ([b34a1e0](https://github.com/loonghao/vx/commit/b34a1e03c68353c22597f9e6e360f407e364a391))


### Bug Fixes

* add missing PathBuf import in services tests ([d7eed59](https://github.com/loonghao/vx/commit/d7eed5970b65a43c4b0a03424c35116d4e52bec6))
* check both vx.toml and .vx.toml in workflow test ([2a3dafc](https://github.com/loonghao/vx/commit/2a3dafc66bab3bc25c1cfda1cd6dc0b5d6959c96))
* update tests to use vx.toml (new format) instead of .vx.toml ([9a214f6](https://github.com/loonghao/vx/commit/9a214f64c3eaad66ab27cc14b90a2d3fa2016a24))


### Code Refactoring

* **vx-paths:** centralize config file constants and discovery functions ([87dbf54](https://github.com/loonghao/vx/commit/87dbf547967e13e9fa566c0d83744557576564fa))

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
