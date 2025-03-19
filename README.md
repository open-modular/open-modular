# Open Modular

Open Modular is a modular synthesis platform written in [Rust][0]. It has several high-level aims:

* Provide an interesting and novel platform for both composition and performance using modular tools
* Provide a target for Rust-curious audio developers to experiment with relatively low friction
* Provide a platform on which higher-level audio software could potentially be built through component re-use

As an additional aim (though no less important) it aims to be a welcoming project for any developer interested in contributing, regardless of experience with Rust, etc.

## Status

Open Modular is currently pre-pre-Alpha in every sense. It is minimally capable, but a depth first approach to functionality is being taken over breadth, so it is unlikely that any particular component of the system will be productively usable in the very near future. This doesn't mean it's not worth exploring - hopefully...

## Exploring

This repository (currently) contains everything related to Open Modular, so a quick overview of where to find information, and how best to get started will hopefully provide an entrypoint until fuller documentation/guidance can be created.

| Directory | Content |
| --------- | ------- |
| [`/adr`](/adr) | Open Modular uses Architecture Decision Records (ADRs) to capture key choices made in the design and development of the project. These can be found in the [`/adr`](/adr) directory. The [`adrs`][1] tool for working with ADRs will work within this directory. See the `README.md` in [`/adr`](/adr) for more detail. |
| [`/doc`](/doc) | Documentation which is not decision-related (not an ADR) can be found under the [`/doc`](/doc) directory. This is expected to be more "internal" documentation for developers, etc. rather than usage related, which will be more likely to end up in a wiki or other more documentation-oriented store. |
| [`/env`](/env) | Environment configuration tools (currently a [`Nix`][2] flake for configuring a basic dev shell, used by the default [direnv][3] [`.envrc` file](.envrc)) is located in the [`/env`](/env) directory. |
| [`/src`](/src) | Rust code (in the form of a multi-crate workspace) is contained in the [`/src`](/src) directory. When working with the Open Modular codebase, this should be considered the Rust "root" directory for CLI usage, etc. Cargo commands or other related actions will not work at higher levels. |

[0]: https://www.rust-lang.org/
[1]: https://github.com/joshrotenberg/adrs
[2]: https://nixos.org/
[3]: https://direnv.net/
