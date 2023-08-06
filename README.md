# meshtastic-populator

This program queries a Postgres database and generates a [Leaflet](https://leafletjs.com/) based webpage with a bunch of markers.

Currently used to populate https://map.technicallyrural.com/, which looks something like this:
![image](https://github.com/tobymurray/meshtastic-populator/assets/3683198/48a229b0-3542-49f7-a7fc-1bc2988c1646)


For configuration, requires a `.env` file (or environment variables) as follows:

```
POSTGRES_DATABASE=meshtastic
POSTGRES_HOST=localhost
POSTGRES_PASSWORD=reallysecure
POSTGRES_PORT=5432
POSTGRES_USER=postgres
POSTGRES_TABLE=meshtastic_positions
```

It's designed to be invoked cron style with a script like:
```
#!/bin/bash

./meshtastic-populator

mv -f index.html public/index.html
```
