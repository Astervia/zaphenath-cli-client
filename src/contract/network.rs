use super::types::NetworkContext;
use std::collections::HashMap;

/// Resolves a known network name to an RPC URL
pub fn resolve_rpc_url(network: &str) -> Option<String> {
    let mut map = HashMap::new();

    map.insert("mainnet", "https://mainnet.infura.io/v3/YOUR_PROJECT_ID");
    map.insert("sepolia", "https://sepolia.infura.io/v3/YOUR_PROJECT_ID");
    map.insert("goerli", "https://goerli.infura.io/v3/YOUR_PROJECT_ID");
    map.insert("localhost", "http://localhost:8545");

    map.get(network).map(|s| s.to_string())
}

pub fn build_network_context(
    rpc_url_opt: Option<&str>,
    network_opt: Option<&str>,
) -> Result<NetworkContext, String> {
    if let Some(url) = rpc_url_opt {
        return Ok(NetworkContext {
            network: network_opt.map(|s| s.to_string()),
            rpc_url: url.to_string(),
        });
    }

    if let Some(network) = network_opt {
        let resolved =
            resolve_rpc_url(network).ok_or_else(|| format!("Unknown network name: {}", network))?;
        return Ok(NetworkContext {
            network: Some(network.to_string()),
            rpc_url: resolved,
        });
    }

    Err("You must provide either --rpc-url or --network".to_string())
}
