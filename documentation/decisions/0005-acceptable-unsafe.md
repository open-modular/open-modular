# 5. Acceptable Unsafe

Date: 2025-03-19

## Status

Accepted

Supports: [4. Block/Lock Free Engine](0004-block-lock-free-engine.md)

## Context

The use of `unsafe` in Rust code is not without controversy still. However, high-performance/real-time code is certainly an area where it can be justifiable, as well as bindings to system interfaces. While many projects forbid unsafe code (and this is perfectly reasonable/desirable for whole categories of project), this should not be the case here.

## Decision

Unsafe code is acceptable where appropriate, particularly around hot path optimisation. The engine architecture takes advantage of this to avoid locking, for example.

## Consequences

Particular care is needed to logically verify/document that where unsafe code is used, it is *actually* safe under the constraints of normal operation. There may also be some pushback in future on the use of unsafe, but this is not of great concern. Sometimes it's needed.