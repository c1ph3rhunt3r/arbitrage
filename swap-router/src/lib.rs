//! Atomic Swap Router Program for Solana Arbitrage
//! 
//! Executes multi-leg DEX swaps via CPI (Cross-Program Invocation) and guarantees
//! atomic profit assertion: reverts the entire transaction if final profit < min_output_amount.

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub const TAG_COMPRESSED_ARB: u8 = 4;

/// Reads the token balance from an SPL Token account (bytes 64..72) or native lamports
#[inline]
fn get_balance(account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = account.try_borrow_data()?;
    if data.len() >= 72 {
        // SPL Token account layout: amount is at bytes 64..72 (u64 little-endian)
        let amount_bytes: [u8; 8] = data[64..72]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(u64::from_le_bytes(amount_bytes))
    } else {
        // Fallback to native SOL lamports
        Ok(account.lamports())
    }
}

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let tag = instruction_data[0];
    match tag {
        TAG_COMPRESSED_ARB => process_compressed_arbitrage(accounts, &instruction_data[1..]),
        _ => {
            msg!("Error: Unknown instruction tag: {}", tag);
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

fn process_compressed_arbitrage(
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // Expected header: 8 bytes min_profit (u64 LE)
    if data.len() < 8 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let min_profit = u64::from_le_bytes(
        data[0..8]
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    // accounts[0] is the profit recipient token account (e.g. WSOL ATA)
    let profit_account = accounts
        .first()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;
    let initial_balance = get_balance(profit_account)?;

    let mut cursor = 8;
    let mut step = 0;

    while cursor < data.len() {
        if cursor + 2 > data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let num_indices = data[cursor] as usize;
        let data_len = data[cursor + 1] as usize;
        cursor += 2;

        if cursor + data_len + num_indices > data.len() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let raw_ix_data = &data[cursor..cursor + data_len];
        cursor += data_len;

        let indices = &data[cursor..cursor + num_indices];
        cursor += num_indices;

        if indices.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // The first index specifies the target DEX program ID
        let program_idx = indices[0] as usize;
        if program_idx >= accounts.len() {
            return Err(ProgramError::NotEnoughAccountKeys);
        }
        let target_program = &accounts[program_idx];

        // Map the remaining indices to instruction AccountMeta and AccountInfo references
        let mut ix_accounts = Vec::with_capacity(num_indices.saturating_sub(1));
        let mut account_infos = Vec::with_capacity(num_indices);
        account_infos.push(target_program.clone());

        for &idx in &indices[1..] {
            let acc_idx = idx as usize;
            if acc_idx >= accounts.len() {
                return Err(ProgramError::NotEnoughAccountKeys);
            }
            let acc = &accounts[acc_idx];
            ix_accounts.push(AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            });
            account_infos.push(acc.clone());
        }

        let swap_ix = Instruction {
            program_id: *target_program.key,
            accounts: ix_accounts,
            data: raw_ix_data.to_vec(),
        };

        // Execute swap leg via CPI
        invoke(&swap_ix, &account_infos)?;
        step += 1;
    }

    // Verify end-to-end profitability
    let final_balance = get_balance(profit_account)?;
    let actual_profit = final_balance.saturating_sub(initial_balance);

    if final_balance < initial_balance || actual_profit < min_profit {
        msg!(
            "Arbitrage reverted: expected min profit {}, actual profit {}",
            min_profit,
            actual_profit
        );
        // Custom error code 1: Slippage / Not enough profit
        return Err(ProgramError::Custom(1));
    }

    msg!(
        "Arbitrage success: {} legs executed, profit {} lamports",
        step,
        actual_profit
    );

    Ok(())
}
