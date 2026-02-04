# Innherred Renovasjon pickup dates calendar

A command line interface for generating an iCal version of IR garbage
pickup days or from a JSON file.

## Help

```
Usage: garbage <COMMAND>

Commands:
  api   Get delivery dates from Innherred Renovasjon
  file  Get delivery dates from JSON file
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Api

```
Get delivery dates from Innherred Renovasjon

Usage: garbage api [OPTIONS] --address <ADDRESS>

Options:
      --address <ADDRESS>  Address
      --output <OUTPUT>    File path, print to stdout if omitted
      --format <FORMAT>    Output format [default: ical] [possible values: ical, json]
  -h, --help               Print help
```

## File

```
Get delivery dates from JSON file

Usage: garbage file [OPTIONS] --address <ADDRESS> [INPUT]

Arguments:
  [INPUT]  File path, read from stdin of omitted

Options:
      --address <ADDRESS>  Address
      --output <OUTPUT>    File path, print to stdout if omitted
      --format <FORMAT>    Output format [default: ical] [possible values: ical, json]
  -h, --help               Print help
```

## Example output

```ical
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Aasan//Aasan Innherred Renovasjon//EN
CALSCALE:GREGORIAN
METHOD:PUBLISH
BEGIN:VEVENT
DTEND;VALUE=DATE:20260211
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20260210
SUMMARY:🍌 Matavfall tirsdag 10.
TRANSP:TRANSPARENT
UID:8C7FDF4D-5B8C-5AD4-B8EA-096A9A9B6E72
SEQUENCE:21977128800
URL:https://innherredrenovasjon.no/tommeplan/
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:3 dager til søppel
TRIGGER:-P3D
END:VALARM
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20260225
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20260224
SUMMARY:🧃 Papp/papir tirsdag 24.
TRANSP:TRANSPARENT
UID:1F5AE68B-CEA2-5E4A-8EC3-3FC6C626536D
SEQUENCE:21977128800
URL:https://innherredrenovasjon.no/tommeplan/
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:3 dager til søppel
TRIGGER:-P3D
END:VALARM
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20260225
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20260224
SUMMARY:♻️ Plastemballasje tirsdag 24.
TRANSP:TRANSPARENT
UID:562EF0B0-62E0-5D54-8016-4223DCCAE20C
SEQUENCE:21977128800
URL:https://innherredrenovasjon.no/tommeplan/
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:3 dager til søppel
TRIGGER:-P3D
END:VALARM
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20260225
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20260224
SUMMARY:🥫 Glass- og metallemballasje tirsdag 24.
TRANSP:TRANSPARENT
UID:B4ED988B-48B2-58C0-8F34-12FA1A9F28BE
SEQUENCE:21977128800
URL:https://innherredrenovasjon.no/tommeplan/
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:3 dager til søppel
TRIGGER:-P3D
END:VALARM
END:VEVENT
BEGIN:VEVENT
DTEND;VALUE=DATE:20260311
DTSTAMP:20230526T233349Z
DTSTART;VALUE=DATE:20260310
SUMMARY:🗑️ Restavfall tirsdag 10.
TRANSP:TRANSPARENT
UID:D36E7BD2-935D-5AF8-9ADA-831B31499757
SEQUENCE:21977128800
URL:https://innherredrenovasjon.no/tommeplan/
BEGIN:VALARM
ACTION:DISPLAY
DESCRIPTION:3 dager til søppel
TRIGGER:-P3D
END:VALARM
END:VEVENT
END:VCALENDAR
```
