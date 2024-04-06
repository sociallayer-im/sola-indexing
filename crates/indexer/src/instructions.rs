use anchor_lang::prelude::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use sea_orm::{ActiveValue::Set, EntityTrait};
use solana_indexer::CbResult;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    MintBadge,
    Register,
    Unknow,
}

impl Instruction {
    pub fn new(data: &[u8]) -> (Self, &[u8]) {
        if data.len() < 8 {
            return (Instruction::Unknow, data);
        }
        let mut ix_data: &[u8] = data;
        let sighash: [u8; 8] = {
            let mut sighash: [u8; 8] = [0; 8];
            sighash.copy_from_slice(&ix_data[..8]);
            ix_data = &ix_data[8..];
            sighash
        };

        let instruction = match sighash {
            Self::MINT_BADGE => Self::MintBadge,
            Self::REGISTER => Self::Register,
            _ => Self::Unknow,
        };

        (instruction, ix_data)
    }

    const MINT_BADGE: [u8; 8] = [242, 234, 237, 183, 232, 245, 146, 1];
    const REGISTER: [u8; 8] = [211, 124, 67, 15, 211, 194, 178, 240];
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Register {
    pub class_id: u64,
    pub profile_id: u64,
    pub params: RegisterParams,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct RegisterParams {
    // metadata里可以直接查询到，从sdk里提供该字段即可
    pub fungible: bool,
    // spl_extension里可以查询到，同样在sdk里提供即可
    pub transferable: bool,
    pub revocable: bool,
    pub address: Pubkey,
    pub schema: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MintBadge {
    pub badge_id: u64,
    pub class_id: u64,
    pub origins: Vec<u64>,
    pub params: MintBadgeParams,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MintBadgeParams {
    pub name: String,
    pub creators: Vec<CreatorsParam>,
    pub seller_fee_basis_points: u16,
    pub symbol: String,
    pub uri: String,
    pub is_mutable: bool,
    pub weights: u64,
    pub schema: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CreatorsParam {
    pub address: Pubkey,
    pub share: u8,
}

pub async fn handle_instruction(
    instruction: &solana_indexer::Instruction,
    db: &sea_orm::DatabaseConnection,
) -> CbResult {
    let Ok(data) = base58::FromBase58::from_base58(instruction.data.as_str()) else {
        return Ok(solana_indexer::ExecutorControlFlow::Skip);
    };

    let (ix, mut ix_data) = Instruction::new(&data);

    match ix {
        Instruction::MintBadge => {
            let params = MintBadge::deserialize(&mut ix_data)?;
            let publisher = instruction
                .account_keys
                .iter()
                .find(|key| key.signer && !key.writable)
                .map(|pa| pa.pubkey.clone())
                .unwrap();

            graph::entities::prelude::Badge::insert(graph::entities::badge::ActiveModel {
                id: Set(params.badge_id),
                class_id: Set(params.class_id),
                publisher: Set(publisher),
            })
            .exec(db)
            .await?;
        }
        Instruction::Register => {
            let params = Register::deserialize(&mut ix_data)?;
            let register = instruction
                .account_keys
                .iter()
                .find(|key| key.signer && key.writable)
                .map(|pa| pa.pubkey.clone())
                .unwrap();

            graph::entities::prelude::Class::insert(graph::entities::class::ActiveModel {
                id: Set(params.class_id),
                controller_id: Set(params.profile_id),
                register: Set(register),
            })
            .exec(db)
            .await?;
        }
        Instruction::Unknow => {
            return Ok(solana_indexer::ExecutorControlFlow::Skip);
        }
    }

    Ok(().into())
}
