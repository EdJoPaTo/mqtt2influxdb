# MQTT 2 InfluxDB

> Subscribe to MQTT topics and push them to InfluxDB 1.x or v2

Something like [Telegraf](https://github.com/influxdata/telegraf) for MQTT like it does with `inputs.mqtt_consumer` and `output.influxdb`.
Telegraf has its downsides which sparked the creation of this tool.

## Features

- Telegraf uses a lot of resources. This tool is easily able to run on a Raspberry Pi 1 without any problems.
- Telegraf publishes retained messages on startup. Retained messages have happened at some point in time. Time series databases are made for exact times, not some undefined times. This tool only pushes values to InfluxDB when the time is known (right when they are published/received).
- Telegraf uses a loop every n seconds which gets the timing of MQTT messages wrong. This tool handles MQTT messages exactly when they arrive (and buffers them for better performance).
- Some devices use values like `true` or `on` which are annoying to visualize. This tool migrates values like this into `1.0` and `0.0`.
- Telegraf publishes the values with a `topic` tag. This is fine but results in a lot of regular expressions in Grafana. This tool also sets the tags `topic1`, `topic2`, …; from the end with `topicE1`, `topicE2`, … and `topicSegments` for the amount of segments. For example for topic `foo/bar/test` this results in the following tags: `topic1=foo`, `topic2=bar`, `topic3=test`, from the end `topicE1=test`, `topicE2=bar`, `topicE3=foo` and `topicSegments=3`. Creating queries with them is way easier compared to regex queries and probably also faster to compute for InfluxDB.

## Usage

Run `mqtt2influxdb --help`.

## Useful Resources

- [Write with v2](https://docs.influxdata.com/influxdb/v2.1/write-data/developer-tools/api/)
- [Write with 1.x](https://docs.influxdata.com/influxdb/v2.1/reference/api/influxdb-1x/write/)
- [Line Protocol](https://docs.influxdata.com/influxdb/v2.1/reference/syntax/line-protocol/)
- [Migrate from InfluxDB to VictoriaMetrics](https://docs.victoriametrics.com/guides/migrate-from-influx.html)

## Alternatives

- <https://github.com/hobbyquaker/influx4mqtt>
