# Jupiter Amm Implementation

This is a guide to help create the implementation necessary

Guideline

- Limit the Amm implementation to deserializing the state and calling your sdk to quote, the detailed implementation should remain in your AMM sdk
- Move everything that needs to be done only when the state changes to update, rather than quote

## Example Implementation

[SPL Token Swap](./jupiter-core/src/amms/spl_token_swap_amm.rs)

Use `cargo test` to run the integration tests to verify that the simulation yields the same swap outcome as the Amm implementation

## Test your own implementation

Make sure your AMM is implemented and added to `amm_factory`

Take a snapshot of your AMM state, this is to allow reproducible test and being able to capture edge cases

`cargo run -r -- --rpc-url <RPC-URL> snapshot-amm --amm-id <AMM-ID>`

Add your amm to `test_exact_in_amms` and run the tests `cargo test`...

## Jupiter AMM Interface

Most importantly, the [Jupiter AMM Interface](https://docs.rs/crate/jupiter-amm-interface) is the main crate this integration depends on and must be used to be compatible with Jupiter. Do check it out.
