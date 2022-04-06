use networking::{ArtificePeer, ArtificeConfig, Layer3SocketAddr, Layer3Addr, get_private_key, ArtificeHostData, encryption::PubKeyComp};

pub fn test_config() -> (ArtificePeer, ArtificeConfig) {
	let peer_addr: Layer3SocketAddr = Layer3SocketAddr::new(Layer3Addr::newv4(127, 0, 0, 1), 6464);
	let host_addr: Layer3SocketAddr = Layer3SocketAddr::new(Layer3Addr::newv4(127, 0, 0, 1), 6464);
	let private_key = get_private_key();
	let pubkey = PubKeyComp::from(&private_key);
	// poorly named, global is unique to each host, and peer hash is a pre-shared key
	let host_hash = "f7Cgkll1EegEa5UyuUEADpYAXRXwrhbSB0FLLiYxHpBotzNrw9";
	let peer_hash = "7VKkjONo1txtTAiR1vQWUTsGxh8jwQJips1ClMv9zv1CsOo3ZX";
	let remote_hash = "73C0YnEJRpTd56wPwR8zHa3egpW8iM1ShCRAtutkcssenNkJ0T";
	let peer = ArtificePeer::new(remote_hash, peer_hash, peer_addr, Some(pubkey));
	let host_data = ArtificeHostData::new(&private_key, host_hash);
	let config = ArtificeConfig::new(host_addr, host_data, false);
	(peer, config)
}
