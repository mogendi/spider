mod connections;
use connections::conns::Connections;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error>{
    let mut c = Connections::new("config.json");
    match c.connect().await {
        Ok(n) => {
            c.read().await?;
            Ok(())
        },
        Err(e) => Err(e)
    }
}
