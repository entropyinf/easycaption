use enthalpy::Res;
use tokio::sync::watch;
use tokio::sync::watch::Ref;
use tracing::event;

#[derive(Clone)]
pub struct ConfigSync<T: Clone> {
    curr_rx: watch::Receiver<T>,
    curr_tx: watch::Sender<T>,
    new_rx: watch::Receiver<T>,
    new_tx: watch::Sender<T>,
}

impl<T: Clone + Send + Sync + 'static> ConfigSync<T> {
    pub fn new(initial: T) -> Self {
        let (curr_tx, curr_rx) = watch::channel(initial.clone());
        let (new_tx, new_rx) = watch::channel(initial);
        Self {
            curr_rx,
            curr_tx,
            new_rx,
            new_tx,
        }
    }

    pub async fn update_sync(&mut self, value: T) -> Res<()> {
        self.new_tx.send(value)?;
        self.curr_rx.changed().await?;
        Ok(())
    }

    pub async fn wait_update(&mut self) -> Res<()> {
        self.new_rx.changed().await?;
        Ok(())
    }

    /// Commit the new value to the current value
    pub async fn finish(&mut self, commit: bool) -> Res<()> {
        if commit {
            let new = self.fresh();
            event!(tracing::Level::DEBUG, "Commit");
            self.curr_tx.send(new.clone())?;
        } else {
            event!(tracing::Level::DEBUG, "Keep");
            self.curr_tx.send_modify(|_v|{});
        }
        Ok(())
    }

    pub fn fresh(&self) -> Ref<'_,T> {
        self.new_rx.borrow()
    }

    pub fn curr(&self) -> Ref<'_,T> {
        self.curr_rx.borrow()
    }
}