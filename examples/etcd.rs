use anyhow::Result;
use etcd_client::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::connect(["localhost:2379"], None).await?;

    let resp = client.lease_grant(30, None).await?;
    println!(
        "创建租约 grant a lease with id {:?}, ttl {:?}",
        resp.id(),
        resp.ttl()
    );
    let id = resp.id();

    // put kv
    let resp = client
        .put("name", "zhangsan", Some(PutOptions::new().with_lease(id)))
        .await?;
    println!("{:?}", resp);

    let resp = client.lease_time_to_live(id, None).await?;
    println!(
        "租约剩余时间 lease({:?}) remain ttl {:?} granted ttl {:?}",
        resp.id(),
        resp.ttl(),
        resp.granted_ttl()
    );

    client.put("name", "lisi", None).await?;

    let resp = client.get("name", None).await?;
    if let Some(kv) = resp.kvs().first() {
        println!("Get kv: {}: {}", kv.key_str()?, kv.value_str()?);
    }

    let resp = client
        .get("name", Some(GetOptions::new().with_prefix()))
        .await?;
    if let Some(kv) = resp.kvs().first() {
        println!("Get kv: {}: {}", kv.key_str()?, kv.value_str()?);
    }

    println!("Get all users:");
    let resp = client
        .get("name", Some(GetOptions::new().with_all_keys()))
        .await?;
    if let Some(kv) = resp.kvs().first() {
        println!("Get kv: {}: {}", kv.key_str()?, kv.value_str()?);
    }

    let resp = client
        .delete("name", Some(DeleteOptions::new().with_prefix()))
        .await?;
    if let Some(kv) = resp.prev_kvs().first() {
        println!("Get kv: {}: {}", kv.key_str()?, kv.value_str()?);
    }

    Ok(())
}
