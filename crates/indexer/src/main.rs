use std::env;

use sea_orm::{Database, DatabaseConnection};
use solana_indexer::{
    CbResult, ExecutorCallback, ExecutorControlFlow, Indexer, IndexerEngine, Instruction,
    TxSignature,
};

mod instructions;

pub struct SolaIndexer {
    db: DatabaseConnection,
}

impl ExecutorCallback for SolaIndexer {
    async fn process_instruction(&mut self, instruction: &Instruction) -> CbResult {
        instructions::handle_instruction(instruction, &self.db).await?;
        Ok(().into())
    }

    async fn process_signature(&mut self, tx: &TxSignature) -> CbResult {
        if tx.err.is_some() {
            return Ok(ExecutorControlFlow::Skip);
        }

        Ok(().into())
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_test_writer()
        .init();
    let database_url: String =
        env::var("DATABASE_URL").expect("DATABASE_URL environment variable not set");

    let database = Database::connect(&*database_url)
        .await
        .expect("Fail to initialize database connection");

    let processor = SolaIndexer { db: database };
    let mut solana_indexer = Indexer::build().await.unwrap();
    solana_indexer.set_executor(processor);
    solana_indexer.start_indexing().await.unwrap();
}
