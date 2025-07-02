use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PySequence;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use sha1::{Digest, Sha1};

const MERSENNE_PRIME: u64 = (1u64 << 61) - 1;
const MAX_HASH: u64 = (1u64 << 32) - 1;

#[pyclass(frozen)]
pub struct HashFunction {
    seed: u64,
    permutations: Vec<(u64, u64)>,
}

impl HashFunction {
    fn new(signature_size: usize, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let permutations = (0..signature_size)
            .map(|_| {
                let a = rng.gen_range(1..MERSENNE_PRIME);
                let b = rng.gen_range(0..MERSENNE_PRIME);
                (a, b)
            })
            .collect();

        Self { seed, permutations }
    }

    /// Hashes a single data slice and returns an iterator over the permuted hash values.
    fn hash_iter<'a>(&'a self, data: &'a [u8]) -> impl Iterator<Item = u64> + 'a {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash_bytes = hasher.finalize();

        let hash_value = u32::from_le_bytes(hash_bytes[..4].try_into().unwrap()) as u64;

        self.permutations.iter().map(move |&(a, b)| {
            let permuted = a.wrapping_mul(hash_value).wrapping_add(b);
            (permuted % MERSENNE_PRIME) & MAX_HASH
        })
    }

    fn signature_size(&self) -> usize {
        self.permutations.len()
    }
}

#[pyclass]
pub struct MinHash {
    #[pyo3(get)]
    signature: Vec<u64>,
    #[pyo3(get)]
    hash_function: Py<HashFunction>,
}

#[pymethods]
impl MinHash {
    #[new]
    fn new(signature_size: usize, seed: u64, py: Python<'_>) -> PyResult<Self> {
        let hash_function = Py::new(py, HashFunction::new(signature_size, seed))?;
        let signature = vec![u64::MAX; signature_size];
        Ok(Self {
            signature,
            hash_function,
        })
    }

    #[staticmethod]
    fn from_hash_function(hash_function: Py<HashFunction>, py: Python<'_>) -> Self {
        let signature = vec![u64::MAX; hash_function.borrow(py).signature_size()];
        Self {
            signature,
            hash_function,
        }
    }

    /// Update the signature with a single value.
    pub fn update(&mut self, data: &[u8], py: Python<'_>) {
        let hf = self.hash_function.borrow(py);
        self.signature
            .iter_mut()
            .zip(hf.hash_iter(data))
            .for_each(|(min_value, new_value)| {
                *min_value = (*min_value).min(new_value);
            });
    }

    /// Update the signature with a batch of values from a Python sequence (e.g., a list).
    pub fn update_batch(&mut self, data: &Bound<'_, PySequence>, py: Python<'_>) -> PyResult<()> {
        for item in data.iter()? {
            self.update(item?.extract()?, py);
        }
        Ok(())
    }

    /// Estimates Jaccard similarity with another MinHash.
    pub fn jaccard(&self, other: &MinHash, py: Python<'_>) -> PyResult<f64> {
        let self_hf = self.hash_function.borrow(py);
        let other_hf = other.hash_function.borrow(py);

        if self_hf.signature_size() != other_hf.signature_size() {
            return Err(PyValueError::new_err(
                "Cannot compare MinHash objects with different numbers of hash functions.",
            ));
        }
        if self_hf.seed != other_hf.seed {
            return Err(PyValueError::new_err(
                "Cannot compare MinHash objects with different seeds.",
            ));
        }

        let intersection_size = self
            .signature
            .iter()
            .zip(other.signature.iter())
            .filter(|(a, b)| a == b)
            .count();

        Ok(intersection_size as f64 / self.signature.len() as f64)
    }
}
