# meshtastic-populator

This program queries a [PostgreSQL](https://www.postgresql.org/) database and generates a [Leaflet](https://leafletjs.com/) based webpage with a bunch of markers.

Currently used to populate https://map.technicallyrural.com/, which looks something like this:
![Global Meshtastic Nodes](https://github.com/tobymurray/meshtastic-populator/assets/3683198/694f566a-ceb2-4c5e-b570-07eeaef85b89)

Expecting a schema as described in [this file](https://github.com/tobymurray/meshtastic-mqtt-harvester/blob/master/schema.sql)

For configuration, requires a `.env` file (or environment variables) as follows:

```
POSTGRES_DATABASE=meshtastic
POSTGRES_HOST=localhost
POSTGRES_PASSWORD=reallysecure
POSTGRES_PORT=5432
POSTGRES_USER=postgres
```

It's designed to be invoked cron style with a script like:
```
#!/bin/bash

./meshtastic-populator

mv -f index.html public/index.html
```
