use tokio::sync::mpsc;

const NAMES: [&str; 5] = ["Steve", "Bob", "Alice", "John", "Jane"];

#[derive(Debug)]
struct Person {
    name: String,
    age: u64,
}

async fn some_computation(name: &str) -> Person {
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

async fn do_thing(tx: mpsc::Sender<Person>) {
    for name in NAMES.iter() {
        let result = some_computation(name).await;
        if result.age > 40 {
            println!("{} is old", &result.name);
            tx.send(result).await.unwrap();
        } else {
            println!("{} is young", &result.name)
        }
    }
}

async fn save_to_db(person: Person) {
    println!("Saving {} to db", person.name);
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        do_thing(tx).await;
    });
    println!("Waiting for results");

    while let Some(res) = rx.recv().await {
        save_to_db(res).await;
    }
}
