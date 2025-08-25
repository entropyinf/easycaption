use futures_util::{SinkExt, StreamExt};
use tokio::io::AsyncReadExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{Bytes, Message};

#[tauri::command]
pub async fn serve() {
    let (ws_stream, response) = connect_async("ws://localhost:8000/asr").await.unwrap();
    println!("响应: {}", response.status());
    let (mut write, mut read) = ws_stream.split();

    tokio::spawn(async move {
        let mut audio = tokio::fs::File::open("audio.mp3").await.unwrap();
        let mut buf = Vec::with_capacity(1024 * 1024);

        let _size = audio.read_to_end(&mut buf).await.unwrap();

        let bytes = Bytes::copy_from_slice(&buf);

        write.send(Message::Binary(bytes)).await.unwrap();
    });

    loop {
        if let Some(msg) = read.next().await {
            let msg = msg.unwrap();
            if msg.is_text() {
                println!("收到响应: {}", msg.into_text().unwrap());
            }
        } else {
            break;
        }
    }
}