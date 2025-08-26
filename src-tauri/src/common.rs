use std::sync::Arc;
use tokio::sync::Notify;

pub type Res<T> = anyhow::Result<T>;

pub struct ResourceGuard(Arc<Notify>);

struct Guard<T>(T);

unsafe impl<T> Send for Guard<T> {}

impl ResourceGuard {
    pub fn new<T: 'static>(v: T) -> Self {
        let notify = Arc::new(Notify::new());

        let n = notify.clone();

        let val = Guard(v);

        tokio::spawn(async move {
            n.notified().await;
            drop(val);
        });

        Self(notify)
    }
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        self.0.notify_one();
    }
}
