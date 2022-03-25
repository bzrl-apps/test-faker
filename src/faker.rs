use futures::Future;
use std::pin::Pin;

use tokio::sync::broadcast::{Sender, Receiver};
use anyhow::{Result, Error};

use serde_yaml::Mapping;

//#[async_trait]
pub trait FakerMod {
    type Future: Future<Output = Result<(), Error>>;

    fn validate_params(&self) -> Result<()>;
    fn func(&self, tx: Sender<bool>, rx: Receiver<bool>) -> Self::Future;
}

pub struct Faker {
    pub name: &'static str,
    pub func: fn(params: Mapping, tx: Sender<bool>, rx: Receiver<bool>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>,
}

inventory::collect!(Faker);

pub fn get_faker(name: &str) -> Option<&'static Faker> {
    inventory::iter::<Faker>
        .into_iter()
        .find(|f| f.name == name)
}

pub fn get_faker_names() -> Vec<&'static str> {
    inventory::iter::<Faker>
        .into_iter()
        .map(|f| f.name)
        .collect::<Vec<&str>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faker_get_faker_names() {
        println!("{:#?}", get_faker_names());
    }
}
