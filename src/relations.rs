// The representation of DB relations that the connections
// module reads into. This will represent the DB 'state'
// at startup or reload.
// This section is a model of the GCS.
pub mod relations{

    #[derive(Deserialize, Clone, Debug)]
    pub struct Domain {
        // The CHECK, UNIQUE and EXCLUDE constraints
        // aren't relevant to fragmentation and are 
        // ignored
        defaut: Options<String>,
        nullable: bool,
        primary_key: bool,
        foreign_key: bool,
    }

    impl Default for Domain{
        fn default() -> Domain {
            Domain{
                default: None,
                nullable: False,
                primary_key: False,
                foreign_key: False
            }
        }
    }

    #[derive(Deserialize, Clone, Debug)]
    pub struct Relation{
        insertable: bool,
        doamins: Options<HashMap<&str, Domain>>,
    }

    #[derive(Deserialize, Clone, Debug)]
    pub struct Database {
        relations: HashMap<&str, Relation>,
    }

    pub struct gcs{
        databases: Vec<Database>,
    }

    impl Database
}