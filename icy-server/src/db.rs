use diesel::prelude::*;

#[database("measurements")]
pub struct DbConn(diesel::PgConnection);
