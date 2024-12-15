# Tooling

1. Rust
2. Wrangler
3. Yarn or npm or whatever
4. [Trunk](https://trunkrs.dev/)
5. [Taskfile](https://taskfile.dev/) 
    - not _required_ but makes life easier
6. [http-server](https://crates.io/crates/http-server)
    - install globally or adjust taskfile command
    - only used for serving media in local dev

Run `yarn install` in `workers/api`


# Configration

Note, some of these steps may require info from later steps

1. Edit [Taskfile.yml](../Taskfile.yml) to adjust paths of media and db dir
    - the defaults are probably fine, but consider moving `media` outside of the repo
2. Edit the [Frontend Config](../frontend/src/config.rs) and [Backend Config](../workers/api/src/config.rs) files with new values
    - Security precaution: make sure to change the salt
    - See [Auth docs](./AUTH.md) for more details
    - Some of these values require oauth, mailchannels, etc. setup below
3. Edit [dev.vars.example](../workers/api/dev.vars.example) with private keys, and rename to `.dev.vars` (this is secret and .gitignored)
    - This requires setting up an Oauth client (see below) and, if using a custom mail domain, `DKIM key`
4. Edit the [shared token strings](../shared/src/auth.rs) to customize your project's auth header/cookie names
5. Edit [workers.toml](../workers/api/wrangler.toml) to customize your worker name 
6. Each of the vars in [.dev.vars](../workers/api/dev.vars.example) must be added to worker secrets in `Settings -> Variables` via the dashboard too

# First-time bootstrap

### Backend

1. From within the `workers/api` directory, make sure to switch your cloudflare account
    - npx wrangler logout
    - npx wrangler login
2. Comment out D1 ids in [workers.toml](../workers/api/wrangler.toml)
3. Run `task api-deploy` to make sure you can deploy worker
4. In cloudflare dashboard, create D1 databases for prod and dev
5. Update D1 ids in [workers.toml](../workers/api/wrangler.toml) with the database ids
6. Run `task api-deploy` again to make sure D1 bindings all work okay 
7. Run `task db-migrations-apply-remote-prod` to apply the database schema to remote production database
8. Run `task db-migrations-apply-local-dev` to apply the database schema to local dev database

After this step is completed, you can get the Worker URL via going to the worker and navigating to `Settings -> Triggers`
You can also add a custom domain here (not done for this demo)

This URL is needed in both [Frontend Config](../frontend/src/config.rs) and [Backend Config](../workers/api/src/config.rs), as well as Oauth configuration

In both cases, trim the trailing `/` character.

### Frontend

1. Run `task frontend-deploy`
2. After this completes (you followed the prompts etc.), get the domain from `Pages -> Deployment`

This URL is needed in [Backend Config](../workers/api/src/config.rs) as well as oauth configuration

### Email

If you're using a custom domain, gotta set it all up with domain locking, SPF, DKIM, etc.

For non-custom domain, just domain locking in the one-time-setup above is enough

See https://developers.cloudflare.com/pages/functions/plugins/mailchannels/#enable-mailchannels-for-your-account---domain-lockdown for more details


# Cloudflare

Dashboard: https://dash.cloudflare.com/

1. Create an account, if you don't already have one
2. Setup billing (necessary for durable objects)
3. Navigate to Workers -> Plains and select paid plan (required for durable objects)

### Logging

It's easy to start real-time logging, hit the site, and dig in. Some error codes like "domain locking misconfigured" will show up in Ok responses, but with an internal message body of something like:

```
Failed to send email: 550 5.7.1 This sender is not authorized to send from example.pages.dev. See https://bit.ly/domain-lockdown. cfid=example.workers.dev
```

# Database

Migrations are in [db/migrations](../db/migrations/)

Various commands in the [Taskfile.yml](../Taskfile.yml) make it easy to create and apply new migrations to local or remote, dev or prod

# Oauth 

There's lots of guides in the wild, basically do what you need to in order to get the ClientId and ClientSecret for Google and Facebook (more providers can be easily added in code)

* Google: https://developers.google.com/identity/openid-connect/openid-connect#getcredentials
* Facebook: https://developers.facebook.com/docs/facebook-login/guides/advanced/oidc-token/

To completely authorize and take this starter code further (i.e. get profile and deeper scope info), you need to publish the app on each provider