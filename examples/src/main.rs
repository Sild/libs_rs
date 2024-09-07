use async_autoreturn_pool::Pool;

#[derive(Debug)]
struct MyObject {
    _value: i32
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = Pool::new(3).await;

    {
        let my_object1 = pool.try_take_or_create(|| Ok(MyObject { _value: 1 })).await?;
        println!("my_obj: {:?}", *my_object1);
    }
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let my_object2 = pool.try_take_or_create(|| Ok(MyObject { _value: 2 })).await?;
    println!("my_obj: {:?}", *my_object2);

    println!("Hello, world!");
    Ok(())
}
