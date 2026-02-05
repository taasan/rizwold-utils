//! Mailbox delivery dates API.

use core::fmt::Debug;
use std::path::PathBuf;

use chrono::{NaiveDate, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use ureq::middleware::Middleware;
use ureq::{
    Agent, Body, SendBody,
    http::{Request, Response, header::HeaderValue},
    middleware::MiddlewareNext,
};
use url::Url;
use uuid::Uuid;

use crate::bring_client::{HEADER_KEY, HEADER_UID};
use crate::calendar::Calendar;
use crate::{
    bring_client::{ApiKey, ApiUid, NORWAY, NorwegianPostalCode},
    io_error_to_string,
};

struct AuthMiddleware {
    api_key: HeaderValue,
    api_uid: HeaderValue,
}

impl Middleware for AuthMiddleware {
    fn handle(
        &self,
        mut req: Request<SendBody<'_>>,
        next: MiddlewareNext<'_>,
    ) -> Result<Response<Body>, ureq::Error> {
        req.headers_mut().insert(HEADER_KEY, self.api_key.clone());
        req.headers_mut().insert(HEADER_UID, self.api_uid.clone());
        next.handle(req)
    }
}


#[derive(Serialize, Deserialize, Debug)]
/// Represents JSON structure from the API.
pub struct ApiResponse {
    pub delivery_dates: Vec<NaiveDate>,
}

/// Delivery day provider.
pub enum DeliveryDays {
    /// Fetches JSON from [Bring API](https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get).
    // https://api.bring.com/address/api/{country-code}/postal-codes/{postal-code}/mailbox-delivery-dates
    Api(Agent),

    /// Reads JSON from a file.
    File(Option<PathBuf>),
}

impl DeliveryDays {
    /// Read dates from REST API.
    #[allow(clippy::missing_panics_doc)]
    pub fn api(api_key: ApiKey, api_uid: ApiUid) -> Self {
        // Define the middleware function
        let auth = AuthMiddleware {
            api_key: api_key.0,
            api_uid: api_uid.0,
        };
        let config = Agent::config_builder()
            .https_only(true)
            .accept("application/json")
            .middleware(auth)
            .build();
        tracing::debug!("Constructing HTTP agent with config: {config:?}");
        Self::Api(config.into())
    }

    #[must_use]
    /// Read dates from file.
    pub const fn file(path: Option<PathBuf>) -> Self {
        Self::File(path)
    }

    pub fn get_calendar(
        &self,
        postal_code: NorwegianPostalCode,
    ) -> Result<::calendar::Calendar, Box<dyn core::error::Error>> {
        const NAMESPACE: Uuid = uuid::uuid!("fa23afe5-b154-41f2-af5b-3e597f67bae6");
        let response: ApiResponse = self.get(postal_code)?;
        tracing::debug!("Got: {response:?}");
        let created = Utc::now();
        let url =
            Url::parse("https://www.posten.no/levering-av-post/").expect("Should never happen");
        let cal = Calendar::new(
            NAMESPACE,
            response.delivery_dates,
            postal_code,
            created,
            url,
        );
        let cal: ::calendar::Calendar = cal.into();
        // let fractions = response.into_values().collect();
        // let url =
        //     Url::parse("https://www.posten.no/levering-av-post/").expect("Should never happen");
        // let cal: ::calendar::Calendar =
        //     Calendar::new(NAMESPACE, fractions, address, created, url).into();
        tracing::info!("Exported {} calendar events", cal.events.len());

        Ok(cal)
    }
    /// Get a list of delivery dates.
    #[allow(clippy::missing_errors_doc)]
    pub fn get<T: DeserializeOwned>(
        &self,
        postal_code: NorwegianPostalCode,
    ) -> Result<T, Box<dyn core::error::Error>> {
        let response: T = match self {
            Self::Api(client) => {
                let url = format!(
                    "https://api.bring.com/address/api/{NORWAY}/postal-codes/{postal_code}/mailbox-delivery-dates"
                );
                tracing::debug!("Using URL: {url}");
                client.get(url).call()?.body_mut().read_json()?
            }
            Self::File(Some(path)) => {
                tracing::debug!("Reading from file: {}", path.display());
                serde_json::from_reader(
                    std::fs::File::open(path).map_err(|err| io_error_to_string(&err, path))?,
                )?
            }
            Self::File(None) => {
                tracing::debug!("Reading from stdin");
                serde_json::from_reader(std::io::stdin())?
            }
        };
        Ok(response)
    }
}
