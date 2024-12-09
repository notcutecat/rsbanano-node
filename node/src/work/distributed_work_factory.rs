use rsban_core::{
    to_hex_string,
    work::{WorkPool, WorkPoolImpl},
    Account, Block, Root,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::oneshot;

#[derive(Serialize)]
pub struct HttpWorkRequest {
    action: &'static str,
    hash: String,
    difficulty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    account: Option<String>,
}

impl HttpWorkRequest {
    pub fn new(root: Root, difficulty: u64, account: Option<Account>) -> Self {
        Self {
            action: "work_generate",
            hash: root.to_string(),
            difficulty: to_hex_string(difficulty),
            account: account.map(|a| a.encode_account()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HttpWorkResponse {
    work: String,
}

#[derive(Clone)]
pub struct WorkRequest {
    pub root: Root,
    pub difficulty: u64,
    pub account: Option<Account>,
    pub peers: Vec<(String, u16)>,
}

impl WorkRequest {
    pub fn new_test_instance() -> Self {
        Self {
            root: Root::from(100),
            difficulty: 42,
            account: Some(Account::from(200)),
            peers: vec![("127.0.0.1".to_string(), 9999)],
        }
    }
}

pub struct DistributedWorkFactory {
    work_pool: Arc<WorkPoolImpl>,
    pub tokio: tokio::runtime::Handle,
}

impl DistributedWorkFactory {
    pub fn new(work_pool: Arc<WorkPoolImpl>, tokio: tokio::runtime::Handle) -> Self {
        Self { work_pool, tokio }
    }

    pub fn make_blocking_block(&self, block: &mut Block, difficulty: u64) -> Option<u64> {
        let work = self.tokio.block_on(self.generate_work(WorkRequest {
            root: block.root(),
            difficulty,
            account: None,
            peers: Vec::new(),
        }));

        if let Some(work) = work {
            block.set_work(work);
        }

        work
    }

    pub fn make_blocking(
        &self,
        root: Root,
        difficulty: u64,
        account: Option<Account>,
    ) -> Option<u64> {
        self.tokio.block_on(self.generate_work(WorkRequest {
            root,
            difficulty,
            account,
            peers: Vec::new(),
        }))
    }

    pub async fn make(&self, root: Root, difficulty: u64, account: Option<Account>) -> Option<u64> {
        self.generate_work(WorkRequest {
            root,
            difficulty,
            account,
            peers: Vec::new(),
        })
        .await
    }

    async fn generate_work(&self, request: WorkRequest) -> Option<u64> {
        self.generate_in_local_work_pool(request.root, request.difficulty)
            .await
    }

    async fn generate_in_local_work_pool(&self, root: Root, difficulty: u64) -> Option<u64> {
        let (tx, rx) = oneshot::channel::<Option<u64>>();
        self.work_pool.generate_async(
            root,
            difficulty,
            Some(Box::new(move |work| {
                tx.send(work).unwrap();
            })),
        );
        rx.await.ok()?
    }

    pub fn cancel(&self, root: Root) {
        self.work_pool.cancel(&root);
    }

    pub fn work_generation_enabled(&self) -> bool {
        self.work_pool.work_generation_enabled()
    }

    pub fn stop(&self) {
        //TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rsban_core::work::WorkPoolImpl;
    use std::sync::Arc;

    #[tokio::test]
    async fn use_local_work_factor_when_no_peers_given() {
        let expected_work = 12345;
        let work_pool = Arc::new(WorkPoolImpl::new_null(expected_work));
        let work_factory =
            DistributedWorkFactory::new(work_pool, tokio::runtime::Handle::current());

        let request = WorkRequest {
            peers: vec![],
            ..WorkRequest::new_test_instance()
        };

        let work = work_factory.generate_work(request.clone()).await;

        assert_eq!(work, Some(expected_work));
    }

    // TODO:
    // Backoff + Workrequest
    // Cancel
    // Local work
    // resolve hostnames
    // multiple peers
    // secondary peers
    // work generation disabled
    // unresponsive work peers => use local work
}
