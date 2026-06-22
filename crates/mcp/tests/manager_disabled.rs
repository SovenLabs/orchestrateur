use mcp::McpManager;
use orchestrator::McpConfig;

#[tokio::test]
async fn connect_empty_servers_succeeds() {
    let config = McpConfig {
        enabled: true,
        servers: Vec::new(),
    };
    let manager = McpManager::connect(&config).await.expect("connect");
    assert_eq!(manager.server_count(), 0);
}