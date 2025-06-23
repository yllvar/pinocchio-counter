use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signer::keypair::read_keypair_file,
    system_instruction,
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;

const PROGRAM_ID: &str = "DzrwGUTH6ZsQcCMYTayJ1b5rorTPdXvcxeTzfTavMqm2";
const RPC_URL: &str = "http://127.0.0.1:8899";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to localnet
    let client = RpcClient::new_with_commitment(RPC_URL, CommitmentConfig::confirmed());
    
    // Use local validator's default keypair
    let payer = read_keypair_file(&*shellexpand::tilde("~/.config/solana/id.json"))
        .expect("Failed to read keypair file");
    
    // Check balance
    let balance = client.get_balance(&payer.pubkey())?;
    println!("Payer balance: {}", balance);
    
    if balance < 100_000_000 {
        client.request_airdrop(&payer.pubkey(), 100_000_000)?;
        // Wait for airdrop confirmation
        loop {
            let new_balance = client.get_balance(&payer.pubkey())?;
            if new_balance > balance {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
    
    // Create counter account
    let counter_account = Keypair::new();
    let space = 4; // u32 takes 4 bytes
    let rent = client.get_minimum_balance_for_rent_exemption(space)?;
    
    let program_id = PROGRAM_ID.parse::<Pubkey>()?;
    
    // Create account transaction
    let create_ix = system_instruction::create_account(
        &payer.pubkey(),
        &counter_account.pubkey(),
        rent,
        space as u64,
        &program_id,
    );
    
    let mut transaction = Transaction::new_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
    );
    
    let recent_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[&payer, &counter_account], recent_blockhash);
    client.send_and_confirm_transaction(&transaction)?;
    
    println!("Created counter account: {}", counter_account.pubkey());
    
    // Increment counter
    let increment_ix = Instruction::new_with_bincode(
        program_id,
        &(), // Empty instruction data
        vec![AccountMeta::new(counter_account.pubkey(), false)],
    );
    
    let mut transaction = Transaction::new_with_payer(
        &[increment_ix],
        Some(&payer.pubkey()),
    );
    
    let recent_blockhash = client.get_latest_blockhash()?;
    transaction.sign(&[&payer], recent_blockhash);
    client.send_and_confirm_transaction(&transaction)?;
    
    println!("Incremented counter");
    
    // Get counter value
    let account = client.get_account(&counter_account.pubkey())?;
    let count = u32::from_le_bytes(account.data[..4].try_into().unwrap());
    println!("Current counter value: {}", count);
    
    Ok(())
}
