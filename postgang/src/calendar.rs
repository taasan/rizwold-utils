//! iCalendar generator

use core::num::NonZeroU8;

use chrono::{
    DateTime, Datelike, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use url::Url;
use uuid::Uuid;

use crate::bring_client::NorwegianPostalCode;

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
    namespace: Uuid,
    delivery_dates: Vec<NaiveDate>,
    postal_code: NorwegianPostalCode,
    created: DateTime<Utc>,
    url: Url,
}

impl From<Calendar> for ::calendar::Calendar {
    fn from(calendar: Calendar) -> Self {
        Self {
            prodid: "-//Aasan//Aasan Postgang//EN".to_string(),
            events: calendar
                .delivery_dates
                .iter()
                .map(move |date| {
                    let weekday = weekday(*date);
                    let day = date.day();
                    let code = calendar.postal_code;
                    let summary = format!("📬 {code}: {weekday} {day}.");

                    ::calendar::Event {
                        uid: generate_stable_uid(calendar.namespace, calendar.postal_code, *date),
                        dtstamp: calendar.created,
                        sequence: calendar.created.timestamp(),
                        date: *date,
                        summary,
                        url: Some(calendar.url.clone()),
                        description: None,
                        duration: NonZeroU8::MIN,
                        rrule: None,
                        rdates: Vec::new(),
                        exdates: Vec::new(),
                        recurrence_id: None,
                    }
                })
                .collect(),
        }
    }
}

fn generate_stable_uid(namespace: Uuid, code: NorwegianPostalCode, date: NaiveDate) -> Uuid {
    let input_data = format!("{date}-{code}");
    Uuid::new_v5(&namespace, input_data.as_bytes())
}

impl Calendar {
    #[must_use]
    pub const fn new(
        namespace: Uuid,
        delivery_dates: Vec<NaiveDate>,
        postal_code: NorwegianPostalCode,
        created: DateTime<Utc>,
        url: Url,
    ) -> Self {
        Self {
            namespace,
            delivery_dates,
            postal_code,
            created,
            url,
        }
    }
}
