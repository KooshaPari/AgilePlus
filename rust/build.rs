// SPDX-License-Identifier: MIT OR Apache-2.0
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(
            &[
                "../agileplus-agents/proto/agileplus/v1/common.proto",
                "../agileplus-agents/proto/agileplus/v1/core.proto",
                "../agileplus-agents/proto/agileplus/v1/agents.proto",
                "../agileplus-agents/proto/agileplus/v1/integrations.proto",
            ],
            &["../agileplus-agents/proto"],
        )?;
    Ok(())
}
