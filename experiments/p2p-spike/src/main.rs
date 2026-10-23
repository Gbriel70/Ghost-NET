use std::{error::Error, io, time::Duration};

use futures::StreamExt;
use libp2p::{
    Multiaddr, SwarmBuilder, multiaddr::Protocol, noise, ping, swarm::SwarmEvent, tcp, yamux,
};
use tracing::{info, warn};

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(libp2p::swarm::NetworkBehaviour)]
struct Behaviour {
    ping: ping::Behaviour,
    limits: libp2p::connection_limits::Behaviour,
}

// Only explicit IPv4 loopback TCP addresses belong to this laboratory.
fn local_address(value: &str, dialing: bool) -> Result<Multiaddr> {
    let addr: Multiaddr = value.parse()?;
    let mut parts = addr.iter();
    match (parts.next(), parts.next()) {
        (Some(Protocol::Ip4(ip)), Some(Protocol::Tcp(port)))
            if ip.is_loopback() && (!dialing || port != 0) => {}
        _ => {
            return Err(
                io::Error::other("use /ip4/127.0.0.1/tcp/<port> (nonzero for dialing)").into(),
            );
        }
    }
    match (parts.next(), parts.next()) {
        (None, None) => Ok(addr),
        (Some(Protocol::P2p(_)), None) if dialing => Ok(addr),
        _ => Err(
            io::Error::other("only an optional final /p2p/<PeerId> is accepted for dialing").into(),
        ),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_target(false)
        .without_time()
        .init();

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() == 1 && args[0] == "--help" {
        println!("Usage: p2p-spike <listen-multiaddr> [dial-multiaddr-with-optional-peer-id]");
        return Ok(());
    }
    if !(1..=2).contains(&args.len()) {
        return Err(
            io::Error::other("usage: p2p-spike <listen-multiaddr> [dial-multiaddr]").into(),
        );
    }
    let listen = local_address(&args[0], false)?;
    let dial = args
        .get(1)
        .map(|value| local_address(value, true))
        .transpose()?;

    // Fresh Ed25519 identity per process; no key files are created.
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| Behaviour {
            ping: ping::Behaviour::new(
                ping::Config::new()
                    .with_interval(Duration::from_secs(1))
                    .with_timeout(Duration::from_secs(3)),
            ),
            limits: libp2p::connection_limits::Behaviour::new(
                libp2p::connection_limits::ConnectionLimits::default()
                    .with_max_pending_incoming(Some(4))
                    .with_max_pending_outgoing(Some(1))
                    .with_max_established(Some(4))
                    .with_max_established_per_peer(Some(1)),
            ),
        })?
        .with_swarm_config(|config| config.with_idle_connection_timeout(Duration::from_secs(60)))
        .with_connection_timeout(Duration::from_secs(5))
        .build();

    info!(event = "identity", local_peer_id = %swarm.local_peer_id());
    swarm.listen_on(listen)?;
    // Swarm extracts a final /p2p identity and verifies it against the authenticated peer.
    if let Some(address) = dial {
        info!(event = "dial_started", address = %address);
        if let Err(error) = swarm.dial(address) {
            warn!(event = "dial_error", error = ?error);
        }
    }

    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            signal = &mut shutdown => {
                signal?;
                info!(event = "shutdown");
                return Ok(());
            }
            event = swarm.next() => match event {
                Some(SwarmEvent::NewListenAddr { address, .. }) => {
                    let dial_address = address.with(Protocol::P2p(*swarm.local_peer_id()));
                    info!(event = "listening", address = %dial_address);
                }
                Some(SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, .. }) => {
                    info!(event = "connected", remote_peer_id = %peer_id,
                        connection_id = %connection_id, endpoint = ?endpoint);
                }
                Some(SwarmEvent::Behaviour(BehaviourEvent::Ping(ping::Event { peer, connection, result }))) => match result {
                    Ok(rtt) => info!(event = "ping_ok", remote_peer_id = %peer,
                        connection_id = %connection, rtt_us = rtt.as_micros() as u64),
                    Err(error) => warn!(event = "ping_error", remote_peer_id = %peer, error = ?error),
                },
                Some(SwarmEvent::ConnectionClosed { peer_id, connection_id, cause, .. }) => {
                    info!(event = "disconnected", remote_peer_id = %peer_id,
                        connection_id = %connection_id, cause = ?cause);
                }
                Some(SwarmEvent::OutgoingConnectionError { peer_id, error, .. }) => {
                    warn!(event = "dial_error", remote_peer_id = ?peer_id, error = ?error);
                }
                Some(SwarmEvent::IncomingConnectionError { error, .. }) => {
                    warn!(event = "incoming_error", error = ?error);
                }
                Some(SwarmEvent::ListenerError { error, .. }) => {
                    warn!(event = "listener_error", error = ?error);
                }
                Some(SwarmEvent::ListenerClosed { reason, .. }) => {
                    return Err(io::Error::other(format!("listener closed: {reason:?}")).into());
                }
                None => return Err(io::Error::other("swarm event stream ended").into()),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lab_addresses_are_explicit_and_local() {
        assert!(local_address("/ip4/127.0.0.1/tcp/0", false).is_ok());
        assert!(local_address("/ip4/127.0.0.1/tcp/4101", true).is_ok());
        for address in [
            "invalid",
            "/ip4/0.0.0.0/tcp/4101",
            "/ip4/192.168.1.1/tcp/4101",
            "/dns/localhost/tcp/4101",
            "/ip4/127.0.0.1/udp/4101",
            "/ip4/127.0.0.1/tcp/4101/ws",
        ] {
            assert!(local_address(address, true).is_err(), "{address}");
        }
        assert!(local_address("/ip4/127.0.0.1/tcp/0", true).is_err());
    }
}
