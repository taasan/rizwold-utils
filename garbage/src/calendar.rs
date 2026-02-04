//! iCalendar generator

use chrono::{
    DateTime, Datelike, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use url::Url;
use uuid::Uuid;

use crate::ir_client::{
    DisposalAddress,
    schedule::{GarbageFraction, WasteFraction},
};

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
    fractions: Vec<GarbageFraction>,
    created: DateTime<Utc>,
    address: DisposalAddress,
    url: url::Url,
}

impl From<Calendar> for ::calendar::Calendar {
    fn from(calendar: Calendar) -> Self {
        Self {
            prodid: "-//Aasan//Aasan Innherred Renovasjon//EN".to_string(),
            events: calendar
                .fractions
                .iter()
                .flat_map(move |fraction| {
                    let address = calendar.address.clone();
                    let url = calendar.url.clone();
                    let waste_fraction: WasteFraction = fraction.into();
                    let icon = waste_fraction.icon();
                    let name = waste_fraction.name();
                    fraction.dates.iter().map(move |dt| {
                        let date = dt.date();
                        let weekday = weekday(date);
                        let day = date.day();
                        let summary = format!("{icon} {name} {weekday} {day}.");

                        ::calendar::Event {
                            uid: generate_stable_uid(
                                calendar.namespace,
                                &address,
                                date,
                                &waste_fraction,
                            ),
                            dtstamp: calendar.created,
                            sequence: calendar.created.timestamp(),
                            date: dt.date(),
                            summary,
                            url: Some(url.clone()),
                        }
                    })
                })
                .collect(),
        }
    }
}

fn generate_stable_uid(
    namespace: Uuid,
    address: &DisposalAddress,
    date: NaiveDate,
    fraction: &WasteFraction,
) -> Uuid {
    let input_data = format!("{}-{}-{}", address, date, fraction.get_id());
    Uuid::new_v5(&namespace, input_data.as_bytes())
}

impl Calendar {
    #[must_use]
    pub const fn new(
        namespace: Uuid,
        fractions: Vec<GarbageFraction>,
        address: DisposalAddress,
        created: DateTime<Utc>,
        url: Url,
    ) -> Self {
        Self {
            namespace,
            fractions,
            created,
            address,
            url,
        }
    }
}
