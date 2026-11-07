mod pb;
use pb::transfers::v1::{Transfer, Transfers};
use substreams_solana::pb::sf::solana::r#type::v1::Block;

const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
const TOKEN_PROGRAM: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
const TOKEN_2022_PROGRAM: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

#[substreams::handlers::map]
fn map_transfers(blk: Block) -> Result<Transfers, substreams::errors::Error> {
    let mut transfers = Vec::new();

    for trx in blk.transactions() {
        for view in trx.walk_instructions() {
            let program = bs58::encode(view.program_id()).into_string();
            let data = view.data();
            let accts = view.accounts();

            if program == SYSTEM_PROGRAM {
                // System Transfer: u32 LE tag 2, then u64 LE lamports
                if data.len() == 12 && data[0..4] == [2, 0, 0, 0] && accts.len() >= 2 {
                    transfers.push(Transfer {
                        from: bs58::encode(&accts[0]).into_string(),
                        to: bs58::encode(&accts[1]).into_string(),
                        amount: u64::from_le_bytes(data[4..12].try_into().unwrap()),
                    });
                }
            } else if program == TOKEN_PROGRAM || program == TOKEN_2022_PROGRAM {
                // SPL Transfer: tag 3, u64 amount. Accounts: source, destination, authority
                if data.len() >= 9 && data[0] == 3 && accts.len() >= 2 {
                    transfers.push(Transfer {
                        from: bs58::encode(&accts[0]).into_string(),
                        to: bs58::encode(&accts[1]).into_string(),
                        amount: u64::from_le_bytes(data[1..9].try_into().unwrap()),
                    });
                // SPL TransferChecked: tag 12, u64 amount, u8 decimals. Accounts: source, mint, destination, authority
                } else if data.len() >= 10 && data[0] == 12 && accts.len() >= 3 {
                    transfers.push(Transfer {
                        from: bs58::encode(&accts[0]).into_string(),
                        to: bs58::encode(&accts[2]).into_string(),
                        amount: u64::from_le_bytes(data[1..9].try_into().unwrap()),
                    });
                }
            }
        }
    }
    Ok(Transfers { transfers })
}