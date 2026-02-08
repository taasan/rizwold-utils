use core::{fmt, num::NonZeroU8};
use std::io::Write;

use chrono::{
    DateTime, Duration, NaiveDate, Utc,
    format::{DelayedFormat, StrftimeItems},
};
use ics::{
    ICalendar,
    components::Property,
    properties::{self, CalScale, Description, Method, Name, RRule, Sequence, Summary, Transp},
};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Calendar {
    pub prodid: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub uid: uuid::Uuid,
    pub dtstamp: DateTime<Utc>,
    pub duration: NonZeroU8,
    pub rrule: Option<rrule::RRule>,
    pub rdates: Vec<NaiveDate>,
    pub exdates: Vec<NaiveDate>,
    pub sequence: i64,
    pub date: NaiveDate,
    pub summary: String,
    pub description: Option<String>,
    pub url: Option<Url>,
    pub recurrence_id: Option<NaiveDate>,
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
        // cal.push(Property::new("X-WR-TIMEZONE", "Europe/Oslo"));
        cal.push(CalScale::new("GREGORIAN"));
        cal.push(Method::new("PUBLISH"));
        if let Some(name) = &value.name {
            cal.push(Name::new(ics::escape_text(name.clone())));
            cal.push(Property::new(
                "X-WR-CALNAME",
                ics::escape_text(name.clone()),
            ));
        }
        if let Some(desc) = &value.description {
            cal.push(Description::new(ics::escape_text(desc.clone())));
            cal.push(Property::new(
                "X-WR-CALDESC",
                ics::escape_text(desc.clone()),
            ));
        }
        for e in &value.events {
            cal.add_event(e.into());
        }
        cal
    }
}

macro_rules! date_property {
    ($type:ident, $date:expr) => {{
        let mut prop = ::ics::components::Property::from(
            ::ics::properties::$type::<'_>::new($date.format("%Y%m%d").to_string())
        );
        prop.append(::ics::parameters!("VALUE" => "DATE"));
        prop
    }};
}

impl<'a> From<&'a Event> for ics::Event<'a> {
    fn from(value: &'a Event) -> Self {
        let mut e = ics::Event::new(
            format_uid(value.uid),
            format_timestamp(&value.dtstamp).to_string(),
        );
        e.push(Sequence::new(value.sequence.to_string()));
        e.push(date_property!(DtStart, value.date));
        e.push(date_property!(
            DtEnd,
            value.date + Duration::days(i64::from(value.duration.get()))
        ));
        if let Some(id) = &value.recurrence_id {
            e.push(date_property!(RecurrenceID, *id));
        }
        if let Some(rrule) = &value.rrule {
            e.push(RRule::new(rrule.to_string()));
        }
        for exdate in &value.exdates {
            e.push(date_property!(ExDate, *exdate));
        }
        for rdate in &value.rdates {
            e.push(date_property!(RDate, *rdate));
        }
        e.push(Summary::new(ics::escape_text(&value.summary)));
        e.push(Transp::transparent());
        if let Some(url) = &value.url {
            e.push(properties::URL::new(url.to_string()));
        }
        if let Some(description) = &value.description {
            e.push(Description::new(ics::escape_text(description)));
        }

        e
    }
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
#[allow(clippy::default_trait_access)]
mod test {
    use super::*;

    #[test]
    fn test_calendar_to_string() {
        let cal = Calendar {
            prodid: "-// Cal test //".to_string(),
            name: Some("Name".to_string()),
            description: Some("Description".to_string()),
            events: vec![Event {
                uid: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
                dtstamp: DateTime::from_timestamp(0, 0).unwrap(),
                date: NaiveDate::from_ymd_opt(2000, 2, 3).unwrap(),
                summary: "Summa summarum, hei; altså A☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️"
                    .to_string(),
                url: url::Url::parse("http://example.com").ok(),
                duration: NonZeroU8::MIN,
                rrule: None,
                rdates: Default::default(),
                exdates: Default::default(),
                sequence: Default::default(),
                description: Default::default(),
                recurrence_id: Default::default(),
            }],
        };
        assert_eq!(
            cal.to_string(),
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-// Cal test //\r\nCALSCALE:GREGORIAN\r\nMETHOD:PUBLISH\r\nNAME:Name\r\nX-WR-CALNAME:Name\r\nDESCRIPTION:Description\r\nX-WR-CALDESC:Description\r\nBEGIN:VEVENT\r\nUID:00000000-0000-0000-0000-000000000000\r\nDTSTAMP:19700101T000000Z\r\nSEQUENCE:0\r\nDTSTART;VALUE=DATE:20000203\r\nDTEND;VALUE=DATE:20000204\r\nSUMMARY:Summa summarum\\, hei\\; altså A☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}\r\n ☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}☣\u{fe0f}\r\nTRANSP:TRANSPARENT\r\nURL:http://example.com/\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        );
    }
}
