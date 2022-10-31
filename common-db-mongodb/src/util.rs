use mongodb::ClientSession;
use mongodb::Collection;

pub fn get_collection<T>(db_session: &ClientSession, collection_name: &str) -> Collection<T> {
    let database = db_session
        .client()
        .default_database()
        .expect("No default db specified");

    database.collection::<T>(collection_name)
}
