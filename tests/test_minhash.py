import pytest

from string_metrics.approx import MinHash


def test_signature() -> None:
    h = MinHash(signature_size=3, seed=1)
    h.update(b"hello")
    expected = [363343485, 1915060716, 2198082125]
    assert h.signature == expected


@pytest.mark.parametrize(
    ("signature_size", "seed", "data1", "data2", "expected"),
    [(8, 100, [b"apple", b"banana", b"cherry"], [b"apple", b"banana", b"cherry"], 1.0)],
)
def test_jaccard(
    signature_size: int,
    seed: int,
    data1: list[bytes],
    data2: list[bytes],
    expected: float,
) -> None:
    h1 = MinHash(signature_size, seed)
    h2 = MinHash(signature_size, seed)
    h1.update_batch(data1)
    h2.update_batch(data2)
    assert h1.jaccard(h2) == expected
