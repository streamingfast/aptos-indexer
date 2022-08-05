use super::schema::block_metadatas;
use diesel::Queryable;

#[derive(Queryable)]
pub struct BlockMetdata {
    pub id: String,
    pub round: u64,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "block_metadatas"]
pub struct NewBlockMetadata<'a> {
    pub id: &'a str,
    // I was not able to find the right type to deal with this at an higher precision,
    // I stopped searching, might be `num_bigint::BigInt`, so for now too bad.
    pub round: i32,
    pub timestamp: &'a chrono::NaiveDateTime,
}
