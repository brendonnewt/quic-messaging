use quinn::{ServerConfig, TransportConfig};
use rcgen::generate_simple_self_signed;
use rustls::{Certificate, PrivateKey};
use std::sync::Arc;
use std::time::Duration;

pub fn generate_self_signed_cert() -> ServerConfig {
    // Generate self-signed certificate using rcgen
    let cert = generate_simple_self_signed(vec!["localhost".into()])
        .expect("Failed to generate self-signed certificate");

    let cert_der = cert.serialize_der().expect("Failed to serialize cert");
    let key_der = cert.serialize_private_key_der();

    let cert_chain = vec![Certificate(cert_der)];
    let private_key = PrivateKey(key_der);

    // Create server config with single cert
    let mut server_config = ServerConfig::with_single_cert(cert_chain, private_key)
        .expect("Failed to create server config");

    // Configure transport settings
    let mut transport_config = TransportConfig::default();

    transport_config.max_concurrent_bidi_streams(100_u32.into());

    transport_config.max_idle_timeout(Some(
        Duration::from_secs(300) //Connection is closed and user logged out after 5 min
            .try_into()
            .expect("valid idle timeout"),
    ));

    server_config.transport = Arc::new(transport_config);

    server_config
}
