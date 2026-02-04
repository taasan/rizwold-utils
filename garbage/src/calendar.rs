//! iCalendar generator
use core::fmt;

use chrono::{
    DateTime, Datelike, Duration, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
    format::{DelayedFormat, StrftimeItems},
};

use crate::ir_client::{DisposalAddress, schedule::GarbageFraction};

use self::content_line::ContentLine;

#[inline]
fn format_naive_date<'a>(date: NaiveDate) -> DelayedFormat<StrftimeItems<'a>> {
    date.format("%Y%m%d")
}

#[inline]
fn format_timestamp<'a>(timestamp: &DateTime<Utc>) -> DelayedFormat<StrftimeItems<'a>> {
    timestamp.format("%Y%m%dT%H%M%SZ")
}

fn weekday(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Mon => "mandag",
        Tue => "tirsdag",
        Wed => "onsdag",
        Thu => "torsdag",
        Fri => "fredag",
        Sat => "lørdag",
        Sun => "søndag",
    }
}

#[derive(Debug, Clone)]
pub struct Calendar {
    fractions: Vec<GarbageFraction>,
    created: Option<DateTime<Utc>>,
    address: DisposalAddress,
}

impl Calendar {
    fn content_lines(&self) -> Vec<ContentLine> {
        self.into()
    }

    #[must_use]
    pub fn count(&self) -> usize {
        self.fractions.iter().fold(0, |acc, x| acc + x.dates.len())
    }
}

impl Calendar {
    #[must_use]
    pub const fn new(
        fractions: Vec<GarbageFraction>,
        address: DisposalAddress,
        created: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            fractions,
            created,
            address,
        }
    }
}

impl fmt::Display for Calendar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for content_line in self.content_lines() {
            content_line.fmt(f)?;
        }
        Ok(())
    }
}

mod content_line {
    use core::fmt;

    use crate::ir_client::{
        DisposalAddress,
        schedule::{DisposalDate, WasteFraction},
    };

    use super::{
        Calendar, DateTime, Datelike, Duration, Utc, format_naive_date, format_timestamp, weekday,
    };

    #[derive(Debug)]
    pub(super) struct ContentLine(String);

    impl ContentLine {
        pub const fn new() -> Self {
            Self(String::new())
        }
    }

    impl From<&str> for ContentLine {
        fn from(x: &str) -> Self {
            Self(x.to_string())
        }
    }

    impl From<String> for ContentLine {
        fn from(x: String) -> Self {
            Self(x)
        }
    }

    impl fmt::Display for ContentLine {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            if self.0.is_empty() {
                return Ok(());
            }
            let content = self.0.replace('\n', "\\n");
            let mut content = content.as_str();
            let mut boundary = next_boundary(&ContentLineToPrint::First(content));
            f.write_str(&content[..boundary])?;

            while boundary < content.len() {
                content = &content[boundary..];
                f.write_str("\r\n ")?;
                boundary = next_boundary(&ContentLineToPrint::Subsequent(content));
                f.write_str(&content[..boundary])?;
            }
            f.write_str("\r\n")
        }
    }

    #[derive(Debug)]
    struct DisposalDateEntry {
        address: DisposalAddress,
        fraction: WasteFraction,
        disposal_date: DisposalDate,
        created: Option<DateTime<Utc>>,
    }

    impl From<&DisposalDateEntry> for Vec<ContentLine> {
        fn from(value: &DisposalDateEntry) -> Self {
            let date = value.disposal_date.date;
            let dt_end = format_naive_date(date + Duration::days(1));
            let timestamp = format_timestamp(&(value.created.unwrap_or_else(Utc::now)));
            let dt_start = format_naive_date(date);
            let weekday = weekday(value.disposal_date.date);
            let day = value.disposal_date.date.day();
            let icon = value.fraction.icon();
            let name = value.fraction.name();
            let escaped_summary = escape_text(format!("{icon} {name} {weekday} {day}."));
            let uid = generate_stable_uid(value);
            let reminder_days: u8 = 3;
            vec![
                "BEGIN:VEVENT".into(),
                format!("DTEND;VALUE=DATE:{dt_end}").into(),
                format!("DTSTAMP:{timestamp}").into(),
                format!("DTSTART;VALUE=DATE:{dt_start}").into(),
                format!("SUMMARY:{escaped_summary}").into(),
                "TRANSP:TRANSPARENT".into(),
                format!("UID:{uid}").into(),
                match value.created.map(|x| x.timestamp()) {
                    Some(sequence) => format!("SEQUENCE:{sequence}").into(),
                    None => ContentLine::new(),
                },
                "URL:https://innherredrenovasjon.no/tommeplan/".into(),
                "BEGIN:VALARM".into(),
                "ACTION:DISPLAY".into(),
                format!("DESCRIPTION:{reminder_days} dager til søppel").into(),
                format!("TRIGGER:-P{reminder_days}D").into(),
                "END:VALARM".into(),
                "END:VEVENT".into(),
            ]
        }
    }

    impl From<&Calendar> for Vec<ContentLine> {
        fn from(value: &Calendar) -> Self {
            let mut res: Self = vec![
                "BEGIN:VCALENDAR".into(),
                "VERSION:2.0".into(),
                "PRODID:-//Aasan//Aasan Innherred Renovasjon//EN".into(),
                "CALSCALE:GREGORIAN".into(),
                "METHOD:PUBLISH".into(),
            ];

            res.extend(value.fractions.iter().flat_map(|x| {
                x.dates.iter().flat_map(|date| {
                    let waste_fraction: WasteFraction = x.clone().into();
                    let xs: Self = (&DisposalDateEntry {
                        fraction: waste_fraction,
                        created: value.created,
                        disposal_date: DisposalDate { date: date.date() },
                        address: value.address.clone(),
                    })
                        .into();
                    xs
                })
            }));
            res.push("END:VCALENDAR".into());
            res
        }
    }

    enum ContentLineToPrint<'a> {
        First(&'a str),
        Subsequent(&'a str),
    }

    fn generate_stable_uid(entry: &DisposalDateEntry) -> String {
        const NAMESPACE: Uuid = uuid!("769d988a-38ee-48b1-908c-5d58c0982349");
        let input_data = format!(
            "{}-{}-{}",
            entry.address,
            entry.disposal_date.date,
            entry.fraction.get_id()
        );
        let mut buf = Uuid::encode_buffer();
        let str = Uuid::new_v5(&NAMESPACE, input_data.as_bytes())
            .hyphenated()
            .encode_upper(&mut buf);
        str.to_string()
    }

    fn next_boundary(content: &ContentLineToPrint) -> usize {
        const MAX_LINE: usize = 75;
        let (content, limit) = match content {
            ContentLineToPrint::First(x) => (x, MAX_LINE),
            ContentLineToPrint::Subsequent(x) => (x, MAX_LINE - 1),
        };
        let content = content.as_bytes();
        let num_bytes = content.len();
        if limit >= num_bytes {
            return num_bytes;
        }
        match content[..=limit]
            .iter()
            .rposition(|&c| !(128..192).contains(&c))
        {
            Some(0) | None => num_bytes,
            Some(i) => i,
        }
    }

    extern crate alloc;
    use alloc::borrow::Cow;
    use uuid::{Uuid, uuid};

    /// Escapes comma, semicolon, backslash and newline character by prepending a
    /// backslash. Newlines are normalized to a line feed character.
    ///
    /// This method is only necessary for properties with the value type "TEXT".
    fn escape_text<'a, S>(input: S) -> Cow<'a, str>
    where
        S: Into<Cow<'a, str>>,
    {
        let input = input.into();
        let mut escaped_chars_count = 0;
        let mut has_carriage_return_char = false;

        for b in input.bytes() {
            if b == b',' || b == b';' || b == b'\\' || b == b'\n' {
                escaped_chars_count += 1;
            } else if b == b'\r' {
                has_carriage_return_char = true;
            }
        }

        if has_carriage_return_char || escaped_chars_count > 0 {
            let escaped_chars = |c| c == ',' || c == ';' || c == '\\' || c == '\r' || c == '\n';
            let mut output = String::with_capacity(input.len() + escaped_chars_count);
            let mut last_end = 0;
            for (start, part) in input.match_indices(escaped_chars) {
                output.push_str(&input[last_end..start]);
                match part {
                    // \r was in old MacOS versions the newline character
                    "\r" => {
                        if input.get(start + 1..start + 2) != Some("\n") {
                            output.push_str("\\n");
                        }
                    }
                    // Newlines needs to be escaped to the literal `\n`
                    "\n" => {
                        output.push_str("\\n");
                    }
                    c => {
                        output.push('\\');
                        output.push_str(c);
                    }
                }
                last_end = start + part.len();
            }
            output.push_str(&input[last_end..input.len()]);
            Cow::Owned(output)
        } else {
            input
        }
    }

    #[cfg(test)]
    mod escape_text_tests {
        use super::escape_text;

        #[test]
        fn escaped_chars() {
            let s = ",\r\n;:\\ \r\n\rö\r";
            let expected = "\\,\\n\\;:\\\\ \\n\\nö\\n";
            assert_eq!(expected, escape_text(s));
        }

        #[test]
        fn no_escaped_chars() {
            let s = "This is a simple sentence.";
            let expected = s;
            assert_eq!(expected, escape_text(s));
        }
    }
    #[test]
    fn test_output_line_display_over_75() {
        let line = ContentLine::from(
            "123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 ",
        );
        assert_eq!(
            format!("{line}"),
            String::from(
                "123456789 123456789 123456789 123456789 123456789 123456789 123456789 12345\r\n 6789 \r\n"
            )
        );
    }

    #[test]
    fn test_output_line_display_75() {
        let line = ContentLine::from(
            "123456789 123456789 123456789 123456789 123456789 123456789 123456789 12345",
        );
        assert_eq!(
            format!("{line}"),
            String::from(
                "123456789 123456789 123456789 123456789 123456789 123456789 123456789 12345\r\n"
            )
        );
    }

    #[test]
    fn test_output_line_display_wide_chars() {
        let line = ContentLine::from("A☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️");
        assert_eq!(
            format!("{line}"),
            String::from("A☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️\r\n ☣️☣️☣️☣️☣️☣️\r\n")
        );
    }

    #[test]
    fn test_output_line_display_newline() {
        let line = ContentLine::from("A\nnna");
        assert_eq!(format!("{line}"), "A\\nnna\r\n");
    }

    #[test]
    fn test_output_line_display_empty() {
        let line = ContentLine::from("");
        assert_eq!(format!("{line}"), "");
    }
}
