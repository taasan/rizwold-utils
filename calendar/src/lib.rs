use core::fmt;
use std::io::Write;

use chrono::{
    DateTime, Duration, NaiveDate, Utc,
    format::{DelayedFormat, StrftimeItems},
};
use ics::{
    ICalendar,
    properties::{self, CalScale, DtEnd, DtStart, Method, Sequence, Summary, Transp},
};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Calendar {
    pub prodid: String,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub uid: uuid::Uuid,
    pub dtstamp: DateTime<Utc>,
    pub sequence: u64,
    pub date: NaiveDate,
    pub summary: String,
    pub url: Option<Url>,
}

impl Calendar {
    /// # Errors
    pub fn write<W>(&self, writer: W) -> Result<(), std::io::Error>
    where
        W: Write,
    {
        let cal: ICalendar<'_> = self.into();
        cal.write(writer)
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cal: ICalendar<'_> = self.into();
        cal.fmt(f)
    }
}

impl<'a> From<&'a Calendar> for ics::ICalendar<'a> {
    fn from(value: &'a Calendar) -> Self {
        let mut cal = ICalendar::new("2.0", &value.prodid);
        cal.push(CalScale::new("GREGORIAN"));
        cal.push(Method::new("PUBLISH"));
        for e in &value.events {
            cal.add_event(e.into());
        }
        cal
    }
}

impl<'a> From<&'a Event> for ics::Event<'a> {
    fn from(value: &'a Event) -> Self {
        let mut e = ics::Event::new(
            format_uid(value.uid),
            format_timestamp(&value.dtstamp).to_string(),
        );
        e.push(Sequence::new(value.sequence.to_string()));
        e.push(dt_start(value.date));
        e.push(dt_end(value.date + Duration::days(1)));
        e.push(Summary::new(ics::escape_text(&value.summary)));
        e.push(Transp::transparent());
        if let Some(url) = &value.url {
            e.push(properties::URL::new(url.to_string()));
        }

        e
    }
}

fn dt_start<'a>(date: NaiveDate) -> DtStart<'a> {
    let mut d = DtStart::new(format_naive_date(date).to_string());
    d.append(ics::parameters!("VALUE" => "DATE"));
    d
}

fn dt_end<'a>(date: NaiveDate) -> DtEnd<'a> {
    let mut d = DtEnd::new(format_naive_date(date).to_string());
    d.append(ics::parameters!("VALUE" => "DATE"));
    d
}

#[inline]
fn format_naive_date<'a>(date: NaiveDate) -> DelayedFormat<StrftimeItems<'a>> {
    date.format("%Y%m%d")
}

#[inline]
fn format_timestamp<'a>(timestamp: &DateTime<Utc>) -> DelayedFormat<StrftimeItems<'a>> {
    timestamp.format("%Y%m%dT%H%M%SZ")
}

#[inline]
fn format_uid(uid: uuid::Uuid) -> String {
    let mut buf = Uuid::encode_buffer();
    uid.hyphenated().encode_upper(&mut buf).to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod test {
    use super::*;

    #[test]
    fn test_calendar_to_string() {
        let cal = Calendar {
            prodid: "-// Cal test //".to_string(),
            events: vec![Event {
                uid: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
                dtstamp: DateTime::from_timestamp(0, 0).unwrap(),
                sequence: 1,
                date: NaiveDate::from_ymd_opt(2000, 2, 3).unwrap(),
                summary: "Summa summarum, hei; altså A☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️"
                    .to_string(),
                url: url::Url::parse("http://example.com").ok(),
            }],
        };
        assert_eq!(
            cal.to_string(),
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-// Cal test //\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nBEGIN:VEVENT\r\nUID:00000000-0000-0000-0000-000000000000\r\nDTSTAMP:19700101T000000Z\r\nSEQUENCE:1\r\nDTSTART;VALUE=DATE:20000203\r\nDTEND;VALUE=DATE:20000204\r\nSUMMARY:Summa summarum\\, hei\\; altså A☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}\r\n ☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}\r\nTRANSP:TRANSPARENT\r\nURL:http://example.com/\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        );
    }
}
