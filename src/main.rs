#[macro_use]
extern crate diesel;

use anyhow::{format_err, Context, Error};
use diesel::{Connection, PgConnection, RunQueryDsl};
use futures03::StreamExt;
use prost::Message;
use proto::{
    module_output::Data as ModuleOutputData, BlockMetadata, BlockMetadataWrapper, BlockScopedData,
};
use std::{env, process::exit, sync::Arc};
use substreams::SubstreamsEndpoint;
use substreams_stream::{BlockResponse, SubstreamsStream};

use crate::models::NewBlockMetadata;

mod models;
mod proto;
mod schema;
mod substreams;
mod substreams_stream;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = env::args();
    if args.len() != 4 {
        println!("usage: aptos-indexer <endpoint> <spkg> <module>");
        exit(1);
    }

    let endpoint_url = env::args().nth(1).unwrap();
    let package_file = env::args().nth(2).unwrap();
    let module_name = env::args().nth(3).unwrap();

    let token_env = env::var("SUBSTREAMS_API_TOKEN").unwrap_or("".to_string());
    let mut token: Option<String> = None;
    if token_env.len() > 0 {
        token = Some(token_env);
    }

    let package = read_package(&package_file)?;
    let endpoint = Arc::new(SubstreamsEndpoint::new(&endpoint_url, token).await?);

    let conn = establish_db_connection()?;

    // FIXME: Handling of the cursor is missing here. It should be loaded from
    // the database and the SubstreamStream will correctly resume from the right
    // block.
    let cursor: Option<String> = None;

    let mut stream = SubstreamsStream::new(
        endpoint.clone(),
        cursor,
        package.modules.clone(),
        module_name.to_string(),
        0,
        100,
    );

    loop {
        match stream.next().await {
            None => {
                println!("Stream consumed");
                break;
            }
            Some(event) => match event {
                Err(_) => {}
                Ok(BlockResponse::New(data)) => {
                    println!("Consuming module output (cursor {})", data.cursor);

                    match extract_block_metadata(data, &module_name)? {
                        Some(metadata) => {
                            insert_metadata(&conn, &metadata).context("insertion in db failed")?;
                        }
                        None => {}
                    }

                    // FIXME: Handling of the cursor is missing here. It should be saved each time
                    // a full block has been correctly inserted in the database. By saving it
                    // in the database, we ensure that if we crash, on startup we are going to
                    // read it back from database and start back our SubstreamsStream with it
                    // ensuring we are continuously streaming without ever losing a single
                    // element.
                }
            },
        }
    }

    Ok(())
}

fn insert_metadata(conn: &PgConnection, metadata: &BlockMetadata) -> Result<(), Error> {
    use schema::block_metadatas;

    let timestamp = metadata.timestamp.as_ref().unwrap();

    let new_metadata = NewBlockMetadata {
        id: &metadata.id,
        round: metadata.round as i32,
        timestamp: &chrono::NaiveDateTime::from_timestamp(
            timestamp.seconds,
            timestamp.nanos as u32,
        ),
    };

    diesel::insert_into(block_metadatas::table)
        .values(&new_metadata)
        .on_conflict(block_metadatas::id)
        .do_nothing()
        .execute(conn)?;

    Ok(())
}

fn extract_block_metadata(
    data: BlockScopedData,
    module_name: &String,
) -> Result<Option<BlockMetadata>, Error> {
    let output = data
        .outputs
        .first()
        .ok_or(format_err!("expecting one module output"))?;
    if &output.name != module_name {
        return Err(format_err!(
            "invalid module output name {}, expecting {}",
            output.name,
            module_name
        ));
    }

    match output.data.as_ref().unwrap() {
        ModuleOutputData::MapOutput(data) => {
            let wrapper: BlockMetadataWrapper = Message::decode(data.value.as_slice())?;

            Ok(wrapper.metadata)
        }
        ModuleOutputData::StoreDeltas(_) => Err(format_err!(
            "invalid module output StoreDeltas, expecting MapOutput"
        )),
    }
}

fn read_package(file: &str) -> Result<proto::Package, anyhow::Error> {
    let content = std::fs::read(file).context(format_err!("read package {}", file))?;
    proto::Package::decode(content.as_ref()).context("decode command")
}

pub fn establish_db_connection() -> Result<PgConnection, Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .context(format_err!("unable to connect to database"))?;

    Ok(connection)
}
