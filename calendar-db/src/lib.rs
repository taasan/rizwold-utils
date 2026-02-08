use std::{
    collections::HashMap,
    fs::File,
    io::{self, Write, stdout},
    path::PathBuf,
};

use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use tracing::{debug, info, trace};
use uuid::Uuid;

use crate::{
    repository::{
        Repository, WritableRepository as _,
        sqlite::{open_readonly_repository, open_writable_repository},
    },
    types::{Calendar, Event, EventException},
};

pub mod repository;
pub mod types;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Ical,
    Json,
}

#[derive(ClapParser, Debug, Default)]
pub struct OutputArg {
    /// File path, print to stdout if omitted
    #[arg(long)]
    output: Option<PathBuf>,
}

#[derive(ClapParser, Debug)]
pub struct CalendarArgs {
    /// File path, print to stdout if omitted
    #[clap(flatten)]
    output: OutputArg,

    /// Output format
    #[arg(value_enum, long, default_value_t = OutputFormat::Ical)]
    format: OutputFormat,

    /// Database id
    #[arg(long)]
    id: Uuid,
}

impl CalendarArgs {
    fn out(&self) -> Result<Box<dyn Write>, io::Error> {
        let writer: Box<dyn Write> = match &self.output.output {
            Some(path) => Box::new(File::create(path)?),
            None => Box::new(stdout().lock()),
        };
        Ok(writer)
    }
}

#[derive(ClapParser, Debug)]
pub struct DatabaseArg {
    #[arg(long, env = "RIZWOLD_CALENDAR_DB")]
    /// Path to database file
    database: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Migrate {
        #[clap(flatten)]
        database_arg: DatabaseArg,
    },
    Export {
        #[clap(flatten)]
        database_arg: DatabaseArg,
        #[clap(flatten)]
        args: CalendarArgs,
    },
    List {
        #[clap(flatten)]
        database_arg: DatabaseArg,
    },
}

fn get_calendar(repo: &impl Repository, cal_id: Uuid) -> anyhow::Result<::calendar::Calendar> {
    let mut collector = EventCollector::new();

    repo.for_each_event(Some(cal_id), |evt| {
        let evt_id = evt.id;
        let has_rrule = evt.rrule.is_some();
        debug!("Processing event {}", evt_id);
        trace!("{evt:?}");
        collector.process_event(evt);
        if has_rrule {
            repo.for_each_event_exceptions(Some(evt_id), |ex| {
                collector.process_exception(ex);
                Ok(())
            })?;
        }
        Ok(())
    })?;
    Ok(collector.finalize())
}

fn export_json(repo: &impl Repository, cal: &Calendar, out: impl Write) -> anyhow::Result<()> {
    let mut events = Vec::new();
    repo.for_each_event(Some(cal.id), |evt| {
        let mut x = (evt.clone(), Vec::new());
        repo.for_each_event_exceptions(Some(evt.id), |ex| {
            x.1.push(ex);
            Ok(())
        })?;
        events.push(x);
        Ok(())
    })?;
    let data = (cal, events);
    serde_json::ser::to_writer(out, &data)?;
    Ok(())
}

fn export(
    repo: &impl Repository,
    cal_id: Uuid,
    format: &OutputFormat,
    out: impl Write,
) -> anyhow::Result<()> {
    match repo.get_calendar(cal_id)? {
        None => Err(anyhow::format_err!("calendar not found")),
        Some(cal) => {
            debug!("Found calendar {cal:?}");
            match format {
                OutputFormat::Ical => {
                    let calendar = get_calendar(repo, cal.id)?;
                    calendar.write(out)?;
                }
                OutputFormat::Json => {
                    export_json(repo, &cal, out)?;
                }
            }
            Ok(())
        }
    }
}

impl Commands {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::missing_errors_doc)]
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Self::Migrate { database_arg } => {
                let mut repo = open_writable_repository(database_arg.database)?;
                repo.migrate()
            }
            Self::Export { database_arg, args } => {
                info!("Open database {}", database_arg.database.display());
                let repo = open_readonly_repository(database_arg.database)?;
                export(&repo, args.id, &args.format, args.out()?)
            }
            Self::List { database_arg } => {
                info!("Open database {}", database_arg.database.display());
                let repo = open_readonly_repository(database_arg.database)?;
                let mut xs: Vec<Calendar> = vec![];
                repo.for_each_calendar(|cal| {
                    xs.push(cal);
                    Ok(())
                })?;
                let out = stdout().lock();
                serde_json::ser::to_writer(out, &xs)?;
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
struct EventCollector {
    // Vi bruker en Map for raskt oppslag på master-events
    masters: HashMap<Uuid, ::calendar::Event>,
    // En liste for unntakene (som blir egne VEVENTs)
    exceptions: Vec<::calendar::Event>,
}
impl EventCollector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            masters: HashMap::new(),
            exceptions: Vec::new(),
        }
    }

    pub fn process_event(&mut self, db_event: Event) {
        // Konverterer DB-rad til domene-Event (master)
        let event = ::calendar::Event {
            uid: db_event.id,
            dtstamp: db_event.last_modified,
            date: db_event.dtstart_initial,
            summary: db_event.summary,
            description: db_event.description,
            rrule: db_event.rrule, // Allerede parset
            sequence: i64::from(db_event.sequence),
            duration: db_event.duration_days,
            rdates: Vec::new(),
            exdates: Vec::new(),
            url: db_event.url.map(Into::into),
            recurrence_id: None,
        };
        self.masters.insert(event.uid, event);
    }

    pub fn process_exception(&mut self, ex: EventException) {
        if let Some(master) = self.masters.get_mut(&ex.event_id) {
            // 1. Legg originaldatoen i masterens EXDATE
            master.exdates.push(ex.original_date);

            // 2. Hvis unntaket ikke bare er en sletting (altså har new_date eller new_summary)
            // lag et nytt VEVENT som peker tilbake til master via RECURRENCE-ID
            if ex.new_date.is_some() || ex.new_summary.is_some() {
                let mut exception_event = master.clone();

                // Overskriv verdier
                exception_event.recurrence_id = Some(ex.original_date);
                exception_event.date = ex.new_date.unwrap_or(ex.original_date);

                if let Some(s) = ex.new_summary {
                    exception_event.summary = s;
                }
                if let Some(d) = ex.new_description {
                    exception_event.description = Some(d);
                }

                // Unntak skal ikke ha RRULE selv
                exception_event.rrule = None;
                exception_event.exdates = Vec::new();

                self.exceptions.push(exception_event);
            }
        }
    }

    #[must_use]
    pub fn finalize(self) -> ::calendar::Calendar {
        let mut all_events = self.masters.into_values().collect::<Vec<_>>();
        all_events.extend(self.exceptions);
        // all_events
        ::calendar::Calendar {
            prodid: "-//Rizwold//Calendar//NO".to_string(),
            events: all_events,
        }
    }
}
