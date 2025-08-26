use std::fs::File;
use crate::audio::audio::record;
use bytes::Bytes;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

#[tauri::command]
pub async fn transcribe() {
    // let (ws_stream, response) = connect_async("ws://localhost:8000/asr").await.unwrap();
    // println!("received from : {}", response.status());

    tokio::task::spawn_blocking(move || {
        // let (write, read) = ws_stream.split();

        let fs = File::create("audio.wav").unwrap();

        // let write = WebSocketWriteWrapper::new(write);
        let stream_handle = record(fs).unwrap();
        // recv_text(read);

        std::thread::sleep(Duration::from_secs(15));
        drop(stream_handle);
    });
}

pub struct WebSocketWriteWrapper {
    ch: watch::Sender<Bytes>,
}

type WebSocketWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

impl WebSocketWriteWrapper {
    pub fn new(sender: WebSocketWrite) -> Self {
        let (ch, mut rx) = watch::channel(Bytes::new());
        tokio::spawn(async move {
            let mut sender = sender;
            loop {
                if rx.changed().await.is_err() {
                    return;
                }

                let bytes = rx.borrow().clone();
                if let Err(e) = sender.send(Message::Binary(bytes)).await{
                    eprintln!("Error sending message: {}", e);
                    return;
                }
            }
        });

        Self { ch }
    }
}

impl Write for WebSocketWriteWrapper {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes = Bytes::copy_from_slice(buf);
        let _ = self.ch.send(bytes);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for WebSocketWriteWrapper {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Ok(0)
    }
}

type WebsocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
fn recv_text(mut read: WebsocketRead) {
    tokio::spawn(async move {
        loop {
            if let Some(msg) = read.next().await {
                let msg = msg.unwrap();
                if msg.is_text() {
                    println!("收到响应: {}", msg.into_text().unwrap());
                } else if msg.is_close() {
                    println!("WebSocket连接已关闭");
                    break;
                }
            } else {
                break;
            }
        }
    });
}
