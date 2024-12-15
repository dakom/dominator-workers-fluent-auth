# Routing

- Routing is done by defining new enum variants in the shared crate, under [frontend](../shared/src/frontend/route.rs) or [backend](../shared/src/backend/route.rs) route modules
- The actual matching just uses Rust's native powerful pattern matching engine
    - Both static and dynamic parts
    - Supports rest arguments (via `@` syntax)
    - Has access to SearchParams for query strings if need-be
- The api authentication level required is defined per-route on the backend enum's [auth_kind()](../shared/src/backend/route.rs) method
- The compiler will enforce that every route can be converted to a url, via exhaustiveness checks 
- The compiler does not enforce that the reverse is true (i.e. that various strings can be converted into the appropriate route)
    - This _could_ be added by creating tests or a procedural macro, perhaps leveraging one of the EnumIter sortof crates out there
    - Real-world, it's not _realy_ an issue - it's added in the same file as the definition and route-to-string, and is not easily missed since the user experience breaks  entirely when trying to access the page or endpoint and it isn't found.