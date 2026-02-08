# Norwegian mailbox delivery dates calendar

A command line interface for generating an iCal version of [Bring's API for mailbox delivery dates](https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get)
or from a JSON file.

## Help

```
Usage: postgang <COMMAND>

Commands:
  api   Get delivery dates from Bring API
  file  Get delivery dates from JSON file
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Api

```
Get delivery dates from Bring API

Usage: postgang api [OPTIONS] --code <CODE> --api-uid <API_UID> --api-key <API_KEY>

Options:
      --code <CODE>        Postal code
      --output <OUTPUT>    File path, print to stdout if omitted
      --format <FORMAT>    Output format [default: ical] [possible values: ical, json]
      --api-uid <API_UID>  [env: POSTGANG_API_UID]
      --api-key <API_KEY>  [env: POSTGANG_API_KEY]
  -h, --help               Print help
```

## File

```
Get delivery dates from JSON file

Usage: postgang file [OPTIONS] --code <CODE> [INPUT]

Arguments:
  [INPUT]  File path, read from stdin of omitted

Options:
      --code <CODE>      Postal code
      --output <OUTPUT>  File path, print to stdout if omitted
      --format <FORMAT>  Output format [default: ical] [possible values: ical, json]
  -h, --help             Print help
```

## Example output

```ical
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Aasan//Aasan Postgang//EN
CALSCALE:GREGORIAN
METHOD:PUBLISH
NAME:Postgang for postnr. 7530
X-WR-CALNAME:Postgang for postnr. 7530
BEGIN:VEVENT
UID:6761F74E-84DC-5B46-9C68-54D48FD3F977
DTSTAMP:20230526T233349Z
SEQUENCE:21977128800
DTSTART;VALUE=DATE:20230206
DTEND;VALUE=DATE:20230207
SUMMARY:📬 7530: mandag 6.
TRANSP:TRANSPARENT
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
UID:2E0AC439-8BF5-5034-BBFB-E68B966E8ECE
DTSTAMP:20230526T233349Z
SEQUENCE:21977128800
DTSTART;VALUE=DATE:20230208
DTEND;VALUE=DATE:20230209
SUMMARY:📬 7530: onsdag 8.
TRANSP:TRANSPARENT
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
UID:BEEDE5C2-3A05-51E4-B52E-66F31AF0E96A
DTSTAMP:20230526T233349Z
SEQUENCE:21977128800
DTSTART;VALUE=DATE:20230210
DTEND;VALUE=DATE:20230211
SUMMARY:📬 7530: fredag 10.
TRANSP:TRANSPARENT
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
UID:7AFC12B0-6A3B-52C6-B1CF-71A1EE12CD96
DTSTAMP:20230526T233349Z
SEQUENCE:21977128800
DTSTART;VALUE=DATE:20230214
DTEND;VALUE=DATE:20230215
SUMMARY:📬 7530: tirsdag 14.
TRANSP:TRANSPARENT
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
UID:4A723381-8F93-5DBE-BC38-84EE084A4C49
DTSTAMP:20230526T233349Z
SEQUENCE:21977128800
DTSTART;VALUE=DATE:20230216
DTEND;VALUE=DATE:20230217
SUMMARY:📬 7530: torsdag 16.
TRANSP:TRANSPARENT
URL:https://www.posten.no/levering-av-post/
END:VEVENT
BEGIN:VEVENT
UID:3BAEBE78-C3C3-55E0-8AD0-F210A4884CAF
DTSTAMP:20230526T233349Z
SEQUENCE:21977128800
DTSTART;VALUE=DATE:20230220
DTEND;VALUE=DATE:20230221
SUMMARY:📬 7530: mandag 20.
TRANSP:TRANSPARENT
URL:https://www.posten.no/levering-av-post/
END:VEVENT
END:VCALENDAR
```
