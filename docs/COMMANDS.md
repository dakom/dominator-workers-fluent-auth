# Commands

Ultimately it's just `task [name]` from [Taskfile.yml](../Taskfile.yml)

The two most often used commands are:

1. `task dev`: spins up all the local dev stuff, frontend will be on http://localhost:8080, and then from there it hits media and worker on other local ports
2. `task deploy` builds and deploys everything to production

The others you're likely to need are the various d1 database manipulation tools... creating new migrations, applying them, deploying them... it's all pretty straightforward from the command names. The database migration schema is in [/db](../db) by default. This can be changed in [wrangler.toml](../workers/api/wrangler.toml)

Also note that the taskfile specifies the parent directory for wrangler to store the persistant, local, sqlite database files for development, and these variables are all configurable.