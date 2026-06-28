# Incremental Compilation in Detail - Persistent Incremental Compilation

Link: https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html

They remark that:

> **The incremental compilation scheme** is, in essence, a **surprisingly simple extension** to the overall **query system**. It relies on the fact that:
>
> 1. Queries are pure function - given the same inputs, a query will always yield the same result, and
> 2. The query model structures compilation in an acyclic graph that makes dependencies between individual computations explicit.

They briefly discuss the **red-green algorithm** for incremental query system here: https://rustc-dev-guide.rust-lang.org/queries/incremental-compilation-in-detail.html#improving-accuracy-the-red-green-algorithm.

However, I want to focus on **persistence** of incremental computation here. **Persistence** means saving the query result cache to disk so it survives across compiler invocations. Without it, incremental compilation only helps within a single compiler process - once the process exits, **all cached results are gone** and the next build starts from scratch. Persistence is what makes incremental compilation **useful in practice**: the next `cargo build` can read the cache from disk and **skip recomputing queries whose inputs have not changed**.

## Persistence Challenges

The sections above describe the underlying algorithm, however, another thing need to be addressed: **Persistence across compilation**.

- The compiler process exits when it finishes - taking the query context and its result cache with it. -> Another compilation round will have to compute from scratch.
- To make incremental compilation useful across sessions, that data must be persisted to disk, which introduces a whole new set of challenges:

**Persistence** introduces further challenges:

1. The query result cache lives on disk, so it is not readily available for change comparison when the next session starts.
2. A subsequent session starts with a new version of the code that may have arbitrary changes applied. All kinds of IDs and indices generated from a global sequential counter (e.g. `NodeId`, `DefId`) might have shifted, **making the persisted results not immediately usable because the same numeric ID might now refer to a completely different thing**.
   > Remark: I think this is the primary problem.
3. **Persisting to disk is not free**, so **not every piece of information should be cached between sessions**.
   > It's desirable that **fixed-sized, plain-old-data is preferred over complex structures** that require an expensive serialization and deserialization step.
