 
use serde::Deserialize;
use std::collections::HashMap;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Zero, One};

#[derive(Deserialize, Debug)]
struct KeysMeta {
    n: usize,
    k: usize,
}

#[derive(Deserialize, Debug)]
struct Share {
    base: String,
    value: String,
}

#[derive(Deserialize, Debug)]
struct Input {
    keys: KeysMeta,
    #[serde(flatten)]
    shares: HashMap<String, Share>,
}

fn parse_in_base(value: &str, base: u32) -> BigInt {
    BigInt::parse_bytes(value.as_bytes(), base)
        .unwrap_or_else(|| panic!("failed to parse {value} in base {base}"))
}

/// Perform Lagrange interpolation at x=0
fn lagrange_at_zero(points: &[(BigInt, BigInt)]) -> BigInt {
    let mut secret = BigRational::zero();

    for i in 0..points.len() {
        let (ref xi, ref yi) = points[i];
        let mut li = BigRational::one();

        for j in 0..points.len() {
            if i == j {
                continue;
            }
            let (ref xj, _) = points[j];
            let num = BigRational::from_integer(-xj.clone());
            let den = BigRational::from_integer(xi.clone() - xj.clone());
            li *= num / den;
        }

        secret += BigRational::from_integer(yi.clone()) * li;
    }

    secret.to_integer()
}

fn main() {
    // Example JSON (testcase2)
    let raw = r#"
    {
      "keys": { "n": 10, "k": 7 },
      "1": { "base": "6", "value": "13444211440455345511" },
      "2": { "base": "15", "value": "aed7015a346d635" },
      "3": { "base": "15", "value": "6aeeb69631c227c" },
      "4": { "base": "16", "value": "e1b5e05623d881f" },
      "5": { "base": "8", "value": "316034514573652620673" },
      "6": { "base": "3", "value": "2122212201122002221120200210011020220200" },
      "7": { "base": "3", "value": "20120221122211000100210021102001201112121" },
      "8": { "base": "6", "value": "20220554335330240002224253" },
      "9": { "base": "12", "value": "45153788322a1255483" },
      "10": { "base": "7", "value": "1101613130313526312514143" }
    }
    "#;

    let input: Input = serde_json::from_str(raw).expect("failed to parse json");

    // Collect points
    let mut points: Vec<(BigInt, BigInt)> = Vec::new();
    for (id, share) in &input.shares {
        let x = BigInt::from(id.parse::<i64>().unwrap());
        let base: u32 = share.base.parse().unwrap();
        let y = parse_in_base(&share.value, base);
        points.push((x, y));
    }

    // Sort by x
    points.sort_by(|a, b| a.0.cmp(&b.0));
    let k = input.keys.k;

    // Step 1: Reconstruct secret with first k points
    let subset: Vec<(BigInt, BigInt)> = points.iter().take(k).cloned().collect();
    let secret = lagrange_at_zero(&subset);
    println!("🔑 Secret reconstructed = {}", secret);

    // Step 2: Check each share for consistency
    let mut wrong_keys = Vec::new();
    for i in 0..points.len() {
        let mut subset: Vec<(BigInt, BigInt)> = points.clone();
        subset.remove(i);

        if subset.len() >= k {
            let trial_secret = lagrange_at_zero(&subset.iter().take(k).cloned().collect::<Vec<_>>());
            if trial_secret != secret {
                wrong_keys.push(points[i].0.clone());
            }
        }
    }

    if wrong_keys.is_empty() {
        println!("✅ All keys are correct, lock opened!");
    } else {
        println!("❌ Wrong key(s) entered by person(s): {:?}", wrong_keys);
    }
}
