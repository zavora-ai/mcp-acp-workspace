# Changelog

## [1.1.0] - 2026-05-23

### Added
- Sequence diagram (SVG) showing full ACP session lifecycle
- Verified output examples with real Gemini 2.5 Flash + tool execution
- API reference documentation for all 9 tools
- Protocol version updated to 1.0 (aligned with adk-acp)

### Verified
- `adk-acp` server: initialize → session/create → session/prompt → session/close
- Tool use: Gemini calls `list_directory`, reads actual filesystem
- Streaming: `agent_message_chunk` notifications delivered correctly

## [1.0.0] - 2026-05-23

### Added
- 9 MCP tools: `list_acp_connections`, `create_acp_session`, `send_acp_prompt`, `stream_acp_events`, `request_acp_permission`, `get_acp_patch`, `review_acp_patch`, `terminate_acp_session`, `expose_adk_agent_as_acp`
- ACP wire protocol alignment: `initialize`, `session/new`, `session/prompt`, `session/update`, `session/cancel`
- Permission gates mapped to governance policy (file_write, file_read, command, network)
- Patch review as platform-derived aggregation from tool-call content
- Session lifecycle: new, load, resume, close
- `session/cancel` handled as notification (per ACP spec)
- `session/update` payload preserved in full (params.update shape)
- Expose ADK-Rust agents as ACP-compatible servers
- Architecture SVG diagram
- Full documentation with protocol alignment table
