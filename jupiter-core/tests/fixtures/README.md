# Solana test fixtures

```
solana program dump [program_id] [filename.so]
```

## Simulation test example based on Invariant AMM

#### Make sure you're on the right network, `mainnet-beta`.
```
solana config set --url https://your-own-rpc.com
```

#### Create a snapshot for our `INVARIANT_USDC_USDT` pool in `/jupiter-core`
```
cargo run snapshot-amm --amm-id <amm_id>
cargo run snapshot-amm --amm-id BRt1iVYDNoohkL1upEb8UfHE8yji6gEDAmuN9Y4yekyc
```
You should see a new `BRt1iVYDNoohkL1upEb8UfHE8yji6gEDAmuN9Y4yekyc` folder being created in `/tests/fixtures/accounts`.

_* If you get this error "No in amount for mint", add an entry to `TOKEN_MINT_TO_IN_AMOUNT` in `test_harness.rs`, then run the snapshot again._

#### Dump the program into `/jupiter-core/tests/fixtures`
```
solana program dump <program_id> <filename>.so
solana program dump HyaB3W9q6XdA5xwpU4XnSZV94htfmbmqJXZcEbRaJutt invariant.so
```

#### Update `test_harness.rs` to include the following lines:
```
pub async fn test_quote_matches_simulated_swap() {
    use crate::amms::{
    	...
        invariant_amm::{self, InvariantAmm},
   };
   ...
}
```

```
 const INVARIANT_USDC_USDT: Pubkey = pubkey!("BRt1iVYDNoohkL1upEb8UfHE8yji6gEDAmuN9Y4yekyc");
 ```

 ```
 let program_id_and_amms_under_test: Vec<(Pubkey, Box<dyn Amm>, u64)> = vec![
 	...
	(
	    invariant_amm::PROGRAM_ID,
	    load_amm!(test_harness, INVARIANT_USDC_USDT, InvariantAmm),
	    0,
	),
];
 ```
 _* The 3rd argument "0" is just a deviation threshold +- 3. It will test the simulation amount with the computed quote amount. Ideally it is 0._

#### Then run `cargo test`

Hopefully everything runs well!

