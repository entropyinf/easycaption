use crate::audio::audio::record;
use crate::common::ResourceGuard;
use crate::AppData;
use bytes::{Bytes, BytesMut};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::io;
use std::io::{Seek, SeekFrom, Write};
use tauri::{AppHandle, Emitter, State};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

#[tauri::command]
pub async fn transcribe(app: AppHandle, state: State<'_, Mutex<AppData>>) -> Result<(), ()> {
    let (ws_stream, response) = connect_async("ws://localhost:8000/asr").await.unwrap();
    println!("Connection status : {}", response.status());

    let (write, read) = ws_stream.split();

    recv_text(read, app.clone());

    let write = WebSocketWrap::new(write);
    let stream_handle = record(write).unwrap();

    println!("Streaming started");
    state.lock().await.audio_stream_handle = Some(ResourceGuard::new(stream_handle));

    Ok(())
}

#[tauri::command]
pub async fn stop_transcribe(state: State<'_, Mutex<AppData>>) -> Result<(), ()> {
    state.lock().await.audio_stream_handle = None;
    Ok(())
}

pub struct WebSocketWrap {
    write: mpsc::Sender<Bytes>,
}

type WebSocketWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;

impl WebSocketWrap {
    pub fn new(sender: WebSocketWrite) -> Self {
        let (ch, mut rx) = mpsc::channel::<Bytes>(1024 * 16);

        tokio::spawn(async move {
            let mut sender = sender;
            let mut buffer = BytesMut::new();
            const BUFFER_THRESHOLD: usize = 1024 * 40;

            loop {
                tokio::select! {
                    // 接收新的数据块
                    received = rx.recv() => {
                        match received {
                            None => {
                                // channel已关闭，发送剩余缓冲区内容
                                if !buffer.is_empty() {
                                    if let Err(e) = sender.send(Message::Binary(buffer.to_vec().into())).await {
                                        eprintln!("Error sending message to websocket: {}", e);
                                    }
                                }
                                eprintln!("Channel closed");
                                break;
                            }
                            Some(bytes) => {
                                // 将数据添加到缓冲区
                                buffer.extend_from_slice(&bytes);

                                // 如果缓冲区达到阈值，则发送
                                if buffer.len() >= BUFFER_THRESHOLD {
                                    if let Err(e) = sender.send(Message::Binary(buffer.to_vec().into())).await {
                                        eprintln!("Error sending message to websocket: {}", e);
                                        break;
                                    }
                                    buffer.clear();
                                }
                            }
                        }
                    }
                    // 定时发送缓冲区内容（例如每100ms）
                    _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                        if !buffer.is_empty() {
                            if let Err(e) = sender.send(Message::Binary(buffer.to_vec().into())).await {
                                eprintln!("Error sending message to websocket: {}", e);
                                break;
                            }
                            buffer.clear();
                        }
                    }
                }
            }
        });

        Self { write: ch }
    }
}

impl Write for WebSocketWrap {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes = Bytes::copy_from_slice(buf);
        let size = self.write.capacity();
        // println!("Sending {} bytes to websocket", size);
        if let Err(e) = self.write.try_send(bytes) {
            // eprintln!("Error sending message to websocket: {}", e);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Seek for WebSocketWrap {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Ok(0)
    }
}

type WebsocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub(crate) fn recv_text(mut read: WebsocketRead, app: AppHandle) {
    tokio::spawn(async move {
        loop {
            match read.next().await {
                None => {
                    eprintln!("Connection closed");
                    break;
                }
                Some(Ok(msg)) => {
                    if msg.is_text() {
                        let json = msg.to_text().unwrap_or("");
                        println!("Received message: {}", json);
                        let value: Option<Value> = serde_json::from_str(json).ok();

                        if let Err(e) = app.emit("caption", value) {
                            eprintln!("Error emitting event: {}", e);
                        };
                    } else if msg.is_close() {
                        println!("WebSocket连接已关闭");
                        break;
                    }
                }
                Some(Err(e)) => {
                    eprintln!("Error receiving message from websocket: {}", e);
                }
            }
        }
    });
}
