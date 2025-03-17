# Architectural Decision Records

Open Modular uses Architectural Decision Records (ADRs) to track key design and implementation decisions. See the [first decision record][0] for more information. If you're using Rust, the [`adrs`][1] tool is useful for creating, editing, etc. these records, and can be installed using `cargo`. This is probably more straightforward to use and up-to-date than the original tooling mentioned in the first ADR.

Pull requests for new ADRs are appropriate as a first step towards implementing a new area of functionality with any significant complexity - they form a good centre of a discussion for viable approaches, and can be updated accordingly. Editing ADRs is often valid for simple factual correction, but the meaning of an ADR reflects a decision that has been made - if it is no longer correct/valid, it should be superceded rather than changed. The [`adrs`][1] tool can help with this.

[0]: ./0001-record-architecture-decisions.md
[1]: https://github.com/joshrotenberg/adrs
