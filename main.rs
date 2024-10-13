use stellar_sdk::{
    types::{Keypair, Network},
    Transaction,
    memo::Memo,
    operation::{Payment, Operation},
    asset::Asset,
};
use serde_json;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::time::{sleep, Duration};

fn string_to_memo_bytes(memo: &str) -> [u8; 28] {
    let mut bytes = [0u8; 28];
    let memo_bytes = memo.as_bytes();
    for (i, &byte) in memo_bytes.iter().take(28).enumerate() {
        bytes[i] = byte;
    }
    bytes
}

#[derive(Serialize, Deserialize, Debug)]
struct TransactionResult {
    id: String,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sender_public_key = "GCOVRUBRTUKGR7SSJMC4KK63QYKPO5JJDQYDA5GRFKPUK4FD3RCY5ZN4";  // Sender public key
    let sender_secret = "GCOVRUBRTUKGR7SSJMC4KK63QYKPO5JJDQYDA5GRFKPUK4FD3RCY5ZN4";  // Sender secret key
    let receiver_public_key = "GDRF4Z6N2TGHU2B7A4YLVIZ5V2F5K73NDRQ4ZDJNVQZIEIC5W57V4O3"; // Receiver public key

    // KeyPair from the secret and public key
    let sender = Keypair::from_secret(sender_secret);

    // Create the payment operation
    let payment = Payment::new(
        &receiver_public_key.to_string(),
        Asset::native(),
        10.0,
    );

    // Create transaction
    let tx = Transaction::new(
        &sender,
        0, // You need to specify the correct sequence number
        vec![Operation::Payment(payment)],
        Memo::Text("Ödeme Yapıldı"),
    );

    // Send to Horizon API
    let server = Server::new("https://horizon-testnet.stellar.org");
    let tx_response = server.submit_transaction(&tx)?;

    println!("Transaction response: {:?}", tx_response);

    Ok(())
}

async fn send_payment(
    source_secret: &str,
    destination_public_key: &str,
    amount: f32,
    memo_message: &str,
) -> Result<(), Box<dyn Error>> {
    let source_account = Keypair::from_secret(source_secret);

    // İşlem oluşturuluyor
    let payment_op = Payment::new(destination_public_key.to_string(), Asset::native(), amount);
    let transaction = Transaction::new(
        &source_account,
        0, // Sequence number needs to be fetched from the network or local state
        vec![Operation::Payment(payment_op)],
        Memo::Text(memo_message),
    );

    // Horizon API ile işlemi gönderme
    let server = Server::new("https://horizon-testnet.stellar.org");
    let tx_response = server.submit_transaction(&transaction)?;

    println!("Transaction submitted successfully: {}", tx_response.id());
    Ok(())
}

async fn send_payment_to_multiple_recipients(
    source_secret: &str,
    recipients: Vec<(&str, f32)>,
) -> Result<(), Box<dyn Error>> {
    let source_account = Keypair::from_secret(source_secret);

    let operations: Vec<Operation> = recipients.iter().map(|(recipient, amount)| {
        Operation::Payment(Payment::new(recipient.to_string(), Asset::native(), *amount))
    }).collect();

    let transaction = Transaction::new(
        &source_account,
        0, // Sequence number needs to be fetched from the network or local state
        operations,
        Memo::Text("Payment to multiple recipients"),
    );

    let server = Server::new("https://horizon-testnet.stellar.org");
    let tx_response = server.submit_transaction(&transaction)?;

    println!("Transaction to multiple recipients submitted: {}", tx_response.id());
    Ok(())
}

async fn schedule_regular_payments(
    source_secret: &str,
    recipients: Vec<(&str, f32)>,
) {
    loop {
        sleep(Duration::from_secs(86400)).await; // 24 saat bekleme
        println!("Starting regular payment...");

        match send_payment_to_multiple_recipients(source_secret, recipients.clone()).await {
            Ok(_) => println!("Payment sent successfully"),
            Err(e) => println!("Failed to send payment: {}", e),
        }
    }
}

async fn check_balance(public_key: &str) -> Result<f32, Box<dyn Error>> {
    let client = Client::new();
    let server_url = "https://horizon-testnet.stellar.org";
    let account_url = format!("{}/accounts/{}", server_url, public_key);

    // Hesap bakiyesi sorgulaması yapılıyor
    let account_response: serde_json::Value = client
        .get(&account_url)
        .send()
        .await?
        .json()
        .await?;

    let balance = account_response["balances"]
        .as_array()
        .ok_or("Invalid balance response")?
        .iter()
        .find(|balance| balance["asset_type"] == "native")
        .ok_or("Native balance not found")?["balance"]
        .as_str()
        .ok_or("Invalid balance format")?;

    Ok(balance.parse()?)
}
