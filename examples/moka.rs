use anyhow::Result;
use moka::future::Cache;
use std::time::Duration;
use tokio::time::sleep;

#[allow(unused)]
#[derive(Debug, Clone)]
struct Phone {
    name: String,
    price: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cache = Cache::new(10_000);

    let key = 200;
    cache
        .insert(
            key,
            Phone {
                name: "华为".to_string(),
                price: 100,
            },
        )
        .await;

    let v = cache.get(&key).await;
    println!("{:?}", v);

    // cacha.invalidate(&key).await; // 无回值

    let v = cache.remove(&key).await; // 删除后有返回值
    println!("{:?}", v);

    cache
        .insert(
            key,
            Phone {
                name: "华为".to_string(),
                price: 5999,
            },
        )
        .await;
    let v = cache.get(&key).await;
    println!("{:?}", v);

    let ttl = 1;

    let cache = Cache::builder()
        .max_capacity(2)
        .time_to_live(Duration::from_secs(ttl))
        .eviction_listener(|key, value, cause| {
            println!("Evicted {:?}, {:?}, because {:?}", key, value, cause)
        })
        .build();

    cache.insert(&0, "刘德华".to_string()).await;
    cache.insert(&1, "赵本山".to_string()).await;
    cache.insert(&2, "郭德纲".to_string()).await;
    cache.insert(&3, "郭德纲".to_string()).await;
    cache.insert(&4, "郭德纲".to_string()).await;
    cache.insert(&5, "郭德纲".to_string()).await;

    sleep(Duration::from_secs(ttl + 3)).await;

    let v = cache.get(&0).await;
    println!("v: {:?}", v);

    Ok(())
}
