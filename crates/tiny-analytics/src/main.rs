use datafusion::error::Result;
use datafusion::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let ctx = SessionContext::new();

    ctx.register_parquet("test", "data/test.parquet", ParquetReadOptions::default())
        .await?;

    let df = ctx
        .sql(
            "SELECT region, SUM(amount) AS total
             FROM test
             GROUP BY region
             ORDER BY region",
        )
        .await?;

    // ctx.sql(
    //     "COPY test TO 'data/test.parquet'
    //  STORED AS PARQUET",
    // )
    // .await?
    // .collect()
    // .await?;

    df.show().await?;

    Ok(())
}
