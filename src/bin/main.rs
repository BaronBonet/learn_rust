use tokio::sync::mpsc;
use tokio::try_join;

const NAMES: [&str; 5] = ["Steve", "Bob", "Alice", "John", "Jane"];

#[derive(Debug)]
struct Person {
    name: String,
    age: u64,
}

async fn some_computation(name: &str) -> Person {
    if name == "Steve" {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    let resp = ureq::get(format!("https://api.agify.io/?name={}", name).as_str())
        .call()
        .unwrap()
        .into_string()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(resp.as_str()).unwrap();
    let age = parsed["age"].as_u64().unwrap();
    Person {
        name: name.to_string(),
        age,
    }
}

async fn save_to_db(person: Person) {
    println!("Saving {} to db", person.name);
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(1);

    for name in NAMES.iter() {
        // Each task needs its own `tx` handle. This is done by cloning the
        // original handle.
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let result = some_computation(name).await;
            tx_clone.send(result).await.unwrap();
        });
    }

    while let Some(res) = rx.recv().await {
        save_to_db(res).await;
    }
}
