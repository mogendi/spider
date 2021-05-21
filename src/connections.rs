pub mod conns {
    #[allow(unused_imports)]
    use serde::Deserialize;
    use sqlx::{
        query,
        postgres::PgPoolOptions, 
        mysql::MySqlPoolOptions, 
        pool::Pool};
    use sqlx_core::row::Row;
    use std::{collections::HashMap, fs};
    use futures_util::stream::TryStreamExt;

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    enum Dbs{
        None,
        MySQL,
        PGSQL,
    }

    #[allow(dead_code)]
    #[derive(Deserialize, Clone, Debug)]
    pub struct Config{
        dbcn: Dbs,   // The DB Type
        dbname: String, // The DB name
        host: String, // Location of the DB
        port: String, // Port of the server
        user: String, // User that can access the DB
        pass: String, // User pass 
    }

    #[derive(Clone, Debug)]
    pub enum Pg { //Abstratcion for pool types
        Pgsql(Pool<sqlx::Postgres>),
        Mysql(Pool<sqlx::MySql>)
    }

    // Necessary abstraction because sqlx::Postgres
    // and sqlx::MySql dont implement serd traits
    // which are necessary for unpacking with serde_json
    #[derive(Deserialize, Clone, Debug)] 
    pub struct Configs{
        configs: Vec<Config>,
    } 

    #[derive(Clone, Debug)]
    pub struct Connections{
        configs: Configs, //All configs from <config>.json
        alive:   Option<HashMap<String, Pg>>, // All connections that are alive
        dead:    Option<HashMap<String, Pg>>, // All dead connections (couldn't be made, or no longer reachable).Unimplimented.
    }

    impl Config{
        pub fn to_url(&self) -> String{
            match self.dbcn{
                Dbs::None => {
                    panic!("Chose a DB type (PGSQL, MySQL) for the server at {}:{}"
                        , self.host, self.port)
                },
                Dbs::MySQL => {
                    format!("{}://{}:{}@{}:{}/{}", "mysql", 
                        self.user, self.pass, self.host, self.port, self.dbname )
                },
                Dbs::PGSQL => {
                    format!("{}://{}:{}@{}:{}/{}", "postgres", 
                        self.user, self.pass, self.host, self.port, self.dbname )
                },
            }
        }

        pub fn get_name(&self) -> String{
            format!("{}:{}/{}", self.host, self.port, self.dbname)
        }
    }

    impl Default for Connections{
        fn default() -> Connections {
            Connections{
                configs: Configs { configs: vec![] },
                alive:   None,
                dead:    None,
            }
        }
    }

    impl Connections {
        pub fn new(loc: &str) -> Connections{
            let cfcs: String = fs::read_to_string(loc) // read config file contents
                .expect("Couldn't read the config file");
            let cfs = serde_json::from_str::<Configs>(&cfcs[..])
                .expect("The config objects are improperly configured");
            Connections { configs: cfs, ..Connections::default() }
        }

        pub async fn connect(&mut self) -> Result<(), sqlx::Error>{
            for i in &self.configs.configs{
                match i.dbcn {
                    Dbs::None => {
                    panic!("Can't make connection. Chose a DB type (PGSQL, MySQL) for the server at {}:{}",
                        i.host, i.port)
                    }        
                    Dbs::MySQL => {
                        let pool = MySqlPoolOptions::new()
                            .max_connections(4)
                            .connect(&i.to_url()[..]).await
                            .expect(&format!("Failed to create Connection for: {}", i.get_name())[..]);
                        match &mut self.alive{
                            Some(n) => {
                                n.insert(i.get_name(), Pg::Mysql(pool));
                            },
                            None => { 
                                let mut alive_map = HashMap::new();
                                alive_map.insert(i.get_name(), Pg::Mysql(pool));
                                self.alive = Some (alive_map);
                            }
                        }
                    },
                    Dbs::PGSQL => {
                        let pool = PgPoolOptions::new()
                            .max_connections(4)
                            .connect(&i.to_url()[..]).await
                            .expect(&format!("Failed to create Connection for: {}", i.get_name())[..]);
                        match &mut self.alive{
                            Some(n) => {
                                 n.insert(i.get_name(), Pg::Pgsql(pool));
                            },
                            None => {
                                let mut alive_map = HashMap::new();
                                alive_map.insert(i.get_name(), Pg::Pgsql(pool));
                                self.alive = Some(alive_map);
                            }
                        }
                    },
                }
            }
            Ok(())
        }

        pub async fn close(&self) -> Result<(), sqlx::Error> {
            for i in &self.configs.configs {
                let con = &self.alive.as_mut().unwrap().get_mut(&i.get_name()[..]).unwrap().unwrap();
                con.close();
            }
        }

        pub fn reload(&self, loc: &str) -> Result<(), ()>{
            let cfcs: String = fs::read_to_string(loc) // read config file contents
                .expect("Couldn't read the config file");
            let cfs = serde_json::from_str::<Configs>(&cfcs[..]) // parse to Config with serde
                .expect("The config objects are improperly configured");
            if self.configs.configs.len() < cfs.configs.len() {
                // The configs state has changed
                for i in &cfs{
                    match i.dbcn {
                        Dbs::None => {
                        panic!("Can't make connection. Chose a DB type (PGSQL, MySQL) for the server at {}:{}",
                            i.host, i.port)
                        }        
                        Dbs::MySQL => {
                            let pool = MySqlPoolOptions::new()
                                .max_connections(4)
                                .connect(&i.to_url()[..]).await
                                .expect(&format!("Failed to create Connection for: {}", i.get_name())[..]);
                            match &mut self.alive{
                                Some(n) => {
                                    n.insert(i.get_name(), Pg::Mysql(pool));
                                },
                                None => { 
                                    let mut alive_map = HashMap::new();
                                    alive_map.insert(i.get_name(), Pg::Mysql(pool));
                                    self.alive = Some (alive_map);
                                }
                            }
                        },
                        Dbs::PGSQL => {
                            let pool = PgPoolOptions::new()
                                .max_connections(4)
                                .connect(&i.to_url()[..]).await
                                .expect(&format!("Failed to create Connection for: {}", i.get_name())[..]);
                            match &mut self.alive{
                                Some(n) => {
                                     n.insert(i.get_name(), Pg::Pgsql(pool));
                                },
                                None => {
                                    let mut alive_map = HashMap::new();
                                    alive_map.insert(i.get_name(), Pg::Pgsql(pool));
                                    self.alive = Some(alive_map);
                                }
                            }
                        },
                    }
                }
            }
            Ok(()) 
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn new_config() {
            let ta = Connections::new("config.json");
            assert_eq!(ta.configs.configs[0].port, "5433");
        }

        #[test]
        #[should_panic]
        fn new_config_bad_file() {
            Connections::new("Conf.json");
        }
    }
}