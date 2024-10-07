use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum ErrorCode {
    #[msg("The Guardian Already Existed")]
    ExistedGuardian, // 0x1770 (6000)
    #[msg("The Number Of Guardian Exceeds Maximum")]
    MaximumGuardiansExceeds, // 0x1770 (6000)
    #[msg("Fee Rate Is Too Large")]
    FeeRateIsTooLarge, // 0x1770 (6000)
    #[msg("Unable to call transfer hook without extra accounts")]
    NoExtraAccountsForTransferHook,
    #[msg("Timestamp should be convertible from i64 to u64")]
    InvalidTimestampConversion, // 0x1785 (6021)
    #[msg("Invalid Deposit Amount")]
    InvalidDepositAmount, // 0x1785 (6021)
    #[msg("Account is already initialized")]
    AccountIsInitialized, // 0x1785 (6021)
    #[msg("Invalid remaining accounts")]
    RemainingAccountsInvalidSlice, // 0x17a0 (6048)
    #[msg("Insufficient remaining accounts")]
    RemainingAccountsInsufficient, // 0x17a1 (6049)
    #[msg("Same accounts type is provided more than once")]
    RemainingAccountsDuplicatedAccountsType, // 0x17a5 (6053)
}
