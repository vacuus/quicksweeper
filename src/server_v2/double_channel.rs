use tokio::sync::mpsc::{self, error::SendError};

pub struct DoubleChannel<T> {
    sender: mpsc::UnboundedSender<T>,
    receiver: mpsc::UnboundedReceiver<T>,
}

impl <T> DoubleChannel<T> {
    pub fn double() -> (Self, Self) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, rx2) = mpsc::unbounded_channel();

        (
            Self {
                sender: tx1, receiver: rx2,
            },
            Self {
                sender: tx2, receiver: rx1,
            }
        )
    }

    pub fn send(&mut self, message: T) -> Result<(), SendError<T>>{
        self.sender.send(message)
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}