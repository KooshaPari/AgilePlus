# Phenotype Traceability

Feature Requirement (FR) traceability for tests across 11+ languages.

## Overview

Phenotype Traceability provides a unified way to trace tests back to Feature Requirements (FRs) across multiple programming languages and test frameworks.

## Supported Languages

| Language | Package | Test Frameworks |
|----------|---------|-----------------|
| Python | `phenotype-traceability` | pytest, unittest |
| Go | `gotreqt` | testing |
| Rust | `traceability-macros` | built-in test |
| TypeScript | `@phenotype/tstreqt` | Jest, Vitest, Playwright, Mocha, AVA, TAP, Node |
| C# | `Phenotype.Traceability` | NUnit, xUnit, MSTest |
| Java | `javatreqt` | JUnit 5, TestNG |
| Kotlin | `kotlintreqt` | JUnit 5, Kotest |
| Swift | `SwiftTreqt` | XCTest |
| C/C++ | `ctreqt` | Custom macro |
| Zig | `zigtreqt` | built-in test |
| Elixir | `extreqt` | ExUnit |

## Installation

### Python

```bash
pip install phenotype-traceability
```

### Go

```bash
go get github.com/phenotype/AgilePlus/go/gotreqt
```

### Rust

Add to `Cargo.toml`:
```toml
[dependencies]
traceability-macros = { path = "../AgilePlus/crates/traceability-macros" }
```

### TypeScript

```bash
npm install @phenotype/tstreqt
```

### C#

```bash
dotnet add package Phenotype.Traceability
```

### Java

```xml
<dependency>
    <groupId>com.phenotype</groupId>
    <artifactId>javatreqt</artifactId>
    <version>0.1.0</version>
</dependency>
```

### Kotlin

```kotlin
implementation("com.phenotype:kotlintreqt:0.1.0")
```

### Swift

Add to `Package.swift`:
```swift
dependencies: [
    .package(path: "../AgilePlus/swift/SwiftTreqt")
]
```

### C/C++

Include the header:
```c
#include "ctreqt.h"
```

### Zig

Add to `build.zig.zon`:
```zig
.dependencies = .{
    .zigtreqt = .{
        .path = "../AgilePlus/zig/zigtreqt"
    }
}
```

### Elixir

Add to `mix.exs`:
```elixir
defp deps do
  [{:extreqt, path: "../AgilePlus/elixir/extreqt"}]
end
```

## Usage

### Python (pytest)

```python
import pytest

@pytest.mark.traces_to("FR-EXAMPLE-001")
def test_feature():
    assert True
```

### Go

```go
import (
    "testing"
    "github.com/phenotype/AgilePlus/go/gotreqt"
)

func TestFeature(t *testing.T) {
    gotreqt.TraceTo(t, "FR-EXAMPLE-001")
    // test code
}
```

### Rust

```rust
use traceability_macros::trace_to;

#[trace_to("FR-EXAMPLE-001")]
#[test]
fn test_feature() {
    assert!(true);
}
```

### TypeScript (Jest)

```typescript
import { tracesTo } from '@phenotype/tstreqt';

test('feature', tracesTo('FR-EXAMPLE-001'), () => {
    expect(true).toBe(true);
});
```

### TypeScript (Vitest)

```typescript
import { test } from 'vitest';
import { tracesTo } from '@phenotype/tstreqt/vitest';

test('feature', tracesTo('FR-EXAMPLE-001'), () => {
    expect(true).toBe(true);
});
```

### TypeScript (Playwright)

```typescript
import { test } from '@playwright/test';
import { tracesTo } from '@phenotype/tstreqt/playwright';

test('feature', tracesTo('FR-EXAMPLE-001'), async ({ page }) => {
    await page.goto('/');
});
```

### TypeScript (Mocha)

```typescript
import { tracesTo, describeFr } from '@phenotype/tstreqt/mocha';

describeFr('FR-EXAMPLE-001', 'Feature', () => {
    it('works', tracesTo('FR-EXAMPLE-002'), (done) => {
        done();
    });
});
```

### TypeScript (AVA)

```typescript
import test from 'ava';
import { withFr } from '@phenotype/tstreqt/ava';

test('feature', withFr('FR-EXAMPLE-001'), (t) => {
    t.pass();
});
```

### TypeScript (TAP)

```typescript
import { tracesTo } from '@phenotype/tstreqt/tap';
import tap from 'tap';

tap.test('feature', tracesTo('FR-EXAMPLE-001'), (t) => {
    t.end();
});
```

### TypeScript (Node.js Test)

```typescript
import { tracesTo, describeFr } from '@phenotype/tstreqt/node';

describeFr('FR-EXAMPLE-001', 'Feature', () => {
    it('works', tracesTo('FR-EXAMPLE-002'), async () => {
        // test code
    });
});
```

### C# (.NET)

```csharp
using Phenotype.Traceability;

[Test]
[TraceTo("FR-EXAMPLE-001")]
public void TestFeature() {
    // test code
}
```

### Java

```java
import com.phenotype.traceability.TraceTo;

@Test
@TraceTo({"FR-EXAMPLE-001"})
public void testFeature() {
    // test code
}
```

### Kotlin

```kotlin
import com.phenotype.traceability.TraceTo

@Test
@TraceTo("FR-EXAMPLE-001")
fun testFeature() {
    // test code
}
```

### Swift

```swift
import SwiftTreqt

func testFeature() throws {
    try traceTo("FR-EXAMPLE-001")
    // test code
}
```

### C/C++

```c
#include "ctreqt.h"

CTREQT_TRACE_TO(test_feature, "FR-EXAMPLE-001");
void test_feature(void) {
    // test code
}
```

### Zig

```zig
const zigtreqt = @import("zigtreqt");

test "feature" {
    try zigtreqt.traceTo(std.testing.allocator, "test_feature", "FR-EXAMPLE-001");
    // test code
}
```

### Elixir

```elixir
defmodule MyTest do
  use ExUnit.Case
  use Extreqt

  @trace_to ["FR-EXAMPLE-001"]
  test "feature works" do
    assert true
  end
end
```

## CLI Usage

The `ptrace` CLI provides commands for analyzing traceability:

```bash
# Analyze FR coverage across all languages
ptrace analyze --path ./src --lang all

# Check for spec drift
ptrace check-drift --spec FR-EXAMPLE-001 --code ./src --threshold 10

# Generate coverage report
ptrace coverage --path ./src --output TEST_COVERAGE.md

# Validate AI attribution
ptrace validate-ai --project . --strict

# Generate FR tracker report
ptrace fr-report --project . --with-ai
```

## Package Structure

```
AgilePlus/
├── crates/
│   ├── traceability-core/      # Shared Rust logic
│   ├── traceability-macros/    # Rust proc macros
│   └── traceability-cli/       # Unified ptrace CLI
├── python/
│   └── phenotype_traceability/  # Python (pytest, unittest)
├── go/
│   └── gotreqt/                 # Go
├── ts/
│   └── tstreqt/                 # TypeScript (Jest, Vitest, Playwright, Mocha, AVA, TAP, Node)
├── csharp/
│   └── Phenotype.Traceability/  # .NET (NUnit, xUnit, MSTest)
├── java/
│   └── javatreqt/               # Java (JUnit, TestNG)
├── kotlin/
│   └── kotlintreqt/             # Kotlin (JUnit, Kotest)
├── swift/
│   └── SwiftTreqt/              # Swift (XCTest)
├── cpp/
│   └── ctreqt/                  # C/C++
├── zig/
│   └── zigtreqt/                # Zig
└── elixir/
    └── extreqt/                 # Elixir (ExUnit)
```

## CI/CD Integration

Copy or reference `AgilePlus/.github/workflows/traceability-check.yml`:

```yaml
- name: Check FR Coverage
  run: ptrace coverage --path . --output TEST_COVERAGE.md

- name: Validate AI Attribution
  run: ptrace validate-ai --project . --strict

- name: Check Spec Drift
  run: ptrace check-drift --path . --threshold 10
```

## License

Apache-2.0
