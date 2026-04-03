"""Step definitions for BDD tests.

Implement the steps referenced in your .feature files.
"""

import pytest
from pytest_bdd import given, when, then, parsers, scenarios
from typing import Dict, Any

# Load scenarios from feature files
scenarios("features/example.feature")


# Shared context fixture
@pytest.fixture
def context() -> Dict[str, Any]:
    """Shared context for step definitions."""
    return {}


# Background steps
@given("the system is initialized")
def system_initialized(context):
    """Initialize the system."""
    context["initialized"] = True


# Given steps
@given("some precondition")
def some_precondition(context):
    """Set up some precondition."""
    context["precondition"] = "met"


@given("an invalid input")
def invalid_input(context):
    """Set up invalid input."""
    context["input_valid"] = False


@given("some data")
def some_data(context):
    """Set up some data."""
    context["data"] = "test data"


@given("a complete workflow")
def complete_workflow(context):
    """Set up a complete workflow."""
    context["workflow"] = "complete"


# When steps
@when("I perform an action")
def perform_action(context):
    """Perform an action."""
    initialized = context.get("initialized", False)
    input_valid = context.get("input_valid", True)

    if initialized and input_valid:
        context["action_result"] = "success"
    else:
        context["action_result"] = "error"
        context["error_message"] = "Invalid input or not initialized"


@when("I save the data")
def save_data(context):
    """Save the data."""
    data = context.get("data", "")
    context["saved_data"] = data


@when("I retrieve the data")
def retrieve_data(context):
    """Retrieve the data."""
    # Data is already saved in context
    pass


@when("all steps are executed")
def execute_all_steps(context):
    """Execute all workflow steps."""
    context["workflow_complete"] = True


# Then steps
@then("the result should be success")
def result_success(context):
    """Verify the result is success."""
    result = context.get("action_result", "")
    assert result == "success", f"Expected success but got {result}"


@then("an error should occur")
def error_occurs(context):
    """Verify an error occurs."""
    result = context.get("action_result", "")
    assert result == "error", f"Expected error but got {result}"


@then("the error message should explain the problem")
def error_message_explains(context):
    """Verify error message exists."""
    message = context.get("error_message", "")
    assert message, "Error message should exist"


@then("the retrieved data should match the saved data")
def data_matches(context):
    """Verify retrieved data matches saved data."""
    original = context.get("data", "")
    saved = context.get("saved_data", "")
    assert original == saved, f"Retrieved data should match: {original} != {saved}"


@then("the workflow should complete successfully")
def workflow_complete(context):
    """Verify workflow completes."""
    complete = context.get("workflow_complete", False)
    assert complete, "Workflow should complete"


# Example of parameterized steps
@given(parsers.parse("I have {count:d} items"))
def given_items(context, count: int):
    """Set up a specific number of items."""
    context["item_count"] = count
    context["items"] = [f"item_{i}" for i in range(count)]


@when(parsers.parse("I add {count:d} more items"))
def add_items(context, count: int):
    """Add more items."""
    current_items = context.get("items", [])
    current_count = len(current_items)
    for i in range(count):
        current_items.append(f"item_{current_count + i}")
    context["items"] = current_items
    context["item_count"] = len(current_items)


@then(parsers.parse("I should have {count:d} items total"))
def verify_item_count(context, count: int):
    """Verify total item count."""
    actual_count = context.get("item_count", 0)
    assert actual_count == count, f"Expected {count} items but got {actual_count}"
