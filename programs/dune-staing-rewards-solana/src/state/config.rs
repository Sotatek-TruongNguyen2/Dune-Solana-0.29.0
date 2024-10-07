use crate::errors::ErrorCode;
use anchor_lang::prelude::*;

#[account]
pub struct DunePoolConfig {
    pub bump: u8,               // 1
    pub guardians: Vec<Pubkey>, // we want to support up to 20 guardians - 4 + 32 * 20
    pub super_guardian: Pubkey, // 32
}

impl DunePoolConfig {
    pub const LEN: usize = 8 + 1 + (4 + 20 * 32) + 32;
    pub const MAXIMUM_GUARDIANS: u32 = 20;

    pub fn set_guardian(&mut self, guardian: Pubkey) -> Result<()> {
        let existed = self.guardians.iter().find(|g| (*g).eq(&guardian));

        if existed.is_some() {
            return Err(error!(ErrorCode::ExistedGuardian));
        }

        if self.guardians.len() as u32 >= Self::MAXIMUM_GUARDIANS {
            return Err(error!(ErrorCode::MaximumGuardiansExceeds));
        }

        // Push the new guardian after satisfy all conditions
        self.guardians.push(guardian);

        Ok(())
    }

    pub fn initialize(&mut self, bump: u8, guardians: Vec<Pubkey>) -> Result<()> {
        self.bump = bump;
        self.guardians = guardians;

        Ok(())
    }
}
