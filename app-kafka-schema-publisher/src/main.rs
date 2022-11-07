use std::vec;

use clap::Parser;
use kafka_schema_accommodation::schema_create_accommodation::RAW_SCHEMA_CREATE_ACCOMMODATION_V1;
use kafka_schema_accommodation::schema_create_accommodation::SCHEMA_NAME_CREATE_ACCOMMODATION;
use kafka_schema_accommodation::schema_create_room_type::RAW_SCHEMA_CREATE_ROOM_TPE_V1;
use kafka_schema_accommodation::schema_create_room_type::SCHEMA_NAME_CREATE_ROOM_TYPE;
use kafka_schema_accommodation::schema_delete_room_type::RAW_SCHEMA_DELETE_ROOM_TPE_V1;
use kafka_schema_accommodation::schema_delete_room_type::SCHEMA_NAME_DELETE_ROOM_TYPE;
use kafka_schema_accommodation::schema_update_accommodation::RAW_SCHEMA_UPDATE_ACCOMMODATION_V1;
use kafka_schema_accommodation::schema_update_accommodation::SCHEMA_NAME_UPDATE_ACCOMMODATION;
use kafka_schema_accommodation::schema_update_room_type::RAW_SCHEMA_UPDATE_ROOM_TPE_V1;
use kafka_schema_accommodation::schema_update_room_type::SCHEMA_NAME_UPDATE_ROOM_TYPE;
use kafka_schema_common::schema_key::RAW_SCHEMA_KEY;
use kafka_schema_common::schema_key::SCHEMA_NAME_KEY;
use kafka_schema_user::schema_create_user::RAW_SCHEMA_CREATE_USER_V1;
use kafka_schema_user::schema_create_user::SCHEMA_NAME_CREATE_USER;
use schema_registry_converter::blocking::schema_registry::post_schema;
use schema_registry_converter::blocking::schema_registry::SrSettings;
use schema_registry_converter::error::SRCError;
use schema_registry_converter::schema_registry_common::RegisteredSchema;
use schema_registry_converter::schema_registry_common::SchemaType;
use schema_registry_converter::schema_registry_common::SuppliedSchema;
use tracing::error;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Action to execute
    #[clap(value_enum)]
    action: PublisherAction,

    /// Environment where to apply
    #[clap(value_enum)]
    env: Environment,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum PublisherAction {
    Register,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Environment {
    Local,
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let schema_registry_url = match args.env {
        Environment::Local => "http://localhost:8081".to_owned(),
    };

    match args.action {
        PublisherAction::Register => {
            let schemas = vec![
                SchemaToRegister::new(SCHEMA_NAME_KEY, RAW_SCHEMA_KEY),
                // User - Context
                SchemaToRegister::new(SCHEMA_NAME_CREATE_USER, RAW_SCHEMA_CREATE_USER_V1),
                // Accommodation - Context
                SchemaToRegister::new(
                    SCHEMA_NAME_CREATE_ACCOMMODATION,
                    RAW_SCHEMA_CREATE_ACCOMMODATION_V1,
                ),
                SchemaToRegister::new(
                    SCHEMA_NAME_UPDATE_ACCOMMODATION,
                    RAW_SCHEMA_UPDATE_ACCOMMODATION_V1,
                ),
                SchemaToRegister::new(SCHEMA_NAME_CREATE_ROOM_TYPE, RAW_SCHEMA_CREATE_ROOM_TPE_V1),
                SchemaToRegister::new(SCHEMA_NAME_DELETE_ROOM_TYPE, RAW_SCHEMA_DELETE_ROOM_TPE_V1),
                SchemaToRegister::new(SCHEMA_NAME_UPDATE_ROOM_TYPE, RAW_SCHEMA_UPDATE_ROOM_TPE_V1),
            ];

            for schema in schemas {
                register_schema(
                    &schema_registry_url,
                    schema.subject_name,
                    schema.schema_definition,
                );
            }
        }
    }
}

fn register_schema(schema_registry_url: &str, subject_name: &str, schema_definition: &str) {
    let schema = get_schema(subject_name, schema_definition);
    print_registration_result(
        subject_name,
        register_schema_as_subject(schema_registry_url, subject_name, schema),
    );
}

fn get_schema(name: &str, schema_definition: &str) -> SuppliedSchema {
    SuppliedSchema {
        name: Some(name.to_owned()),
        schema_type: SchemaType::Avro,
        schema: schema_definition.to_owned(),
        references: vec![],
    }
}

fn register_schema_as_subject(
    registry_url: &str,
    subject: &str,
    schema: SuppliedSchema,
) -> Result<RegisteredSchema, SRCError> {
    let sr_settings = SrSettings::new_builder(registry_url.to_owned())
        .build()
        .expect("Initialization of schema registry configuration failed");

    post_schema(&sr_settings, subject.to_owned(), schema)
}

fn print_registration_result(subject_name: &str, result: Result<RegisteredSchema, SRCError>) {
    match result {
        Ok(registered_schema) => println!(
            "Registered schema \"{}\" with id: {}",
            subject_name, registered_schema.id
        ),
        Err(e) => error!("Failed to register schema \"{}\": \n{}", subject_name, e),
    }
}

struct SchemaToRegister<'a> {
    subject_name: &'a str,
    schema_definition: &'a str,
}

impl<'a> SchemaToRegister<'a> {
    fn new(name: &'a str, schema: &'a str) -> SchemaToRegister<'a> {
        Self {
            subject_name: name,
            schema_definition: schema,
        }
    }
}
