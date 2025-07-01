use axum::{debug_handler, extract::Path, Json};
use borsh::BorshDeserialize;
use uuid::Uuid;

#[derive(Debug, serde::Serialize, BorshDeserialize)]
pub struct Fruit {
    // Singular name is more conventional
    name: String,
    nutrients: Vec<String>,
    id: Option<String>,
}

impl Fruit {
    pub fn new(name: String, nutrients: Vec<String>) -> Self {
        Self {
            name,
            nutrients,
            id: Some(Uuid::new_v4().to_string()),
        }
    }
}

#[debug_handler]
pub async fn get_all_fruits() -> Json<Vec<Fruit>> {
    // Return Vec for "all"
    println!("Getting all fruits");

    let fruits = vec![
        Fruit::new(
            "banana".to_string(),
            vec!["potassium".to_string(), "vitamin B6".to_string()],
        ),
        Fruit::new(
            "apple".to_string(),
            vec!["fiber".to_string(), "vitamin C".to_string()],
        ),
        Fruit::new(
            "orange".to_string(),
            vec!["vitamin C".to_string(), "folate".to_string()],
        ),
    ];

    Json(fruits)
}

#[debug_handler]
pub async fn get_single_fruit(Path(fruit_name): Path<String>) -> Json<Fruit> {
    println!("Getting single fruit");

    let all_fruits = vec![
        Fruit::new(
            "banana".to_string(),
            vec!["potassium".to_string(), "vitamin B6".to_string()],
        ),
        Fruit::new(
            "apple".to_string(),
            vec!["fiber".to_string(), "vitamin C".to_string()],
        ),
        Fruit::new(
            "orange".to_string(),
            vec!["vitamin C".to_string(), "folate".to_string()],
        ),
    ];

    let fruit = all_fruits
        .into_iter()
        .find(|fruit| fruit.name == fruit_name);

    fruit.map(Json).unwrap()
}
