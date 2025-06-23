use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Signer, keypair::Keypair},
    system_instruction,
    transaction::Transaction,
};
use pinocchio_counter::process_instruction;

#[tokio::test]
async fn test_counter() {
    // Program ID from deployment
    let program_id = Pubkey::new_unique();
    
    // Start test validator
    let mut program_test = ProgramTest::new(
        "pinocchio_counter",
        program_id,
        processor!(process_instruction),
    );
    
    // Start test client
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // Create counter account
    let counter_account = Keypair::new();
    let space = 4; // u32 takes 4 bytes
    let rent = banks_client.get_rent().await.unwrap();
    let rent_lamports = rent.minimum_balance(space);
    
    // Create account transaction
    let mut transaction = Transaction::new_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &counter_account.pubkey(),
                rent_lamports,
                space as u64,
                &program_id,
            ),
        ],
        Some(&payer.pubkey()),
    );
    
    transaction.sign(&[&payer, &counter_account], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify initial counter is 0
    let account = banks_client.get_account(counter_account.pubkey()).await.unwrap().unwrap();
    let count = u32::from_le_bytes(account.data[..4].try_into().unwrap());
    assert_eq!(count, 0);

    // Call increment instruction
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &(), // Empty instruction data
            vec![AccountMeta::new(counter_account.pubkey(), false)],
        )],
        Some(&payer.pubkey()),
    );
    
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Verify counter is now 1
    let account = banks_client.get_account(counter_account.pubkey()).await.unwrap().unwrap();
    let count = u32::from_le_bytes(account.data[..4].try_into().unwrap());
    assert_eq!(count, 1);
}
