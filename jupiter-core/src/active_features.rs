// eval "$(solana feature status --display-all --output json | jq -r '.features | map(select(.status == "active"))[] | [ "printf", "\"%s\", // %s\\n", .id, .description ] | @sh')"
pub const MAINNET_ACTIVE_FEATURES: &[&str] = &[
    "E3PHP7w8kB7np3CTQ1qQ2tW3KCtjRSXBQgW9vM2mWv2Y", // secp256k1 program
    "E5JiFDQCwyC6QfT9REFyMpfK2mHcmv1GUDySU1Ue7TYv", // spl-token multisig fix
    "4kpdyrcj5jS47CZb2oJGfVxjYbsMm2Kx97gFyZrxxwXz", // no overflow rent distribution
    "GaBtBJvmS4Arjj5W1NmFcyvPjsHN38UGYDq2MDwbs9Qu", // deprecate unused rewards sysvar
    "4RWNif6C2WCNiKVW7otP4G7dkmkHGyKQWRpuZ1pxKU5m", // pico inflation
    "GE7fRxmW46K6EmCD9AMZSbnaJ2e3LfqCZzdHi9hmYAgi", // filter stake_delegation_accounts #14062
    "7XRJcS5Ud5vxGB54JbK9N2vBZVwnwdBNeJW1ibRgD9gx", // full inflation enabled by Certus One
    "BzBBveUDymEYoYzcMWNQCx3cd4jQs7puaVFHLtsbB6fm", // community vote allowing Certus One to enable full inflation
    "BL99GYhdjjcv6ys22C9wPgn2aTVERDbPHHo4NbS3hgp7", // spl-token self-transfer fix
    "GvDsGDkH5gyzwpDhxNixx8vtx1kwYHH13RiNAPw27zXb", // warp timestamp again, adjust bounding to 25% fast 80% slow #15204
    "3ccR6QpxGYsAbWyfevEtBNGfWV4xBffxRj2tD6A9i39F", // check initialized Vote data
    "D4jsDcXaqdW8tDAWn8H4R25Cdns2YwLneujSL1zvjW6R", // require custodian to authorize withdrawer change for locked stake
    "BcWknVcgvonN8sL4HE4XFuEVgfcee5MwxWPAgP6ZV89X", // vote/state program checked instructions #18345
    "BrTR9hzw4WBGFP65AJMbpAo64DcA3U6jdPSga9fMV5cS", // perform all checks for transfers of 0 lamports
    "FToKNBYyiF4ky9s8WsmLBXHCht17Ek7RXaLZGHzzQhJ1", // spl-token set_authority fix
    "3E3jV7v9VcdJL8iYZUMax9DiDno8j7EWUVbhm9RtShj2", // demote program write locks to readonly, except when upgradeable loader present #19593 #20265
    "C5fh68nJ7uyKAuYZg2x9sEQ5YrVf3dkW6oojNBSc3Jvo", // send votes to the tpu vote port
    "EBeznQDjcPG8491sFsKZYBi5S5jTVXMpAKNDJMQPS2kq", // reduce required payer balance for program deploys
    "EVW9B5xD9FFK7vw1SBARwMA4s5eRo5eKJdKpsBikzKBz", // prohibit extra transaction signatures
    "SAdVFw3RZvzbo6DvySbSdBnHN4gkzSTH9dSxesyKKPj", // Enable advancing credits observed for activation epoch #19309
    "meRgp4ArRPhD3KtCY9c5yAf2med7mBLsjKTPeVUHqBL", // allow merging active stakes with unmatched credits_observed #18985
    "6RvdSWHh8oh72Dp7wMTS2DBkf3fRPtChfNrAo3cZZoXJ", // secp256k1_recover syscall
    "BKCPBQQBZqggVnFso5nQ8rQ4RwwogYwjuUt9biBjxwNF", // collect rent from accounts owned by sysvars
    "265hPS8k8xJ37ot82KEgjRunsUp5w4n4Q4VwwiN9i9ps", // optimize epoch boundary updates
    "8kEuAshXLsgkUEdcFVLqrjCGGHVWFW99ZZpxvAzzMtBp", // dedupe config program signers
    "DhsYfRjxfnh2g7HKJYSzT79r74Afa1wbHkAgHndrA1oy", // upgrade libsecp256k1 to v0.5.0
    "HFpdDDNQjvcXnXKec697HDDsyk6tFoWS2o8fkxuhQZpL", // remove delegations from stakes cache when inactive
    "4d5AKtxoh93Dwm1vHXUU3iRATuMndx1c431KgT2td52r", // Add compute_budget_program
    "7txXZZD6Um59YoLMF7XUNimbMjsqsWhc7g2EniiTrmp1", // fail vote withdraw instructions which leave the account non-rent-exempt
    "EMX9Q7TVFAmQ9V1CggAkhMzhXSg8ECp7fHrWQX2G1chf", // evict invalid stakes cache entries on epoch boundaries
    "Ftok2jhqAqxUWEiCVRrfRs9DPppWP8cgTB7NQNKL88mS", // spl-token v3.3.0 release
    "HTTgmruMYRZEntyL3EdCDdnS6e4D5wRq1FA7kQsb66qq", // remove support for the native loader
    "6ppMXNYLhVd7GcsZ5uV11wQEW7spppiMVfqQv5SXhDpX", // enable builtin ed25519 signature verify program
    "6uaHcKPGUy4J7emLBgUTeufhJdiwhngW6a1R9B7c2ob9", // enable sol_log_data syscall
    "DwScAzPUjuv65TMbDnFY7AgwmotzWy3xpEJMXM3hZFaB", // enable sol_{set,get}_return_data syscall
    "FaTa4SpiaSNH44PGC4z8bnGVTkSRYaWvrBs3KTu8XQQq", // SPL Associated Token Account Program release version 1.0.4, tied to token 3.3.0 #22648
    "E8MkiWZNNPGU6n55jkGzyj8ghUmjCHRmDFdYYFYHxWhQ", // leave nonce as is on success
    "BkFDxiJQWZXGTZaJQxH7wVEHkAmwCgSEVkrvswFfRJPD", // require all new transaction accounts with data to be rent-exempt
    "75m6ysz33AfLA5DDEzWM1obBrnPQRSsdVQ2nRmc8Vuu1", // support account data reallocation
    "CFK1hRCNy8JJuAAY8Pb2GjLFNdCThS2qwZNe3izzBMgn", // add add_get_processed_sibling_instruction_syscall
    "5ekBxc8itEnPv4NzGJtr8BVVQLNMQuLMNQQj7pHoLNZ9", // transaction wide compute cap
    "CCu4boMmfLuqcmfTLPHQiUo22ZdUsXjgzPAURYaWt1Bw", // Requestable heap frame size
    "3BX6SBeEBibHaVQXywdkcgyUk6evfYZkHdztXiDtEpFS", // warp timestamp again, adjust bounding to 150% slow #25666
    "BiCU7M5w8ZCMykVSyhZ7Q3m2SWoR2qrEQ86ERcDX77ME", // nonce must be writable
    "9kdtFSrXHQg3hKkbXkQ6trJ3Ja1xpJ22CTFSNAciEwmL", // fail instructions which have native_loader as program_id directly
    "Ds87KVeqhbv7Jw8W6avsS1mqz3Mw5J3pRTpPoDQ2QdiJ", // add shred-type to shred seed #25556
    "36PRUK2Dz6HWYdG9SpjeAsF5F3KxnFCakA2BZMbtMhSb", // use correct check for nonoverlapping regions in memcpy syscall
    "3u3Er5Vc2jVcwz4xr2GJeSAXT3fAj6ADHZ4BJMZiScFd", // durable nonces must be advanceable
    "4EJQtF2pkRyawwcTVfQutzq4Sa5hRhibF6QAK1QXhtEX", // enable durable nonce #25744
    "Gea3ZkK2N4pHuVZVxWcnAtS6UEDdyumdYt4pFcKjA3ar", // separate durable nonce and blockhash domains #25744
    "HxrEu1gXuH7iD3Puua1ohd5n4iUKJyFNtNxk9DVJkvgr", // nonce must be authorized
    "2h63t332mGCCsWK2nqqqHhN4U9ayyqhLVFvczznHDoTZ", // update syscall base costs
    "AVZS3ZsN4gi6Rkx2QUibYuSJG3S6QHib7xCYhG6vGJxU", // vote account withdraw authority may change the authorized voter #22521
    "3XgNukcZWf9o3HdA3fpJbm94XFc4qpvTXc8h1wxYwiPi", // disable ldabs* and ldind* SBF instructions
    "4yuaYAj2jGMGTh1sSmi4G2eFscsDq8qjugJXZoBN6YEa", // disable reporting of unresolved SBF symbols at runtime
    "7GUcYgq4tVtaqNCKT3dho9r4665Qp5TxCZ27Qgjx3829", // Executables incur CPI data costs
    "CBkDroRDqm8HwHe6ak9cguPjUomrASEkfmxEaZ5CNNxz", // enforce max number of locked accounts per transaction
    "DpJREPyuMZ5nDfU6H3WTqSqUFSXAfw8u7xqmWtEwJDcP", // quick bail on panic
    "J2QdYx8crLbTVK8nur1jeLsmc3krDbfjoxoea2V1Uy5Q", // Default max tx-wide compute units calculated per instruction
    "3aJdcZqxoLpSBxgeYGjPwaYS1zzcByxUDqJkbzWAH1Zb", // move the CPI stack overflow check to the end of push
    "98std1NSHqXi9WYvFShfVepRdCoq1qvsp8fsR2XZtG8g", // add compute budget ix for setting a compute unit price
    "7g9EUwj4j7CS21Yx1wvgWLjSZeh5aPq8x9kpoPwXM8n8", // limit secp256k1 recovery id
    "nWBqjr3gpETbiaVj3CBJ3HFC5TMdnJDGt21hnvSTvVZ",  // check physical overlapping regions
    "4ApgRX3ud6p7LNMJmsuaAcZY5HWctGPr5obAsjB3A54d", // prevent calling precompiles as programs
    "FaTa17gVKoqbh38HcfiQonPsAaQViyDCCSg71AubYZw8", // SPL Associated Token Account Program version 1.1.0 release #24741
    "Ftok4njE8b7tDffYkC5bAbCaQv5sL6jispYrprzatUwN", // SPL Token Program version 3.4.0 release #24740
    "2jXx2yDmGysmBKfKYNgLj2DQyAQv6mMk2BPh4eSbyB4H", // deprecate fee calculator
    "6tRxEYKuy2L5nnv5bgn7iT28MxUbYxp5h7F3Ncf1exrT", // An instruction you can use to change a vote accounts authority when the current authority is a derived key #25860
    "HyrbKftCdJ5CrUfEti6x26Cj7rZLNe32weugk7tLcWb8", // syscalls use saturated math
    "21AWDosvp3pBamFW91KB35pNoaoZVTM7ess8nr2nt53B", // merge NonceError into SystemError
    "H3kBSaKdeiUsyHmeHqjJYNc27jesXZ6zWj3zWkowQbkV", // fix owner for instructions sysvar
    "8FdwgyHFEjhAdjWfV2vfqk7wA1g9X3fQpKH7SBpEv3kC", // require static program ids in versioned transactions
    "2R72wpcQ7qV7aTJWUumdn8u5wmmTyXbK7qzEy7YSAgyY", // include account index in rent tx error #25190
    "3KZZ6Ks1885aGBQ45fwRcPXVBCtzUvxhUTkwKMR41Tca", // enable versioned transaction message processing
    "HH3MUYReL2BvqqA3oEcAa7txju5GY6G4nxJ51zvsEjEZ", // preserve rent epoch for rent exempt accounts #26479
    "3gtZPqvPpsbXZVCx6hceMfWxtsmrjMzmg8C7PLKSxS2d", // filter vote slots older than the slot hashes history
    "812kqX67odAp5NFwM8D2N24cku7WTm9CHUTFUXaDkWPn", // prevent crediting rent paying accounts #26606
    "GTUMCZ8LTNxVfxdrw7ZsDFTxXb7TutYkzJnFwinpE6dg", // disable the deprecated BPF loader
    "ALBk3EWdeAg2WAGf6GPDUf1nynyNqCdEVmgouG7rpuCj", // fail vote account withdraw to 0 unless account earned 0 credits in last completed epoch
    "Vo5siZ442SaZBKPXNocthiXysNviW4UYPwRFggmbgAp", // fixes Bank::transaction_count to include all committed transactions, not just successful ones
    "3uRVPBpyEJRo1emLCrq38eLRFGcu6uKSpUXqGvU8T7SZ", // check syscall outputs do_not overlap #28600
    "437r62HoAdUb63amq3D7ENnBLDhHT2xY8eFkLJYVKK4x", // enable the deactivate delinquent stake instruction #23932
    "4Di3y24QFLt5QEUPZtbnjyfQKfm6ZMTfa6Dw1psfoMKU", // drop redundant turbine path
    "St8k9dVXP97xT6faW24YmRSYConLbhsMJA4TJTBLmMT", // add GetMinimumDelegation instruction to stake program
    "sTKz343FM8mqtyGvYWvbLpTThw3ixRM4Xk8QvZ985mw", // Allow zero-lamport undelegated amount for initialized stakes #24670
    "BUS12ciZ5gCoFafUHWW8qaFMMtwFQGVxjsDheWLdqBE2", // Auto rewind stake's credits_observed if (accidental) vote recreation is detected #22546
    "54KAoNiUERNoWWUhTWWwXgym94gzoXFVnHyQwPA18V9A", // fail libsecp256k1_verify if count appears wrong
    "G74BkWBzmsByZ1kxHy44H3wjwp5hp7JbrGRuDpco22tY", // fix root in vote state updates #27361
    "74CoWuBmt3rUVUrCb2JiSTvh6nXyBWUsK4SaMj3CtE3T", // cpi ignore serialized_len_ptr #29592
    "FQnc7U4koHqWgRvFaBJjZnV8VPg6L6wWK33yJeDp4yvV", // stake split instruction uses rent sysvar
    "CpkdQmspsaZZ8FVAouQTtTWZkc8eeQ7V3uj7dWz543rZ", // on bank load account, do not try to fix up rent_epoch #28541
    "DTVTkmw3JSofd8CJVJte8PXEbxNQ2yZijvVr3pe2APPj", // on accounts hash calculation, do not try to rehash accounts #28934
    "6iyggb5MTcsvdcugX7bEKbHV8c6jdLbpHwkncrgLMhfo", // stop adding hashes for skipped slots to recent blockhashes
    "9k5ijzTbYPtjzu8wj2ErH9v45xecHzQ1x4PMYMMxFgdM", // enforce max number of accounts per bpf program instruction #26628
    "28s7i3htzhahXQKqmS2ExzbEoUypg9krwvtK2M9UWXh9", // update rewards from cached accounts
    "8sKQrMQoUHtQSUP83SPG4ta2JDjSAiWs7t5aJ9uEd6To", // use default units per instruction in fee calculation #26785
    "4UDcAfQ6EcA6bdcadkeHpkarkhZGJ7Bpq7wTAiRMjkoi", // disable builtin loader ownership chains #29956
    "GmC19j9qLn2RFk5NduX6QXaDhVpGncVVBzyM8e9WMz2F", // check size when translating slices
    "JAN1trEUEtZjgXYzNBYHU9DYd7GnThhXfFP7SzPXkPsG", // disable fees sysvar
    "79HWsX9rpnnJBPcdNURVqygpMAfxdrAirzAGAVmf92im", // disable new deployments of deprecated sol_alloc_free_ syscall
    "noRuG2kzACwgaY7TVmLRnUNPLKNVQE1fb7X55YWBehp", // validator commission updates are only allowed in the first half of an epoch #29362
    "Bj2jmUsM2iRhfdLLDSTkhM5UQRQvQHm57HSmPibPtEyu", // Return InsufficientDelegation instead of InsufficientFunds or InsufficientStake where applicable #31206
    "86HpNqzutEZwLcPxS6EHDcMNYWk6ikhteg9un7Y2PBKE", // Compact vote state updates to lower block size
    "CveezY6FDLVBToHDcvJRmtMouqzsmj4UXYh5ths5G5Uv", // Calculate vote credits for VoteStateUpdate per vote dequeue to match credit awards for Vote instruction
    "Ff8b1fBeB86q8cjq47ZhsQLgv5EkHu3G1C99zjUfAzrq", // enable direct vote state update
    "Hr1nUA9b7NJ6eChS26o7Vi8gYYDDwWD3YeBfzJkTbU86", // Enable transaction to request heap frame using compute budget instruction #30076
    "7Vced912WrRnfjaiKRiNBcbuFw7RrnLv3E3z95Y4GTNc", // enable early verification of account modifications #25899
    "Ffswd3egL3tccB6Rv3XY6oqfdzn913vUcjCSnpvCKpfx", // better error codes for tx lamport check #33353
    "GQALDaC48fEhZGWRj9iL5Q889emJKcj3aCvHF7VCbbF4", // limit max instruction trace length #27939
    "9gxu85LYRAcZL38We8MYJ4A9AwgBBPtVBAqebMcT1241", // cap accounts data allocations per transaction #27375
    "SVn36yVApPLYsa8koK3qUcy14zXDnqkNYWyUh1f4oK1", // ignore slot when calculating an account hash #28420
    "B9cdB55u4jQsDNsdTK525yE9dmSc5Ga7YBaBrDFvEhM9", // disable setting is_executable and_rent_epoch in CPI #26987
    "5GpmAKxaGsWWbPp4bNXFLJxZVvG92ctxf7jQnzTQjF3n", // enable epoch accounts hash calculation #27539
    "GmuBvtFb2aHfSfMXpuFeWZGHyDeCLPS79s48fmCWCfM5", // delay visibility of program upgrades #30085
    "J4HFT8usBxpcF63y46t1upYobJgChmKyZPm5uTBRg25Z", // enable program redeployment cooldown #29135
    "8Zs9W7D9MpSEtUWSQdGniZk2cNmV22y6FLJwCx53asme", // enable bpf upgradeable loader ExtendProgram instruction #25234
    "DdLwVYuvDz26JohmgSbA7mjpJFgX5zP2dkp8qsF2C33V", // cap transaction accounts data size up to a limit #27839
    "G6vbf1UBok8MWb8m25ex86aoQHeKTzDKzuZADHkShqm6", // add compute budget instruction for setting account data size per transaction #30366
    "3uFHb9oKdGfgZGJK9EHaAXN4USvnQtAFC13Fh5gGFS5B", // Update desired hashes per tick on epoch boundary
    "EfhYd3SafzGT472tYQDUc4dPd2xdEfKs5fwkowUgVt4W", // remove support for RequestUnitsDeprecated instruction #27500
    "Fab5oP3DmsLYCiQZXdjyqT3ukFFPrsmqhXU4WU1AWVVF", // prevent recipients of rent rewards from ending in rent-paying state #30151
    "5Pecy6ie6XGm22pc9d4P9W5c31BugcFBuy6hsP2zkETv", // checked arithmetic in fee validation #31273
    "CE2et8pqgyQMP2mQRg3CgvX8nJBKUArMu3wfiQiQKY1y", // round up heap size when calculating heap cost #30679
    "EYVpEP7uzH1CoXzbD6PubGhYmnxRXPeq3PPsm1ba3gpo", // stop the search in get_processed_sibling_instruction when the parent instruction is reached #27289
    "A8xyMHZovGXFkorFqEmVH2PKGLiBip5JD7jt4zsUWo4H", // Remove congestion multiplier from transaction fee calculation #29881
    "2HmTkCj9tXuPE4ueHzdD7jPeMf9JGCoZh5AsyoATiWEe", // stop incorrectly throwing IncorrectProgramId in bpf_loader #30747
    "16FMCmgLzCNNz6eTwGanbyN2ZxvTBSLuQ6DZhgeMshg",  // Stop truncating strings in syscalls #31029
    "8pgXCMNXC8qyEFypuwpXyRxLXZdpM4Qo72gJ6k87A6wL", // Native program should consume compute units #30620
    "5ZCcFAzJ1zsFKe1KSZa9K92jhx7gkcKj97ci2DBo1vwj", // Simplify checks performed for writable upgradeable program accounts #30559
    "25vqsfjk7Nv1prsQJmA4Xu1bN61s8LXCBGUPp8Rfy1UF", // only hash accounts in incremental snapshot during incremental snapshot creation #26799
    "GwtDQBghCTBgmX2cpEGNPxTEBUTQRaDMGTr5qychdGMj", // reduce stake warmup cooldown from 25% to 9%
    "BTWmtJC8U5ZLMbBUUA1k6As62sYjPEjAiNAT55xYGdJU", // revise turbine epoch stakes
    "5wAGiy15X1Jb2hkHnPDCM8oB9V42VNA9ftNVFK84dEgv", // set rent epoch to Epoch::MAX for rent-exempt accounts #28683
    "D31EFnLgdiysi84Woo3of4JMu7VmasUS3Z7j9HYXCeLY", // enable turbine fanout experiments #29393
    "FKAcEvNgSY79RpqsPNUV5gDyumopH4cEHqUxyfm8b8Ap", // relax authority signer check for lookup table creation #27205
    "5x3825XS7M2A3Ekbn5VGGkvFoAg5qrRWkTrY4bARP1GL", // enable bpf upgradeable loader SetAuthorityChecked instruction #28424
    "7axKe5BTYBDD87ftzWbk5DfzWMGyRvqmWTduuo22Yaqy", // replace Lockout with LandedVote (including vote latency) in vote state #31264
    "EWme9uFqfy1ikK1jhJs8fM5hxWnK336QJpbscNtizkTU", // Update desired hashes per tick to 2.8M
    "D2aip4BBr8NPWtU9vLrwrBvbuaQ8w1zV38zFLxx4pfBV", // Require stake split destination account to be rent exempt
    "8C8MCtsab5SsfammbzvYz65HHauuUYdbY2DZ4sznH6h5", // Update desired hashes per tick to 4.4M
    "8We4E7DPwF2WfAN8tRTtWQNhi98B99Qpuj7JoZ3Aikgg", // Update desired hashes per tick to 7.6M
    "BsKLKAn1WM4HVhPRDsjosmqSg2J8Tq5xP2s2daDS6Ni4", // Update desired hashes per tick to 9.2M
    "FKu1qYwLQSiehz644H6Si65U5ZQ2cp9GxsyFUfYcuADv", // Update desired hashes per tick to 10M
    "prpFrMtgNmzaNzkPJg9o753fVvbHKqNrNTm76foJ2wm",  // validate fee collector account #33888
    "GV49KKQdBNaiv2pgqhS2Dy3GWYJGXMTVYbYkdk91orRy", // drops legacy shreds #34328
    "7WeS1vfPRgeeoXArLh7879YcB9mgE9ktjPDtajXeWfXn", // disable bpf loader management instructions #34194
    "6YsBCejwK96GZCkJ6mkZ4b68oP63z2PLoQmWjC7ggTqZ", // consume duplicate proofs from blockstore in consensus #34372
    "dupPajaLy2SSn8ko42aZz4mHANDNrLe8Nw8VQgFecLa", // generate duplicate proofs for index and erasure conflicts #34360
    "eca6zf6JJRjQsYYPkBHF3N32MTzur4n2WL4QiiacPCL", // restrict curve25519 multiscalar multiplication vector lengths #34763
    "Cdkc8PPTeTNUPoZEfCY5AyetUrEdkZtNPMgz58nqyaHD", // switch to new ELF parser #30497
    "JDn5q3GBeqzvUa7z67BbmVHVdE3EbUAjvFep3weR3jxX", // simplify alt_bn128 syscall error codes SIMD-0129
    "7rcw5UtqgDTBBv2EcynNfYckgdAaH1MAsCjKgXMkN7Ri", // enable curve25519 syscalls
    "A16q37opZdQMCbe5qJ6xpBB9usykfv8jZaMkxvZQi4GJ", // add alt_bn128 syscalls #27961
    "EJJewYSddEEtSZHiqugnvhQHiWyZKjkFDQASd7oKSagn", // add alt_bn128 compression syscalls
    "FL9RsQA6TVUoh5xJQ9d936RHSebA1NLQqe3Zv9sXZRpr", // Enable Poseidon syscall
    "3NKRSwpySNwD3TvP5pHnRmkAQRsdkXWRr1WaQh8p4PWX", // Reject bpf callx r10 instructions
    "8199Q2gMD2kwgfopK5qqVWuDbegLgpuFUFHCcUJQDN8b", // error on bpf function hash collisions
    "HooKD5NC9QNxk25QuzCssB8ecrEzGt6eXEPBUxWp1LaR", // enable new sysvar last_restart_slot
    "6Uf8S75PVh91MYgPQSHnjRAPQq6an5BDv9vomrCwDqLe", // Deprecate unused legacy vote tx plumbing
    "mrkPjRg79B2oK2ZLgd7S3AfEJaX9B6gAF3H9aEykRUS", // generate duplicate proofs for merkle root conflicts #34270
    "FNKCMBzYUdjhHyPdsKG2LSmdzH8TCHXn3ytj8RNBS4nG", // enable gossip duplicate proof ingestion #32963
    "tvcF6b1TRz353zKuhBjinZkKzjmihXmBAHJdjNYw1sQ", // use timeliness of votes in determining credits to award
    "decoMktMcnmiq6t3u7g5BfgcQu91nKZr6RvMYf9z1Jb", // Allow commission decrease at any time in epoch #33843
    "PERzQrt5gBD1XEe2c9XdFWqwgHY3mr7cYWbm5V772V8", // replaces enable_partitioned_epoch_reward to enable partitioned rewards at epoch boundary SIMD-0118
    "7uZBkJXJ1HkuP6R3MJfZs7mLwymBcDbKdqbF51ZWLier", // Enable chained Merkle shreds #34916
    "ed9tNscbWLYBooxWA7FE2B5KHWs8A6sxfY8EzezEcoo", // Use strict verification in ed25519 precompile SIMD-0152
    "FuS3FPfJDKSNot99ECLXtp3rueq36hMNStJkPJwWodLh", // Abort when elliptic curve syscalls invoked on invalid curve id SIMD-0137
    "wLckV1a64ngtcKPRGU4S4grVTestXjmNjxBjaKZrAcn", // cost model uses number of requested write locks #34819
    "GDH5TVdbTPUpRnXaRyQqiKUa7uZAbZ28Q2N9bhbKoMLm", // loosen cpi size restrictions #26641
    "7bTK6Jis8Xpfrs8ZoUfiMDPazTcdPcTWheZFJTA5Z6X4", // Enable MoveStake and MoveLamports stake program instructions #1610
    "EQUMpNFr7Nacb1sva56xn1aLfBxppEoSBH8RRVdkcD1x", // Disable account loader special case #3513
    "zkhiy5oLowR7HY4zogXjCjeMXyruLqBwSWH21qcFtnv",  // Enable ZkElGamalProof program SIMD-0153
    "BtVN7YjDzNE6Dk7kTT7YTDgMNUZTNgiSJgsdzAeTg2jF", // Removing unwanted rounding in fee calculation #34982
    "3opE3EzAKnUftUDURkzMgwpNgimBAypW1mNDYH4x4Zg7", // Reward full priority fee to validators #34731
    "8U4skmMVnF6k2kMvrWbQuRUT3qQSiTYpSjqmhmgfthZu", // add new unwritable reserved accounts #34899
    "CLCoTADvV64PSrnR6QXty6Fwrt9Xc6EdxSJE4wLRePjq", // Enable syscall for fetching Sysvar bytes #615
    "tSynMCspg4xFiCj1v3TDb4c7crMR5tSBhLz4sF7rrNA",  // Enable tower sync vote instruction
    "4eohviozzEeivk1y9UbrnekbAFMDQyJz5JjA9Y6gyvky", // Migrate Feature Gate program to Core BPF (programify) #1003
    "2Fr57nzzkLYXW695UdDxDeR5fhnZWSttZeZYemrnpGFV", // Migrate Config program to Core BPF #1378
    "CGB2jM8pwZkeeiXQ66kBMyBR6Np61mggL7XUsmLjVcrw", // skip rewriting rent exempt accounts during rent collection #26491
    "CJzY83ggJHqPGDq8VisV3U91jDJLuEaALZooBrXtnnLU", // Disable rent fees collection #33945
    "PaymEPK2oqwT9TXAVfadjztH2H6KfLEB9Hhd5Q5frvP", // Enable fees for some additional transaction failures SIMD-0082
    "C9oAhLxDBm3ssWtJx1yBGzPY55r2rArHmN1pbQn6HogH", // Reserve minimal CUs for builtin instructions SIMD-170 #2562
    "B7H2caeia4ZFcpE3QcgMqbiWiBtWrdBRBSJ1DY6Ktxbq", // Deplete compute meter for vm errors SIMD-0182 #3993
    "9ypxGLzkMxi89eDerRKXWDXe44UY2z4hBig4mDhNq5Dp", // SIMD-0159: Move precompile verification into SVM
    "C97eKZygrkU4JxJsZdjgbUY7iQR7rKTr4NyDWo2E5pRm", // Migrate Address Lookup Table program to Core BPF #1651
    "2ry7ygxiYURULZCrypHhveanvP5tzZ4toRwVp89oCNSj", // apply cost tracker to blocks during replay #29595
    "FKe75t4LXxGaQnVHdUKM6DSFifVVraGZ8LyNo7oPwy1Z", // Enable syscall: sol_get_epoch_stake #884
    "ffecLRhhakKSGhMuc6Fz2Lnfq4uT9q3iu9ZsNaPLxPc",  // vote only full fec sets
    "5oMCU3JPaFLr8Zr4ct7yFA7jdk6Mw1RmB8K4u9ZbS42z", // Raise block limit to 50M SIMD-0207
    "srremy31J5Y25FrAApwVb9kZcfXbusYMMsvTK9aWv5q",  // Enable secp256r1 precompile SIMD-0075
    "RENtePQcDLrAbxAsP3k8dwVcnNYQ466hi2uKvALjnXx", // SIMD-0267: Sets rent_epoch to a constant in the VM
    "LTHasHQX6661DaDD4S6A2TFi6QBuiwXKv66fB1obfHq", // enables lattice-based accounts hash SIMD-0215
    "LTdLt9Ycbyoipz5fLysCi1NnDnASsZfmJLJXts5ZxZz", // removes accounts delta hash SIMD-0223
    "2B2SBNbUcr438LtGXNcJNBP2GBSxjx81F945SdSkUSfC", // SIMD-0175: Disable partitioned rent collection
    "JE86WkYvTrzW8HgNmrHY7dFYpCmSptUpKupbo2AdQ9cG", // Enables deployment and execution of SBPFv1 programs SIMD-0161
    "FXs1zh47QbNnhXcnB6YiAQoJ4sGB91tKF3UFHLcKT7PM", // Remove checks of accounts is_executable flag SIMD-0162
];
