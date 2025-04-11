# 3. Simple Modularity

Date: 2025-03-19

## Status

Accepted

## Context

Modular synthesis, particularly in terms of software engines, has many complicating choices which can be made, particularly in terms of which aspects of the historical physical model will be extended. Typically these have included such things as connection polyphony, port multi-connection, and so on, all things which couldn't generally exist in physical implementations.

## Decision

For the initial versions of Open Modular, the simplest possible model of modularity will be implemented. This means that connections between modules will be:

* Monophonic - each connection carries a single signal
* One-to-One - each port may only be connected to one other port.

Realistically, later iterations of Open Modular may remove either or both of these restrictions, but that is only likely to happen when a genuinely ergonomic and user-friendly design can be found, both for developers and end users. Unless writing polyphonic modules can be made approximately as simple as monophonic modules, the design is not yet ready.

## Consequences

This will certainly complicate patches which rely heavily on polyphony, and will increase the number of utility modules for splitting/merging etc. signals, however this is also a system where modules are effectively free, unlike a physical modular system.