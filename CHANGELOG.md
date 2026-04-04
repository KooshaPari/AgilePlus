# Changelog

All notable changes to thegent are documented here.
Format: [CalVer](https://calver.org) -- `YEAR.MONTH(WAVE).PATCH`

**Visual Navigation:** Click any GIF to see the feature in action. Click [Documentation →] for detailed usage.

---

## [Unreleased]

### Features

<!-- Template for new entries:
- **scope**: Description (#PR)
  ![feature-demo](docs/assets/gifs/feature.gif)
  [Documentation →](docs/path/to/feature.md)
-->

---

## [2026.03B.0] - 2026-03-29

### Auto

- Sync and evaluate chore/dotagents-setup (#115) ([7dc0f22](https://github.com/KooshaPari/thegent/commit/7dc0f22ec6b1bc4c36148ca26a27dd420732c5d3))

### Bug Fixes

- Gate integration-only imports behind cfg(feature = "integration") ([9b5a7e3](https://github.com/KooshaPari/thegent/commit/9b5a7e39ae455b616a547411e67d6d31009db2f8))
- Remove hx-trigger load to prevent Alpine scope loss on kanban board (#20) ([3f4d245](https://github.com/KooshaPari/thegent/commit/3f4d2456c6c3d1e3e680c8603b27ed21cba3b8c4))
- Replace fake SpecKitty project with real Phenotype org repos (#21) ([0e5e13d](https://github.com/KooshaPari/thegent/commit/0e5e13d52a9f270a818a33913a5f32b3d72d23d1))
- Add missing project_for_feature and feature_counts_for_project methods (#22) ([2365c93](https://github.com/KooshaPari/thegent/commit/2365c934ef2044ba7929c8a7fe7d0d3e6d01d80f))
- **deps**: Bump time crate for security (#32) ([1885abb](https://github.com/KooshaPari/thegent/commit/1885abbeeefcfed71e0a37408a3bfc077b521b7c))
- Resolve Rust compilation errors across workspace (#33) ([08575a0](https://github.com/KooshaPari/thegent/commit/08575a07bc85d60b6e7854c35024c810f6305586))
- Resolve dependabot security alerts (#55) ([5a8d808](https://github.com/KooshaPari/thegent/commit/5a8d80888d2534b03ccc1aca6eb59c5435261536))
- Ignore RUSTSEC-2026-0049 in cargo-deny config ([13f7f4f](https://github.com/KooshaPari/thegent/commit/13f7f4f3b50058b1cd05bf727579d0f7821d94148e11c))
- **ci**: Grant pull-requests write permission to buf job ([1b2ca55](https://github.com/KooshaPari/thegent/commit/1b2ca55f14b7f4f5871955a25085c1afd04852d4))
- Resolve linting, syntax errors, and formatting issues in CI ([7b4c993](https://github.com/KooshaPari/thegent/commit/7b4c993cd5c958c86d3a272edbcc6e264688cec5))
- **ci**: Grant pull-requests write permission to buf job (#168) ([fb64a91](https://github.com/KooshaPari/thegent/commit/fb64a91e340c8d4918b193d3354e9a63c14c819b))
- Add #[default] to BacklogStatus enum for Rust 2024 ([9321827](https://github.com/KooshaPari/thegent/commit/9321827693275674f048bf81b392504152d73635))
- **dashboard**: Resolve all compiler errors in agileplus-dashboard (#186) ([953a0e7](https://github.com/KooshaPari/thegent/commit/953a0e739c8c37a8613d17ba1a052ecd790382b3))
- **dashboard**: Resolve all compiler errors in agileplus-dashboard (#187) ([0cc3b55](https://github.com/KooshaPari/thegent/commit/0cc3b55806cd31a821c0b5bb4b502544adce7f33))
- Align AgilePlus dashboard expectations and template comparisons ([84e1d9d](https://github.com/KooshaPari/thegent/commit/84e1d9da4347d0d1481048f3755aa9f692f5aada))
- **dashboard**: Populate event timeline links from WorkPackage data (#182) ([d499ac3](https://github.com/KooshaPari/thegent/commit/d499ac3eeba49b87f35fc60172b0048d231ce5f8))
- **dashboard**: Replace stub implementations with working service controls and settings (#183) ([0b1b42e](https://github.com/KooshaPari/thegent/commit/0b1b42edb0e6dc045d4e855207def6a9b335a232))
- Format agileplus dashboard hub projects block ([9f65b89](https://github.com/KooshaPari/thegent/commit/9f65b892fd7f727040b6e53e81325a02753e88db))
- **dashboard**: Replace stub implementations with working code (#193) ([dc24cf0](https://github.com/KooshaPari/thegent/commit/dc24cf0d21d186d92ce45cecb5cc0461c217f415))
- **dashboard**: Populate event timeline links from WorkPackage data (#194) ([f424f14](https://github.com/KooshaPari/thegent/commit/f424f1468fd12829a479bde05ea669fa61e57861))
- **ci**: Repair VitePress Pages and Release Drafter workflows (#196) ([4fa48ef](https://github.com/KooshaPari/thegent/commit/4fa48ef525259dcd52fd5bd364871d019ec6bbcc))
- **dashboard**: Implement service controls and agent settings UI (#198) ([4f3b19f](https://github.com/KooshaPari/thegent/commit/4f3b19f5c1d45c39da658f4f94b7b6d0d7fab196))
- Implement missing StoragePort methods in test MockStorage impls (#204) ([032b8f7](https://github.com/KooshaPari/thegent/commit/032b8f722b28824a1dcc29c0f885610dc95406d6))
- **ci**: Fix VitePress Pages deployment workflow (#205) ([59647d1](https://github.com/KooshaPari/thegent/commit/59647d1ef4690326caf038532224a82fd2268f18))
- Dashboard service control integration + safe restart command registry + governance docs ([5de7648](https://github.com/KooshaPari/thegent/commit/5de7648b610966b785a50e8cc072545c924e20fb))
- Post-merge seed bridge, MockStorage, and dashboard compilation fixes ([ebfe0ae](https://github.com/KooshaPari/thegent/commit/ebfe0ae103b7e41b343bfd33bb6db99208e6629f))
- Restore .gitignore with proper colab service exclusions ([c0a9f02](https://github.com/KooshaPari/thegent/commit/c0a9f02edb604583293c6db788bdf420cc6b120f))
- **ci**: Remove --frozen-lockfile from bun install ([bd4f349](https://github.com/KooshaPari/thegent/commit/bd4f34984724b75ee37a0142d0644d014b618eab))
- **ci**: Upload-pages-artifact v3 -> v4 ([72816b9](https://github.com/KooshaPari/thegent/commit/72816b9a5fad203e370e3c3786a9cab142b17a63))
- Update phench import paths from thegent to phench ([cdc8329](https://github.com/KooshaPari/thegent/commit/cdc8329b4c01ce11b969cff494d892ea9fc58b4f))
- Gitleaks hook grep - case-insensitive 'no leaks found' match ([bf93430](https://github.com/KooshaPari/thegent/commit/bf93430b8eb4052884343daacf4fce1586e262b2))
- Gitleaks hooks use exit code instead of grep ([f5149f0](https://github.com/KooshaPari/thegent/commit/f5149f00679cc0f87996e087ef698f888717ac9a))
- Update phench import paths from thegent to phench ([d747c6a](https://github.com/KooshaPari/thegent/commit/d747c6af8553b53aa413d72e5ccadb00d20c87b1))

### CI/CD

- Mark quality-gate.sh executable for GitHub Actions ([ed33faf](https://github.com/KooshaPari/thegent/commit/ed33faf873bd4c1eb94bd3bbdaa568e416997d4f))
- Add release-drafter config ([a96c986](https://github.com/KooshaPari/thegent/commit/a96c986f98fb2d0c1af234939f4874d9ad66e7ac))
- Remove duplicate pages-deploy workflow (deploy.yml handles VitePress) ([0e87ab1](https://github.com/KooshaPari/thegent/commit/0e87ab12b2246601a8b73402bfa1c2ddca4fc485))

### Chores

- **deps**: Bump authlib (#18) ([adbeb16](https://github.com/KooshaPari/thegent/commit/adbeb16fd7f2f6c6524d17a7dae859e9c5dd2557))
- Add dotagents for agent configuration management (#30) ([106891d](https://github.com/KooshaPari/thegent/commit/106891d6ec570100c1234be62bdae6edabf0a334))
- **deps**: Bump actions/upload-artifact from 4 to 7 (#35) ([75dc408](https://github.com/KooshaPari/thegent/commit/75dc4087b8f03f000dcd4c2d00b82bd1b300c4f7))
- **deps**: Bump actions/github-script from 7 to 8 (#50) ([9f18c87](https://github.com/KooshaPari/thegent/commit/9f18c873bd571bef1dff3d28a702c6dbcaa66594))
- Add worktrees/ and agent directories to gitignore (#52) ([1c5030c](https://github.com/KooshaPari/thegent/commit/1c5030c252f272880b235f5143a642c81eea2c76))
- **deps**: Update tonic requirement from 0.12 to 0.14 in /rust (#48) ([15d8a94](https://github.com/KooshaPari/thegent/commit/15d8a94fda334d6c99e95b2ea0b0a742f9c9e4c2))
- Dotagents setup (#43) ([91036f8](https://github.com/KooshaPari/thegent/commit/91036f870a0fa5e4a88ee504c4607e6b61d6fa61))
- **deps**: Bump actions/upload-pages-artifact from 3 to 4 (#37) ([d49def4](https://github.com/KooshaPari/thegent/commit/d49def418c2a13e78145a494c60ee5bcd1de0166))
- **deps**: Update tonic-build requirement from 0.12 to 0.14 in /rust (#49) ([630971c](https://github.com/KooshaPari/thegent/commit/630971cc16c73cd34664fa08b23b6bf3db810eee))
- **deps**: Bump actions/setup-python from 5 to 6 (#34) ([d30db20](https://github.com/KooshaPari/thegent/commit/d30db20fa0c4759696dcc339cf748eb04ae436ce))
- **deps**: Bump jdx/mise-action from 2 to 3 (#44) ([44e708a](https://github.com/KooshaPari/thegent/commit/44e708a144b5b4e2096c04ae709eebea6d337f9f))
- Modernize tooling - remove Makefiles, add Taskfile.yml, Python 3.14 ([f676b43](https://github.com/KooshaPari/thegent/commit/f676b439254bcd42b6c8f28cac4b224caf83fc3b))
- Modernize tooling - remove Makefiles, add Taskfile.yml, Python 3.14 (#63) ([8767993](https://github.com/KooshaPari/thegent/commit/8767993ad73a2b8c9000c19294ba8e7422650773))
- **deps**: Bump actions/upload-pages-artifact from 3 to 4 (#121) ([2cc7e29](https://github.com/KooshaPari/thegent/commit/2cc7e293569257c693c1e296a3a9b6630bf00ac6))
- **deps**: Bump actions/github-script from 7 to 8 (#120) ([ab5780f](https://github.com/KooshaPari/thegent/commit/ab5780f41b819b1ce84528d043bcb1d72f111c68))
- **deps**: Update tonic-build requirement from 0.12 to 0.14 in /rust (#119) ([76e5183](https://github.com/KooshaPari/thegent/commit/76e5183a5d693f6ba33713b43ede21952403d84e))
- Bump version to 0.1.1 and update CHANGELOG ([74264bb](https://github.com/KooshaPari/thegent/commit/74264bbad29fafd255fce1b6b8d8c60e4ead83e7))
- Add spec documentation (PRD, ADR, FR, PLAN, trackers) ([2589828](https://github.com/KooshaPari/thegent/commit/2589828ffce92467942034c031dddc1c7c67d220))
- Bump version to 0.1.2 and update changelog (#165) ([384a771](https://github.com/KooshaPari/thegent/commit/384a7717daf4851a6da90a844cf49a217d1d361d))
- Remove obsolete AgilePlus worktrees ([f0010ee](https://github.com/KooshaPari/thegent/commit/f0010eef93a4b6f12cd350186547c2a80c6c8b8a))
- Add cargo-deny configuration and fix security dependencies ([d5a47f6](https://github.com/KooshaPari/thegent/commit/d5a47f6740fb52ce5b03ef9e4ffa7b13ba03c254))
- Modernize tooling (#176) ([1d46cad](https://github.com/KooshaPari/thegent/commit/1d46cad7d9645d8807c53598a4a994264009643b))
- Update ADR, process docs, and kitty-specs ([5e5a054](https://github.com/KooshaPari/thegent/commit/5e5a05428fd1705d9e827e2e542a036d61060aa3))
- **specs**: Migrate kitty-specs to AgilePlus format, archive BMAD refs (#190) ([20454dc](https://github.com/KooshaPari/thegent/commit/20454dc30e0d5fc059a388a5a02cea40a8cfedb2))
- Migrate kitty-specs to docs/specs (AgilePlus format) (#203) ([0e67006](https://github.com/KooshaPari/thegent/commit/0e67006ea7e244e614b9b62305c3a0a21e0bf859))
- Migrate kitty-specs to docs/specs (AgilePlus format) ([93404b6](https://github.com/KooshaPari/thegent/commit/93404b6238f53f07338bbad6bc62e3be3f527da4))
- **wip**: Commit prior-agent seed_bridge refactor and import fix ([4b43712](https://github.com/KooshaPari/thegent/commit/4b43712e44754e478a7d7289f955acd5ab853786))
- Update docs and configs ([a74671d](https://github.com/KooshaPari/thegent/commit/a74671d11735b38bb46c88d13c152a12f6a62d97))
- Sync ([c23ab93](https://github.com/KooshaPari/thegent/commit/c23ab93ee18260fa1a2bfa3e7d8b47c60d7dadc1))
- Sync ([04bfd6e](https://github.com/KooshaPari/thegent/commit/04bfd6e4180259a447bdd5dfaae0cf77e889835a))
- Sync ([afad5ed](https://github.com/KooshaPari/thegent/commit/afad5edf68f35912fbf848f3987c5cb77e3c2111))
- Sync ([27e56ca](https://github.com/KooshaPari/thegent/commit/27e56cad7b55303262b1ef24894cbda908011f52))
- Sync ([cb38ad9](https://github.com/KooshaPari/thegent/commit/cb38ad9170859ac52cd5b4c728de853bbe0ee19d))
