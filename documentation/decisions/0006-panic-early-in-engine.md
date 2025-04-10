# 6. Panic Early In Engine

Date: 2025-03-21

## Status

Accepted

## Context

Rust obviously provides a variety of ways for dealing with error conditions. In this case, the engine part  of the platform, which is solely concerned with processing modules in realtime, needs a decision on how it will handle failures.

## Decision

For now, the policy for the engine is that it should fail and fail fast. Long-term, the engine needs to be solid and reliable even under live use, so the sooner issues are discovered the better. There is also an expectation that the engine should only ever be instructed to do things which are expected to succeed - logical error handling should already have occurred, and any interaction with the engine is not expected to fail.

For this reason, the error handling approach for the engine is to panic, and to panic early.

## Consequences

Errors in the engine will be fatal, but should also always be seen as top priority for fixes. It is better to know about errors in the engine than have them subtly impact functionality over time.
