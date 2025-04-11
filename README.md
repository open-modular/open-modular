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
| [`/crates`](/crates) | Rust code (in the form of a multi-crate workspace) is contained in the [`/crates`](/crates) directory. When working with the Open Modular codebase, this should be considered the Rust "root" directory for CLI usage, etc. Cargo commands or other related actions will not work at higher levels. |
| [`/documents`](/documents) | Documentation which is not decision-related (not an ADR) can be found under the [`/documents`](/documents) directory. This is expected to be more "internal" documentation for developers, etc. rather than usage related, which will be more likely to end up in a wiki or other more documentation-oriented store. |
| [`/documents/decisions`](/documents/decisions) | Open Modular uses Architecture Decision Records (ADRs) to capture key choices made in the design and development of the project. These can be found in the [`decisions`](/documents/decisions) directory. The [`adrs`][1] tool for working with ADRs will work within this directory. See the `README.md` in [`decisions`](/documents/decisions) for more detail. |

In general, most significant locations in the repository will contain their own `README.md` file which will give more specific detail and explanation on content and usage. This includes specific crates within the [`/crates`](/crates) directory, which should all give clear expectations on their scope and implementation approaches (and potentially links to any ADRs which apply).

[0]: https://www.rust-lang.org/
[1]: https://github.com/joshrotenberg/adrs
[2]: https://nixos.org/
[3]: https://direnv.net/
