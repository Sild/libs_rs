use crate::clients::tonlibjson::client_raw::tl_request::TLRequest;
use crate::clients::tonlibjson::client_raw::tl_response::TLResponse;
use crate::clients::tonlibjson::client_raw::tl_types::TLKeyStoreType;
use crate::clients::tonlibjson::clients_impl::TLJConnection;
use crate::clients::tonlibjson::tlj_client::TLJClient;
use crate::clients::tonlibjson::tlj_config::TLJClientConfig;
use crate::clients::tonlibjson::tlj_utils::update_init_block;
use crate::errors::TonlibError;
use async_trait::async_trait;
use rand::prelude::{IndexedRandom, StdRng};
use rand::SeedableRng;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, Semaphore};

/// Simple client with many connections
pub struct TLJClientDefault(Arc<Inner>);

impl TLJClientDefault {
    pub async fn new(mut config: TLJClientConfig) -> Result<Self, TonlibError> {
        if config.update_init_block {
            let timeout = Duration::from_secs(config.update_init_block_timeout_sec);
            let new_config = update_init_block(&config.init_opts.config.net_config, timeout).await?;
            config.init_opts.config.net_config = new_config;
        }

        if let TLKeyStoreType::Directory { directory } = &config.init_opts.keystore_type {
            std::fs::create_dir_all(directory)?
        }
        let semaphore = Arc::new(Semaphore::new(config.max_parallel_requests));
        let mut connections = Vec::with_capacity(config.connections_count);
        for _ in 0..config.connections_count {
            let connection = TLJConnection::new(&config, semaphore.clone()).await?;
            connections.push(connection);
        }
        let inner = Inner {
            rnd: Mutex::new(StdRng::from_rng(&mut rand::rng())),
            connections,
        };
        Ok(TLJClientDefault(Arc::new(inner)))
    }
}

#[async_trait]
impl TLJClient for TLJClientDefault {
    async fn get_connection(&self) -> Result<&dyn TLJClient, TonlibError> {
        let mut rng_lock = self.0.rnd.lock().await;
        let conn = self.0.connections.choose(&mut rng_lock.deref_mut()).unwrap();
        Ok(conn)
    }

    async fn exec_impl(&self, _req: &TLRequest) -> Result<TLResponse, TonlibError> {
        Err(TonlibError::TLJWrongUsage("exec_impl can be called only on TLJConnection".to_string()))
    }
}

struct Inner {
    rnd: Mutex<StdRng>,
    connections: Vec<TLJConnection>,
}
