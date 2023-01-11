use std::{
    collections::HashMap,
    env::current_dir,
    fs,
    path::{Path, PathBuf},
};

use circom::circuit::{CircomCircuit, R1CS};
use nova_snark::{
    circuit::{NovaAugmentedCircuit, NovaAugmentedCircuitInputs, NovaAugmentedCircuitParams},
    r1cs::{
        R1CSGens, R1CSInstance, R1CSShape, R1CSWitness, RelaxedR1CSInstance, RelaxedR1CSWitness,
    },
    traits::{
        circuit::TrivialTestCircuit, AbsorbInROTrait, Group, ROConstants, ROConstantsCircuit,
        ROConstantsTrait, ROTrait,
    },
    PublicParams, RecursiveSNARK,
};
use num_bigint::BigInt;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::circom::reader::generate_witness_from_bin;

#[cfg(not(target_family = "wasm"))]
use crate::circom::reader::generate_witness_from_wasm;

#[cfg(target_family = "wasm")]
use crate::circom::wasm::generate_witness_from_wasm;

pub mod circom;

pub type G1 = pasta_curves::pallas::Point;
pub type F1 = <G1 as Group>::Scalar;
pub type G2 = pasta_curves::vesta::Point;
pub type F2 = <G2 as Group>::Scalar;
type C1 = CircomCircuit<<G1 as Group>::Scalar>;
type C2 = TrivialTestCircuit<<G2 as Group>::Scalar>;

pub enum FileLocation {
    PathBuf(PathBuf),
    URL(String),
}

pub fn create_public_params(
    r1cs: R1CS<F1>,
) -> PublicParams<G1, G2, CircomCircuit<F1>, TrivialTestCircuit<F2>> {
    let circuit_primary = CircomCircuit {
        r1cs,
        witness: None,
    };
    let circuit_secondary = TrivialTestCircuit::default();

    // let ro_consts_primary: ROConstants<G1> = ...;
    // ro_consts_primary.
    // ro_consts_circuit_primary: ROConstantsCircuit<G2>,
    // r1cs_gens_primary: R1CSGens<G1>,
    // r1cs_shape_primary: R1CSShape<G1>,
    // r1cs_shape_padded_primary: R1CSShape<G1>,
    // ro_consts_secondary: ROConstants<G2>,
    // ro_consts_circuit_secondary: ROConstantsCircuit<G1>,
    // r1cs_gens_secondary: R1CSGens<G2>,
    // r1cs_shape_secondary: R1CSShape<G2>,
    // r1cs_shape_padded_secondary: R1CSShape<G2>,

    let pp = PublicParams::<G1, G2, CircomCircuit<F1>, TrivialTestCircuit<F2>>::setup(
        circuit_primary.clone(),
        circuit_secondary.clone(),
    ); 
    pp
}

#[derive(Serialize, Deserialize)]
pub struct CircomInput {
    pub step_in: Vec<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[cfg(not(target_family = "wasm"))]
pub fn create_recursive_circuit(
    witness_generator_file: FileLocation,
    r1cs: R1CS<F1>,
    private_inputs: Vec<HashMap<String, Value>>,
    start_public_input: Vec<F1>,
    pp: &PublicParams<G1, G2, C1, C2>,
) -> Result<RecursiveSNARK<G1, G2, C1, C2>, std::io::Error> {
    let root = current_dir().unwrap();
    let witness_generator_output = root.join("circom_witness.wtns");

    let iteration_count = private_inputs.len();
    let mut circuit_iterations = Vec::with_capacity(iteration_count);

    let start_public_input_hex = start_public_input
        .iter()
        .map(|&x| format!("{:?}", x).strip_prefix("0x").unwrap().to_string())
        .collect::<Vec<String>>();
    let mut current_public_input = start_public_input_hex.clone();

    for i in 0..iteration_count {
        let decimal_stringified_input: Vec<String> = current_public_input
            .iter()
            .map(|x| BigInt::from_str_radix(x, 16).unwrap().to_str_radix(10))
            .collect();

        let input = CircomInput {
            step_in: decimal_stringified_input.clone(),
            extra: private_inputs[i].clone(),
        };

        let input_json = serde_json::to_string(&input).unwrap();
<<<<<<< HEAD
<<<<<<< HEAD
=======
        println!("- {}", input_json);
=======
        println!("  - R1CS instance w/ input {}", input_json);
>>>>>>> 8080895 (attempted revert)
        fs::write(&witness_generator_input, input_json).unwrap();
>>>>>>> d481e7f (more verbose printing)

        let is_wasm = match &witness_generator_file {
            FileLocation::PathBuf(path) => path.extension().unwrap_or_default() == "wasm",
            FileLocation::URL(_) => true,
        };

        let witness = if is_wasm {
            generate_witness_from_wasm::<<G1 as Group>::Scalar>(
                &witness_generator_file,
                &input_json,
                &witness_generator_output,
            )
        } else {
            let witness_generator_file = match &witness_generator_file {
                FileLocation::PathBuf(path) => path,
                FileLocation::URL(_) => panic!("unreachable"),
            };
            generate_witness_from_bin::<<G1 as Group>::Scalar>(
                &witness_generator_file,
                &input_json,
                &witness_generator_output,
            )
        };
        let circuit = CircomCircuit {
            r1cs: r1cs.clone(),
            witness: Some(witness),
        };
        let current_public_output = circuit.get_public_outputs();
        println!("  - OUTPUT: {:?}", current_public_output);

        circuit_iterations.push(circuit);
        current_public_input = current_public_output
            .iter()
            .map(|&x| format!("{:?}", x).strip_prefix("0x").unwrap().to_string())
            .collect();
    }
<<<<<<< HEAD
    fs::remove_file(witness_generator_output)?;
    println!("==");
=======
    // fs::remove_file(witness_generator_input)?;
    // fs::remove_file(witness_generator_output)?;
>>>>>>> 8080895 (attempted revert)

    let circuit_secondary = TrivialTestCircuit::default();

    let mut recursive_snark: Option<RecursiveSNARK<G1, G2, C1, C2>> = None;

    let z0_secondary = vec![<G2 as Group>::Scalar::zero()];

    for i in 0..iteration_count {
        // println!("  - folding step {}", i);
        let res = RecursiveSNARK::prove_step(
            &pp,
            recursive_snark,
            circuit_iterations[i].clone(),
            circuit_secondary.clone(),
            start_public_input.clone(),
            z0_secondary.clone(),
        );

        assert!(res.is_ok());
        recursive_snark = Some(res.unwrap());
    }

    let recursive_snark = recursive_snark.unwrap();
    Ok(recursive_snark)
}

#[cfg(target_family = "wasm")]
pub async fn create_recursive_circuit(
    witness_generator_file: FileLocation,
    r1cs: R1CS<F1>,
    private_inputs: Vec<HashMap<String, Value>>,
    start_public_input: Vec<F1>,
    pp: &PublicParams<G1, G2, C1, C2>,
) -> Result<RecursiveSNARK<G1, G2, C1, C2>, std::io::Error> {
    let iteration_count = private_inputs.len();
    let mut circuit_iterations = Vec::with_capacity(iteration_count);

    let start_public_input_hex = start_public_input
        .iter()
        .map(|&x| format!("{:?}", x).strip_prefix("0x").unwrap().to_string())
        .collect::<Vec<String>>();
    let mut current_public_input = start_public_input_hex.clone();

    for i in 0..iteration_count {
        let decimal_stringified_input: Vec<String> = current_public_input
            .iter()
            .map(|x| BigInt::from_str_radix(x, 16).unwrap().to_str_radix(10))
            .collect();

        let input = CircomInput {
            step_in: decimal_stringified_input.clone(),
            extra: private_inputs[i].clone(),
        };

        let input_json = serde_json::to_string(&input).unwrap();

        let is_wasm = match &witness_generator_file {
            FileLocation::PathBuf(path) => path.extension().unwrap_or_default() == "wasm",
            FileLocation::URL(_) => true,
        };

        let witness = if is_wasm {
            generate_witness_from_wasm::<<G1 as Group>::Scalar>(
                &witness_generator_file,
                &input_json,
                Path::new(""),
            )
            .await
        } else {
            let witness_generator_file = match &witness_generator_file {
                FileLocation::PathBuf(path) => path,
                FileLocation::URL(_) => panic!("unreachable"),
            };
            generate_witness_from_bin::<<G1 as Group>::Scalar>(
                &witness_generator_file,
                &input_json,
                Path::new(""),
            )
        };
        let circuit = CircomCircuit {
            r1cs: r1cs.clone(),
            witness: Some(witness),
        };
        let current_public_output = circuit.get_public_outputs();

        circuit_iterations.push(circuit);
        current_public_input = current_public_output
            .iter()
            .map(|&x| format!("{:?}", x).strip_prefix("0x").unwrap().to_string())
            .collect();
    }

    let circuit_secondary = TrivialTestCircuit::default();

    let mut recursive_snark: Option<RecursiveSNARK<G1, G2, C1, C2>> = None;

    let z0_secondary = vec![<G2 as Group>::Scalar::zero()];

    for i in 0..iteration_count {
        let res = RecursiveSNARK::prove_step(
            &pp,
            recursive_snark,
            circuit_iterations[i].clone(),
            circuit_secondary.clone(),
            start_public_input.clone(),
            z0_secondary.clone(),
        );

        assert!(res.is_ok());
        recursive_snark = Some(res.unwrap());
    }

    let recursive_snark = recursive_snark.unwrap();

    Ok(recursive_snark)
}
