---
layout: doc
title: Hello World Story
---

# Hello World: Your First AgilePlus Operation

<StoryHeader
    title="First Operation"
    duration="2"
    difficulty="beginner"
    :gif="'/gifs/agileplus-hello-world.gif'"
/>

## Objective

Execute your first AgilePlus operation successfully.

## Prerequisites

- Rust/Node/Python installed
- AgilePlus CLI installed

## Implementation

```rust
use agileplus::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new().await?;
    let result = client.hello().await?;
    println!("Success: {}", result);
    Ok(())
}
```

## Expected Output

```
Success: Hello from AgilePlus!
```

## Next Steps

- [Core Integration](./core-integration)
- [API Reference](../reference/api)
