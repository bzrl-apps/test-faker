use anyhow::{Result, Error};

use serde_yaml::Mapping;

pub trait VerifierMod {
    fn validate_params(&self) -> Result<()>;
    fn func(&self) -> Result<()>;
}

pub struct Verifier {
    pub name: &'static str,
    pub func: fn(params: Mapping) -> Result<()>,
}

inventory::collect!(Verifier);

pub fn get_verifier(name: &str) -> Option<&'static Verifier> {
    inventory::iter::<Verifier>
        .into_iter()
        .find(|f| f.name == name)
}

pub fn get_verifier_names() -> Vec<&'static str> {
    inventory::iter::<Verifier>
        .into_iter()
        .map(|f| f.name)
        .collect::<Vec<&str>>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_get_verifier_names() {
        println!("{:#?}", get_verifier_names());
    }
}
