# 4. Block/Lock Free Engine

Date: 2025-03-19

## Status

Accepted

Relevant to implementation: [5. Acceptable Unsafe](0005-acceptable-unsafe.md)

## Context

This is relatively common to all audio programming, but care will be taken to elminate/minimise potentially blocking operations on the hot path.

## Decision

A specific architecture (see [Architecture](../doc/ARCHITECTURE.md)) will be developed which minimises blocking operations, and controls when different categories of operation happen. This may be subtly different to traditional approaches to a) take advantage of the capabilities of Rust and b) experiment with relaxing some of the historic prohibitions where they can be constrained/made predictable by implementation.

## Consequences

The core implementation will take care and alignment to the architectural approach, and the approach must be validated through testing, etc.