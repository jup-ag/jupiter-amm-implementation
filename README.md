# Jupiter Rust Amm Implementation

This is a guide to help create the implementation for easier integration with Jupiter

## Example implementation

[Spl token swap](./jupiter-core/src/amms/spl_token_swap_amm.rs)

You may run the test inside to run it.

## Main interface

[file](./jupiter-core/src/amms/amm.rs)
```rust
pub trait Amm {
    // Amm name
    fn label(&self) -> String;
    // Amm identifier, should be your pool address
    fn key(&self) -> Pubkey;
    // Token mints that the amm supports for swapping
    fn get_reserve_mints(&self) -> Vec<Pubkey>;
    // Accounts related for quoting and creating ix
    fn get_accounts_to_update(&self) -> Vec<Pubkey>;
    // Picks data necessary to update it's internal state
    fn update(&mut self, accounts_map: &HashMap<Pubkey, Vec<u8>>) -> Result<()>;
    // Returns quote for the given quote params
    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote>;
    
    // Just state how do we make a swap instruction, you don't have to implement this
    fn get_swap_leg_and_account_metas(
        &self,
        swap_params: &SwapParams,
    ) -> Result<SwapLegAndAccountMetas>;

}
```

Quote interface
```rust
pub struct Quote {
    pub not_enough_liquidity: bool,
    pub min_in_amount: Option<u64>,
    pub min_out_amount: Option<u64>,
    pub in_amount: u64,
    pub out_amount: u64,
    pub fee_amount: u64,
    pub fee_mint: Pubkey,
    pub fee_pct: Decimal,
    pub price_impact_pct: Decimal,
}
```