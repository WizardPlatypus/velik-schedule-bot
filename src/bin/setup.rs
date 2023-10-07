use schedule_bot::data::{unpack, Schedule, Subject};
use sqlx::SqlitePool as Pool;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // could've used Clap, but I literally need just 1 argument
    let mut args = std::env::args();
    args.next(); // skip the first element
    let url = args.next().expect("Missing database url");
    let pool = Pool::connect(&url)
        .await
        .expect("Failed to connect to database");

    let db = schedule_bot::db::Database::new(pool);

    let subjects: Vec<Subject> = unpack("data/subjects.packed", 4).unwrap();
    log::trace!("Read subjects.packed");
    let schedule: Vec<Schedule> = unpack("data/schedule.packed", 4).unwrap();
    log::trace!("Read schedule.packed");

    for subject in subjects {
        if let Err(error) = db.add_subject(&subject).await {
            log::error!("Failed to add {:?} to db: {:?}", subject, error);
        }
    }
    log::trace!("Written subjects to db");

    for record in schedule {
        if let Err(error) = db.add_schedule(&record).await {
            log::error!("Failed to add {:?} to db: {:?}", record, error);
        }
    }
    log::trace!("Written schedule to db");
}
