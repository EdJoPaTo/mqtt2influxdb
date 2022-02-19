# MQTT 2 InfluxDB

> Subscribe to MQTT topics and push them to InfluxDB 1.x or v2

Something like [Telegraf](https://github.com/influxdata/telegraf) for MQTT like it does with `inputs.mqtt_consumer` and `output.influxdb`.
Telegraf has its downsides which sparked the creation of this tool.

## Features

- Telegraf uses a lot of resources. This tool is easily able to run on a Raspberry Pi 1 without any problems.
- Telegraf publishes retained messages on startup. Retained messages have happened at some point in time. Time series databases are made for exact times, not some undefined times. This tool only pushes values to InfluxDB when the time is known (right when they are published/received).
- Telegraf uses a loop every n seconds which gets the timing of MQTT messages wrong. This tool handles MQTT messages exactly when they arrive (and buffers them for better performance).
- Some devices use values like `true` or `on` which are annoying to visualize. This tool migrates values like this into `1.0` and `0.0`.
- Telegraf publishes the values with a `topic` tag. This is fine but results in a lot of regular expressions in Grafana. This tool also sets the tags `topic1`, `topic2`, … and from the end with `topic-1`, `topic-2`, … (negative numbers). For example for topic `foo/bar/test` this results in the following tags: `topic1=foo`, `topic2=bar`, `topic3=test` and from the end `topic-1=test`, `topic-2=bar` and `topic-3=foo`. Creating queries with them is way easier and probably also faster to compute for InfluxDB.

## Usage

### Command Line Arguments

```plaintext
mqtt2influxdb 0.1.0
EdJoPaTo <mqtt2influxdb-rust@edjopato.de>
Subscribe to MQTT topics and push them to InfluxDB 1.x or v2

USAGE:
    mqtt2influxdb [OPTIONS] <--influx-org <STRING>|--influx-database <STRING>> [TOPIC]...

ARGS:
    <TOPIC>...
            MQTT topics to subscribe

            [env: MQTT_TOPICS=]
            [default: #]

OPTIONS:
        --buffer-amount <INT>
            Send the buffer when the amount of messages is reached (or the time)

            [default: 1000]

        --buffer-seconds <SECONDS>
            Send the buffer when the timeout in seconds has reached (or the amount)

            [default: 28.2]

    -h, --help
            Print help information

        --influx-bucket <STRING>
            InfluxDB v2 bucket to put the values in

            [env: INFLUX_BUCKET=]
            [default: mqtt]

        --influx-database <STRING>
            InfluxDB 1.x database to put the value in

            [env: INFLUX_DATABASE=]

        --influx-host <INFLUX_HOST>
            HTTP address of InfluxDB

            [env: INFLUX_HOST=]
            [default: http://localhost:8086/]

        --influx-org <STRING>
            InfluxDB v2 organization

            [env: INFLUX_ORG=]

        --influx-token <TOKEN>
            InfluxDB api token with write access

            [env: INFLUX_TOKEN]

        --mqtt-broker <HOST>
            Host on which the MQTT Broker is running

            [env: MQTT_BROKER=]
            [default: localhost]

        --mqtt-password <STRING>
            Password to access the MQTT broker. Passing the password via command line is
            insecure as the password can be read from the history!

            [env: MQTT_PASSWORD]

        --mqtt-port <INT>
            Port on which the MQTT Broker is running

            [env: MQTT_PORT=]
            [default: 1883]

        --mqtt-user <STRING>
            Username to access the MQTT broker. Anonymous access when not supplied.

            [env: MQTT_USERNAME=]

    -v, --verbose
            Show more details

    -V, --version
            Print version information
```

## Useful Resources

- [Write with v2](https://docs.influxdata.com/influxdb/v2.1/write-data/developer-tools/api/)
- [Write with 1.x](https://docs.influxdata.com/influxdb/v2.1/reference/api/influxdb-1x/write/)
- [Line Protocol](https://docs.influxdata.com/influxdb/v2.1/reference/syntax/line-protocol/)

## Alternatives

- <https://github.com/hobbyquaker/influx4mqtt>
