# string-metrics-rs

## Basic Usage

### Edit distance

```py
from string_metrics import levenshtein

assert levenshtein("abcde", "abc") == 2
```

### MinHash

```py
from string_metrics.approx import MinHash

h1 = MinHash(signature_size=100, seed=42)
h2 = MinHash(signature_size=100, seed=42)

h1.update(b"hello")
h2.update_batch([b"hello", b"world"])

assert 0.0 <= h1.jaccard(h2) <= 1.0
```

## Installation

```bash
pip install string-metrics-rs
```
