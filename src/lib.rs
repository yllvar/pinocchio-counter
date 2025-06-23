use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Declare the program's entrypoint
entrypoint!(process_instruction);

// Counter account data structure
#[repr(C)]
#[derive(Default)]
pub struct CounterAccount {
    pub count: u32,
}

// Program entrypoint implementation
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Get the account iterator
    let accounts_iter = &mut accounts.iter();

    // Get the counter account
    let counter_account = next_account_info(accounts_iter)?;

    // Verify the account is owned by our program
    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Mutably reference the account data
    let mut data = counter_account.try_borrow_mut_data()?;
    if data.len() < 4 {
        return Err(ProgramError::InvalidAccountData);
    }

    // Deserialize the counter
    let mut counter = CounterAccount::default();
    unsafe {
        let count_ptr = data.as_mut_ptr() as *mut u32;
        counter.count = *count_ptr;
    }

    // Increment the counter
    counter.count = counter.count.saturating_add(1);

    // Serialize the counter back
    unsafe {
        let count_ptr = data.as_mut_ptr() as *mut u32;
        *count_ptr = counter.count;
    }

    Ok(())
}

// Tests can be added later once we have proper test infrastructure
