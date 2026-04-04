# Phenotype Traceability

Python package for Feature Requirement (FR) traceability in tests.

## Installation

```bash
pip install phenotype-traceability
```

## Usage

### pytest

```python
import pytest

@pytest.mark.traces_to("FR-EXAMPLE-001")
def test_feature():
    assert True
```

### unittest

```python
from phenotype_traceability import traces_to

class TestFeature(unittest.TestCase):
    @traces_to("FR-EXAMPLE-001")
    def test_feature(self):
        self.assertTrue(True)
```

## License

Apache-2.0
