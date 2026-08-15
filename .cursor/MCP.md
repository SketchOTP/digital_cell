# MCP Configuration

`.cursor/mcp.json` intentionally contains an empty `mcpServers` object in this portable governance package.

Machine-local or user-specific MCP servers, credentials, executable paths, and workspace bindings must not be committed here. A repository that adopts an MCP integration should document only verified repository-owned configuration and keep private configuration outside version control.
