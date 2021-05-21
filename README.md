# Spider #
A distributed db manager for managing all instances of replication and fragmentation of relations.

## Configure DB Instances ##
Spider needs to be aware of some configuation information from databases before it can manage relations:
- The location of the relation, ie: host, port, databse where those relations exist
- Identification credentials: username and password for the user allower to **Write** to the database instance, that is, they can execute SELECT, INSERT, DELETE, UPDATE on the relations in the database.

Spider expects these instances at "config.json" file at the application root.

### Example Configuration ###
The configuration should be in the format:
```json
{
    "configs":[
        {
            "dbcn": "PGSQL",
            "dbname": "test",
            "host": "localhost",
            "port": "5433",
            "user": "test",
            "pass": "test"
        },

        {
            "dbcn": "MySQL",
            "dbname": "test",
            "host": "localhost",
            "port": "3306",
            "user": "test",
            "pass": "test"
        }
    ]
}
```
the above config is a postgres instance example

The first configuration in the above list of configurations is treated as the default/central DB.

All the relations in the specified DB will be read into a static state, therefore any changes that don't happen through spider might be untracked. Spider needs this information to process fragmentation, read and write queries.

## Query Behaviour ##
Certain default and configureable behaviour is expected when using the system.
On instantiation with a correct config file, Spider will try to collect all relations from the specified DBs.

All ```CREATE``` queries should be passed through spider. If any new relations are created outside the system, call spider with the ```--reload``` option for it to track any new relations.

All queries (before fragmentation) are assumed to target the default db instances. Therefore a calculus query like ```INSERT <values> INTO <relation fields> WHERE <conditions>``` will insert into the first db in the specified db instances.

### The Fragment Query ###
Fragment queries are expressed in the format:
```sql
FRAGMENT [VERTICAL|HORIZONTAL] <relation> ON <conditions> TO <locations> 
```
Where ```<relation>``` is any valid relation (Either a product of a query or any named relation). Can be ```(JOIN R ON S WHERE <condition>)``` or any other form of relation.

Where ```<conditions>``` can be:
- For Vertical fragments: the conditions are comma separate groups of space seperated domains. Eg: Assuming a relation 'tickets' that has columns (Event_Name, Price, Location, Other_details) the condition would be something like: ```Event_Name Other_details, Price Location```.
- For horizontal fragments: the conditions are standard query predicates. Eg; taking the above exmaple of a 'tickets' relation, the condition would be: ``` WHERE price>1000```

And ```<locations>``` is: a group of configured db instances represented in the format ```"host:port/dbname", ...```. **Note**: These instances should represented something that's in the config.json.

#### Fragment Query Processing Behaviour ####
All produced fragments will be mapped onto the specified ```<locations>``` in the query in the order of specification.
- If the number of fragments exceeds the number of specified ```<locations>``` any excess fragments will be located on the default/central db location.
- If the number of fragments is less than the number of specified ```<locations>``` any excess locations will be ignored

Example fragment query on the earlier stated 'tickets' relation and the example db instances: 
```sql
FRAGMENT VERTICAL tickets ON Event_Name Other_details, Price Location TO "localhost:5433/test", "localhost:3306/test"
```
or 
```sql
FRAGMENT HORIZONTAL tickets on Price >= 1500 AND Price < 4000 TO "localhost:5433/test", "localhost:3306/test", "some:other/location" 
```
