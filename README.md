# Fullstack Rust Starter

The stack is:

* Frontend: Dominator
* Backend: Cloudflare (Workers, D1, DO, Mailchannels, ...)
* Localization: Fluent

There's no public-facing working live demo because running this in production requires:

* Paid cloudflare account (for durable objects)
* Custom domain (for Mailchannels)
* Verified Google Project (for fully working Google Signin)
* Verified Facebook App (for fully working Facebook Login)

This isn't a totally wild list of pre-requisites, but it's more than I feel like maintaining indefinitely :) 

In the meantime - this is more of a cutoff point that can be cloned and used as a foundation to build real projects.

# High-level supported features

* Compiletime API checks
    * (pseudocode) `let response = await endpoint.fetch(request)` from client is typechecked, impossible to send mismatched request or get mismatched response
    * (pseudocode) `endpoint.send_response(request)` from server is typechecked, impossible to send mismatched response, request is known
    * both of these are kept fully in sync, api changes anywhere along the stack are checked at compiletime
    * isn't (yet) macro-driven, requires implementing a trait in frontend, shared, and backend for each endpoint
* Strongly typed simple routing
    * rust's pattern matching is all we need, and it's very powerful
* Sensible responsive design and theming setup
    * dominator signals are used in certain root stylesheet properties and reusable classes
* Best-practices auth system
    * argon2 clientside
    * csrf protection
    * xss protection
    * logout everywhere capability
* Self-cleaning backend tokens
    * using durable object alarms
* First-class support for localization with Fluent engine
* Transactional emails
    * Built-in forgot password and verify email flows
* Oauth support
    * Built-in Google and Facebook support, easily extensible
* Generic setup for authenticated fetches, route protection, and so on - the foundation is there

# More docs 

* [Setup](./docs/SETUP.md)
* [Commands](./docs/COMMANDS.md)
* [Customization](./docs/CUSTOMIZATION.md)
* [Routing](./docs/ROUTING.md)
* [Adding new API endpoints](./docs/API.md)
* [Auth details](./docs/AUTH.md)

# Where to go from here

Some ideas:

* Add new workers, call them from `api`
* Have api use fluent for emails too (easy enough, just didn't get around to it)
* Build something awesome