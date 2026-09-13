use std::collections::{HashMap, HashSet};
use std::pin::Pin;

use chrono::Utc;
use tonic::{Response, Status};
use tracing::info;

use agileplus_domain::domain::governance::{GovernanceContract, GovernanceRule};
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::ports::{AgentPort, ObservabilityPort, ReviewPort, StoragePort, VcsPort};
use agileplus_proto::agileplus::v1::{
    CommandResponse, DispatchCommandRequest, DispatchCommandResponse, StreamAgentEventsRequest,
    StreamAgentEventsResponse,
};

use super::{domain_error_to_status, parse_evidence_requirement, evidence_satisfies_requirement, AgilePlusCoreServer};

pub(super) type StreamAgentEventsStream =
    Pin<Box<dyn tokio_stream::Stream<Item = Result<StreamAgentEventsResponse, Status>> + Send>>;

impl<S, V, A, R, O> AgilePlusCoreServer<S, V, A, R, O>
where
    S: StoragePort + 'static,
    V: VcsPort + 'static,
    A: AgentPort + 'static,
    R: ReviewPort + 'static,
    O: ObservabilityPort + 'static,
{
    pub(super) async fn handle_stream_agent_events(
        &self,
        request: StreamAgentEventsRequest,
    ) -> Result<Response<StreamAgentEventsStream>, Status> {
        let stream =
            crate::streaming::agent_event_stream(self.event_bus.subscribe(), request.feature_slug);
        Ok(Response::new(stream))
    }

    pub(super) async fn handle_dispatch_command(
        &self,
        request: DispatchCommandRequest,
    ) -> Result<Response<DispatchCommandResponse>, Status> {
        let command = request
            .command
            .ok_or_else(|| Status::invalid_argument("missing command field"))?;

        let agent_commands = ["implement"];
        let (success, message, outputs) = if agent_commands.contains(&command.command.as_str()) {
            let result = self
                .proxy
                .dispatch_agent_command(&command.command, &command.feature_slug, &command.args)
                .await;
            (
                result.is_success(),
                result.message().to_string(),
                result.outputs(),
            )
        } else {
            match self.dispatch_core_command(&command.command, &command.feature_slug, &command.args)
            {
                Ok((message, outputs)) => (true, message, outputs),
                Err(status) => return Err(status),
            }
        };

        Ok(Response::new(DispatchCommandResponse {
            result: Some(CommandResponse {
                success,
                message,
                outputs,
            }),
        }))
    }

    fn dispatch_core_command(
        &self,
        command: &str,
        feature_slug: &str,
        args: &HashMap<String, String>,
    ) -> Result<(String, HashMap<String, String>), Status> {
        match command {
            // Commands that are fully implemented in the CLI and can be dispatched
            // through the gRPC core. Each handler lives in the application layer
            // and operates on repository-local SQLite state.
            "specify" | "research" | "plan" | "validate" | "ship" | "retrospective" => {
                info!(command, feature_slug, "core command dispatched via gRPC");
                // NOTE: these commands are handled by the CLI binary, not the gRPC
                // daemon. Return unimplemented so callers know the gRPC surface
                // does not yet replicate CLI lifecycle handling.
                Err(Status::unimplemented(format!(
                    "command '{command}' is not yet implemented over gRPC; \
                     use the CLI binary 'agileplus {command}' instead"
                )))
            }
            other => Err(Status::unimplemented(format!("unknown command: '{other}'"))),
        }
    }
}
