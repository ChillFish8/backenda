use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::sync::Arc;

use scylla::{QueryResult, SessionBuilder};
use scylla::frame::value::ValueList;
use scylla::prepared_statement::PreparedStatement;
use concread::arcache::{ARCache, ARCacheBuilder};

#[derive(Clone)]
pub struct Session(Arc<scylla::Session>, Arc<ARCache<String, PreppedStmt>>);

impl From<scylla::Session> for Session {
    fn from(s: scylla::Session) -> Self {
        let cache = ARCacheBuilder::new()
            .set_size(50, num_cpus::get())
            .build()
            .unwrap();

        Self(Arc::new(s), Arc::new(cache))
    }
}

impl Session {
    pub async fn query(
        &self,
        query: &str,
        values: impl ValueList,
    ) -> anyhow::Result<QueryResult> {
        self.0.query(query, values).await.map_err(anyhow::Error::from)
    }

    pub async fn query_prepared(
        &self,
        query: &str,
        values: impl ValueList,
    ) -> anyhow::Result<QueryResult> {
        {
            let mut reader = self.1.read();
            if let Some(prep) = reader.get(query) {
                return self.0
                    .execute(prep, values)
                    .await
                    .map_err(anyhow::Error::from)
            };
        }

        let stmt = self.0.prepare(query).await?;
        let result = self.0
            .execute(&stmt, values)
            .await
            .map_err(anyhow::Error::from);

        let mut writer = self.1.write();
        writer.insert(query.to_string(), PreppedStmt::from(stmt));
        writer.commit();

        result
    }
}

#[derive(Clone)]
struct PreppedStmt(PreparedStatement);

impl Debug for PreppedStmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PreparedStmt")
    }
}

impl From<PreparedStatement> for PreppedStmt {
    fn from(v: PreparedStatement) -> Self {
        Self(v)
    }
}

impl Deref for PreppedStmt{
    type Target = PreparedStatement;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}



pub async fn connect(node: &str) -> anyhow::Result<Session> {
    let session = SessionBuilder::new()
        .known_node(node)
        .build()
        .await?;

    // session.query("CREATE KEYSPACE spooderfy WITH replication = {'class': 'SimpleStrategy', 'replication_factor' : 1};", &[]).await?;
    session.use_keyspace("spooderfy", false).await?;

    create_tables(&session).await?;

    Ok(Session::from(session))
}

async fn create_tables(session: &scylla::Session) -> anyhow::Result<()> {
    let query_block = include_str!("./scripts/tables.cql");

    for query in query_block.split("--") {
        info!("creating table {}", query.replace("\r\n", "").replace("    ", " "));
        session.query(
            query,
            &[]
        ).await?;
    }

    Ok(())
}