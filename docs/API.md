# Adding new API endpoints

The type and data structure of the API system is meant to ensure that the entire stack is kept in sync at compiletime.
In other words, changing any of the request, response, method, or route, must result in a compiletime fail until the changes are handled correctly in the codebase.

There is a bit of ceremony in order to create new endpoints to ensure this:

1. In `shared`
    - Create a new data structure to represent the endpoint (see [auth](../shared/src/api/auth.rs)) for example
    - implement precisely *one* of the traits from [shared api](../shared/src/api.rs)
        - the one you choose varies based on whether there is a request and response, just a request with no response, etc.
        - the trait will require choosing:
            - the request/response types (if used) as associated data types
            - the route
            - the method
2. In `frontend`
    - Nothing to do! The [extension traits](../frontend/src/api_ext.rs) automatically create all the methods
    - Specifically, it adds a `::fetch()` method that just works and is typechecked across the board
3. In `backend`
    - Implement precisely *one* of the traits from [workers api ext](../workers/api/src/api_ext.rs)
    - Make sure to set the assiociated type generically
        - For example: `type Req = <MyEndpoint as ApiBoth>::Req;` and `type Res = <MyEndpoint as ApiBoth>::Res;`
        - Once that's set, handler impl can use the concrete type for clarity (it's typechecked anyway)
    - This will vary by most of the same rules of choosing the correct trait from step 1
    - However, here there's a couple more considerations, e.g. if it's needed to mixin headers in the response type etc.
    - After choosing a trait, the core logic goes in the `handler` implementation
    - For this reason, impls usually sit in a `handler.rs`, such as [auth handler impls](../workers/api/src/auth/handler.rs)