use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::{AccountMeta, Instruction},
    program::invoke,
    pubkey::Pubkey,
    program_error::ProgramError,
};
use std::convert::TryInto;

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Ok(());
    }

    let tag = instruction_data[0];
    let mut cursor = 9;

    if tag == 4 {
        // tag 4: Compressed Indexed Instructions
        while cursor < instruction_data.len() {
            let indices_len = instruction_data[cursor] as usize;
            cursor += 1;
            
            let ix_data_len = instruction_data[cursor] as usize;
            cursor += 1;
            
            let raw_ix_data = &instruction_data[cursor..cursor + ix_data_len];
            cursor += ix_data_len;
            
            let indices = &instruction_data[cursor..cursor + indices_len];
            cursor += indices_len;

            execute_cpi(raw_ix_data, indices, accounts)?;
        }
    } else if tag == 3 {
        // tag 3: Uncompressed Sequential Instructions
        while cursor < instruction_data.len() {
            let num_accounts = instruction_data[cursor] as usize;
            cursor += 1;

            let ix_data_len = instruction_data[cursor] as usize;
            cursor += 1;

            let raw_ix_data = &instruction_data[cursor..cursor + ix_data_len];
            cursor += ix_data_len;

            // Extract the sequential accounts directly
            let mut cpi_account_metas = Vec::with_capacity(num_accounts - 1);
            let mut cpi_account_infos = Vec::with_capacity(num_accounts);

            let mut accounts_cursor = cursor;
            
            // Program ID is the first account
            let program_account_info = &accounts[accounts_cursor];
            cpi_account_infos.push(program_account_info.clone());
            accounts_cursor += 1;

            for _ in 1..num_accounts {
                let is_signer = instruction_data[accounts_cursor] == 1;
                accounts_cursor += 1;
                let is_writable = instruction_data[accounts_cursor] == 1;
                accounts_cursor += 1;
                
                let acc_info = &accounts[accounts_cursor];
                cpi_account_metas.push(if is_writable {
                    AccountMeta::new(*acc_info.key, is_signer)
                } else {
                    AccountMeta::new_readonly(*acc_info.key, is_signer)
                });
                
                cpi_account_infos.push(acc_info.clone());
                accounts_cursor += 1;
            }
            cursor = accounts_cursor;

            execute_cpi(raw_ix_data, &[], &cpi_account_infos)?; // Need to fix this for sequential
            // Actually tag 3 is barely used based on my search, but I should implement it fully if I need to.
            // But let's refine execute_cpi first!
        }
    }

    Ok(())
}

fn execute_cpi(
    raw_ix_data: &[u8],
    indices: &[u8],
    accounts: &[AccountInfo],
) -> ProgramResult {
    if raw_ix_data.is_empty() {
        return Ok(());
    }

    if accounts.is_empty() {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let mut data = raw_ix_data.to_vec();
    let program_id_index;
    let accounts_start_index;

    // Determine if it's obfuscated and how to parse accounts
    if !indices.is_empty() {
        let obf_tag = raw_ix_data[0];
        
        if obf_tag == 1 {
            // Simple obfuscated swap
            data = raw_ix_data[1..].to_vec();
            data.reverse();
            program_id_index = indices[0] as usize;
            accounts_start_index = 1;
        } else if obf_tag == 2 {
            // Swap with close ATA
            data = raw_ix_data[1..].to_vec();
            let offset_bytes: [u8; 2] = [data[0], data[1]];
            let offset = u16::from_le_bytes(offset_bytes) as usize;
            data.drain(0..2);
            
            let token_account_index = indices[0] as usize;
            let token_account_info = &accounts[token_account_index];
            
            // Dynamically get the token balance
            // token balance is stored at offset 64 in spl-token Account struct
            let balance_bytes: [u8; 8] = token_account_info.try_borrow_data()?[64..72].try_into().unwrap();
            
            // Insert balance into data at offset
            if data.len() >= offset + 8 {
                data[offset..offset+8].copy_from_slice(&balance_bytes);
            }
            
            program_id_index = indices[1] as usize;
            accounts_start_index = 2;
        } else {
            // Unobfuscated or standard
            program_id_index = indices[0] as usize;
            accounts_start_index = 1;
        }

        if indices.len() < accounts_start_index {
            return Err(ProgramError::InvalidInstructionData);
        }

        let program_account_info = &accounts[program_id_index];
        let mut cpi_account_metas = Vec::with_capacity(indices.len() - accounts_start_index);
        let mut cpi_account_infos = Vec::with_capacity(indices.len() - accounts_start_index + 1);
        
        cpi_account_infos.push(program_account_info.clone());

        for i in accounts_start_index..indices.len() {
            let idx = indices[i] as usize;
            let acc_info = &accounts[idx];
            
            cpi_account_metas.push(if acc_info.is_writable {
                AccountMeta::new(*acc_info.key, acc_info.is_signer)
            } else {
                AccountMeta::new_readonly(*acc_info.key, acc_info.is_signer)
            });
            
            cpi_account_infos.push(acc_info.clone());
        }

        let instruction = Instruction {
            program_id: *program_account_info.key,
            accounts: cpi_account_metas,
            data,
        };

        invoke(&instruction, &cpi_account_infos)
    } else {
        // Sequential mode: accounts array IS the cpi_account_infos
        // Data is always unobfuscated because obfuscation requires indices
        let program_account_info = &accounts[0];
        let mut cpi_account_metas = Vec::with_capacity(accounts.len() - 1);
        
        for acc_info in accounts.iter().skip(1) {
            cpi_account_metas.push(if acc_info.is_writable {
                AccountMeta::new(*acc_info.key, acc_info.is_signer)
            } else {
                AccountMeta::new_readonly(*acc_info.key, acc_info.is_signer)
            });
        }

        let instruction = Instruction {
            program_id: *program_account_info.key,
            accounts: cpi_account_metas,
            data,
        };

        invoke(&instruction, accounts)
    }
}
