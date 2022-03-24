use serde::{Deserialize, Serialize};
use serde_yaml::{Value, Mapping};

use similar::{ChangeTag, TextDiff};

use std::fs::File;
use std::io::Read;

use anyhow::{anyhow, Result};

use log::*;

use crate::verifier::{Verifier, VerifierMod};

// Our plugin implementation
#[derive(Default, Debug ,Serialize, Deserialize, Clone)]
struct FileComparator {
    expected: String,
    actual: String,
}

impl VerifierMod for FileComparator {
    fn validate_params(&self) -> Result<()> {
        if self.expected.is_empty() {
            return Err(anyhow!("expected cannot be empty"));
        }

        if self.actual.is_empty() {
            return Err(anyhow!("actual cannot be empty"));
        }

        Ok(())
    }

    fn func(&self) -> Result<()> {
        let _ =  env_logger::try_init();

        let mut f_expected = File::open(self.expected.clone())?;
        let mut contents_expected = String::new();
        f_expected.read_to_string(&mut contents_expected)?;

        let mut f_actual = File::open(self.actual.clone())?;
        let mut contents_actual = String::new();
        f_actual.read_to_string(&mut contents_actual)?;

        let diff = TextDiff::from_lines(
            &contents_expected,
            &contents_actual
        );

        let diffops: Vec<&similar::DiffOp> = diff.ops().iter().filter(|x| x.tag() != similar::DiffTag::Equal).collect();

        if !diffops.is_empty() {
            return Err(anyhow!("Difference found between 2 files: expected & actual"));
        }

       Ok(())
    }
}

fn func(params: Mapping) -> Result<()> {
    let v_params = Value::Mapping(params);

    let comparator: FileComparator  = serde_yaml::from_value(v_params)?;

    comparator.validate_params()?;
    comparator.func()?;

    Ok(())
}

inventory::submit!(Verifier {name: "file-comparator", func: func });
