# BDD Integration Template

This template shows how to integrate BDD testing with pytest-bdd into your Python project.

## Files

- `requirements.txt` - Dependencies (pytest-bdd, pytest)
- `pytest.ini` - Configuration for pytest
- `tests/features/example.feature` - Sample Gherkin feature file
- `tests/steps/test_example.py` - Step definitions

## Quick Start

1. Copy these files to your project
2. Install dependencies:
   ```bash
   pip install -r requirements.txt
   ```
3. Customize the feature files for your domain
4. Implement the step definitions
5. Run with `pytest`

## Project Structure

```
your-project/
├── tests/
│   ├── features/          # Gherkin feature files
│   │   └── example.feature
│   └── steps/             # Step definitions
│       └── test_example.py
├── requirements.txt
└── pytest.ini
```

## Writing Features

Create `.feature` files in `tests/features/`:

```gherkin
Feature: User Management
  As an admin
  I want to manage users
  So that I can control system access

  Scenario: Create a new user
    Given I am logged in as admin
    When I create a user with name "Alice"
    Then the user should exist with name "Alice"
```

## Writing Step Definitions

Implement steps in `tests/steps/test_*.py`:

```python
from pytest_bdd import given, when, then, parsers

@given('I am logged in as admin')
def logged_in_as_admin():
    # Setup admin session
    pass

@when(parsers.parse('I create a user with name "{name}"'))
def create_user(name):
    # Create user logic
    pass

@then(parsers.parse('the user should exist with name "{name}"'))
def user_exists(name):
    # Verify user exists
    pass
```

## Running Tests

```bash
# Run all BDD tests
pytest tests/steps/

# Run with verbose output
pytest -v tests/steps/

# Run specific feature
pytest tests/steps/test_example.py -v

# Run with tags (install pytest-bdd[tags] for tag support)
pytest -m "integration"
```

## Step Registry Pattern

For complex projects, use a step registry to organize steps:

```python
# steps/registry.py
from typing import Dict, Callable, Any

StepFunc = Callable[..., Any]

class StepRegistry:
    def __init__(self):
        self.givens: Dict[str, StepFunc] = {}
        self.whens: Dict[str, StepFunc] = {}
        self.thens: Dict[str, StepFunc] = {}
    
    def register_given(self, pattern: str, func: StepFunc):
        self.givens[pattern] = func
    
    def register_when(self, pattern: str, func: StepFunc):
        self.whens[pattern] = func
    
    def register_then(self, pattern: str, func: StepFunc):
        self.thens[pattern] = func

# Global registry instance
registry = StepRegistry()
```

## Customization

Replace the example domain ("Example Domain Logic") with your actual domain concepts. Add more scenarios and steps as needed.

## Integration with Hexagonal Architecture

BDD tests work well with hexagonal architecture:

- **Given** steps set up the domain state
- **When** steps invoke application services (ports)
- **Then** steps verify domain invariants and output

Example:

```python
# Application service (port)
class UserServicePort(ABC):
    @abstractmethod
    def create_user(self, name: str) -> User:
        pass

# BDD test using the port
@given('a user service')
def user_service():
    return UserServiceAdapter()

@when('I create a user')
def create_user(user_service):
    return user_service.create_user("Alice")
```

## Additional Resources

- [pytest-bdd documentation](https://pytest-bdd.readthedocs.io/)
- [Gherkin syntax reference](https://cucumber.io/docs/gherkin/)
- [BDD best practices](https://cucumber.io/docs/bdd/)
