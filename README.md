# stromer2mqtt

Service that connects to the Stromer e-bike cloud API and publishes the bike's data to an MQTT broker.

```
Usage: stromer2mqtt [OPTIONS] --stromer-username <STROMER_USERNAME> --stromer-password <STROMER_PASSWORD> --stromer-client-id <STROMER_CLIENT_ID> --mqtt-addr <MQTT_ADDR>

Options:
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
      --stromer-username <STROMER_USERNAME>
          [env: S2M_STROMER_USERNAME=]
      --stromer-password <STROMER_PASSWORD>
          [env: S2M_STROMER_PASSWORD=]
      --stromer-client-id <STROMER_CLIENT_ID>
          Stromer API client ID [env: S2M_STROMER_CLIENT_ID=]
      --stromer-client-secret <STROMER_CLIENT_SECRET>
          Stromer API client secret (enables v3 API) [env: S2M_STROMER_CLIENT_SECRET=]
      --stromer-bike-id <STROMER_BIKE_ID>
          Stromer bike ID (auto-detected if not provided) [env: S2M_STROMER_BIKE_ID=]
      --poll-interval <POLL_INTERVAL>
          [env: S2M_POLL_INTERVAL=] [default: 600]
      --mqtt-addr <MQTT_ADDR>
          [env: S2M_MQTT_ADDRESS=]
      --mqtt-port <MQTT_PORT>
          [env: S2M_MQTT_PORT=] [default: 1883]
      --mqtt-user <MQTT_USER>
          [env: S2M_MQTT_USER=]
      --mqtt-pass <MQTT_PASSWORD>
          [env: S2M_MQTT_PASS=]
      --mqtt-client-id <MQTT_CLIENT_ID>
          [env: S2M_CLIENT_ID=] [default: stromer2mqtt]
      --mqtt-base-topic <MQTT_BASE_TOPIC>
          [env: S2M_MQTT_BASE_TOPIC=] [default: stromer]
      --hass-discovery
          [env: S2M_HASS_DISCOVERY=]
  -h, --help
          Print help
```

### Authentication

You will need a Stromer API **client ID** (`--stromer-client-id` / `S2M_STROMER_CLIENT_ID`) in addition to your Stromer account username and password. Providing a **client secret** (`--stromer-client-secret` / `S2M_STROMER_CLIENT_SECRET`) enables the v3 API.

### Bike detection

If `--stromer-bike-id` is not specified, the bike ID is auto-detected from the first bike found on the account.

### Published data

The following sensor values are polled from the API and published as individual MQTT topics under `<base-topic>/stromer_<bike_id>/`:

| Field | Unit |
|---|---|
| Assistance Level | % |
| Atmospheric Pressure | bar |
| Average Energy Consumption | Wh |
| Average Speed Total | km/h |
| Average Speed Trip | km/h |
| Battery SOC | % |
| Battery Health | % |
| Battery Temperature | °C |
| Bike Speed | km/h |
| Motor Temperature | °C |
| Power On Cycles | — |
| Speed | km/h |
| Total Distance | km |
| Total Energy Consumption | Wh |
| Total Time | s |
| Trip Distance | km |
| Trip Time | — |
| Altitude | m |
| Receive Timestamp | — |
| Position Timestamp | — |
| Time Timestamp | — |
| SUI Version | — |
| TNT Version | — |
| Next Maintenance Date | — |
| Next Maintenance Distance | km |
| Maintenance Interval | km |
| Last Maintenance Date | — |
| Last Maintenance Distance | km |

Binary sensors: **Light**, **Lock**, **Theft**.

The bike's GPS coordinates are published as a JSON object to `<base-topic>/stromer_<bike_id>/position`.

### Commands

The service subscribes to `<base-topic>/stromer_<bike_id>/cmnd/<action>` and supports the following commands:

| Action | Payload |
|---|---|
| `lock` | `ON` / `OFF` |
| `light` | `ON` / `OFF` |

### Home Assistant discovery

Run with `--hass-discovery` (or `S2M_HASS_DISCOVERY=true`) to automatically publish MQTT discovery messages so that sensors, binary sensors, switches, and a device tracker appear in Home Assistant without any manual configuration.

### Docker

Prebuilt images are available at `dirkvdb/stromer2mqtt`.

To build the image locally using Nix:
```
nix build .#dockerImage
docker load < result
```

Example `docker-compose.yml`:
```yamlversion: "3.9"
services:
  stromer2mqtt:
    container_name: stromer2mqtt
    restart: unless-stopped
    stop_grace_period: 30s
    image: dirkvdb/stromer2mqtt:main
    environment:
      RUST_LOG: stromer2mqtt=info
      S2M_STROMER_USERNAME: <stromeracount@email.com>
      S2M_STROMER_PASSWORD: <stromerpass>
      S2M_STROMER_CLIENT_ID: <clientsecret>
      S2M_MQTT_ADDRESS: 192.168.1.2
      S2M_MQTT_USER: <mqttuser>
      S2M_MQTT_PASS: <mqttpass> 
      S2M_MQTT_BASE_TOPIC: home/stromer
      S2M_HASS_DISCOVERY: "true"
```
