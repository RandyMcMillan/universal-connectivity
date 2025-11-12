use rust_libp2p_webrtc_peer::prelude::*;

use anyhow::Result;
use clap::Parser;
use libp2p::{identity, PeerId};
use libp2p_webrtc::tokio::Certificate;
use std::path::{Path, PathBuf};
use tokio::{fs, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tracing::info;
use rust_libp2p_webrtc_peer::util::is_valid_git_commit_hash;

#[tokio::main]
async fn main() -> Result<()> {
    // parse the command line arguments
    let opt = Options::parse();

    // initialize the tracing logger and get the receiver for log messages
    let from_log = Log::init();

    // create a shutdown token
    let shutdown = CancellationToken::new();

    // load the identity and certificate
    let local_key = read_or_create_identity(&opt.local_key_path).await?;
    let webrtc_cert = read_or_create_certificate(&opt.local_cert_path).await?;

    // create the ui and the channels to communicate with it
    let git_commit_message = if let Some(topic) = &opt.topic {
        if is_valid_git_commit_hash(topic) {
            let output = tokio::process::Command::new("git")
                .arg("show")
                .arg("-s")
                .arg("--format=%B")
                .arg(topic)
                .output()
                .await?;
            if output.status.success() {
                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                info!("Failed to get git commit message for topic: {}", topic);
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let (mut ui, to_ui, from_ui) = if opt.headless {
        Headless::build(local_key.public().into(), from_log, shutdown.clone())
    } else {
        Tui::build(local_key.public().into(), from_log, shutdown.clone(), opt.topic, git_commit_message)
    };

    // create the peer, connecting it to the ui
    let mut peer = Peer::new(local_key, webrtc_cert, to_ui, from_ui, shutdown.clone()).await?;

    // spawn tasks for both the swarm and the ui
    let peer_task: JoinHandle<Result<()>> = tokio::spawn(async move { peer.run().await });
    let ui_task: JoinHandle<Result<()>> = tokio::spawn(async move { ui.run().await });

    // wait for the tasks to finish
    let (ui_result, peer_result) = tokio::try_join!(peer_task, ui_task)?;

    // check the inner results
    ui_result?;
    peer_result?;

    Ok(())
}

async fn read_or_create_certificate(path: &Path) -> Result<Certificate> {
    if path.exists() {
        let pem = fs::read_to_string(&path).await?;

        info!("Using existing certificate from {}", path.display());

        return Ok(Certificate::from_pem(&pem)?);
    }

    let cert = Certificate::generate(&mut rand::thread_rng())?;
    fs::write(&path, &cert.serialize_pem().as_bytes()).await?;

    info!(
        "Generated new certificate and wrote it to {}",
        path.display()
    );

    Ok(cert)
}

async fn read_or_create_identity(path: &Path) -> Result<identity::Keypair> {
    let mut key_path = PathBuf::from(path);
    let is_key = key_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "key")
        .unwrap_or(false);
    if !is_key {
        key_path.set_extension("key");
    }

    let mut peer_id_path = PathBuf::from(path);
    let is_peer_id = peer_id_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "peerid")
        .unwrap_or(false);
    if !is_peer_id {
        peer_id_path.set_extension("peerid");
    }

    if key_path.exists() {
        let bytes = fs::read(&key_path).await?;
        info!("Using existing identity from {}", key_path.display());
        // This only works for ed25519 but that is what we are using
        return Ok(identity::Keypair::from_protobuf_encoding(&bytes)?);
    }

    let identity = identity::Keypair::generate_ed25519();
    fs::write(&key_path, &identity.to_protobuf_encoding()?).await?;
    let peer_id: PeerId = identity.public().into();
    fs::write(&peer_id_path, peer_id.to_string()).await?;

    info!(
        "Generated new identity and wrote it to {}",
        key_path.display()
    );

    Ok(identity)
}
