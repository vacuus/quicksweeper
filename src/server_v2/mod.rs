use tokio::{runtime::Runtime, net::TcpListener};

#[allow(dead_code)]
pub fn srv_start(address: String) {
    Runtime::new().unwrap().block_on(srv_main(address))
}

async fn srv_main(address: String) {
    let listener = TcpListener::bind(address).await.unwrap();

    while let Ok(sock) = listener.accept().await {
        tokio::spawn(async {
            
        });
    }
}


