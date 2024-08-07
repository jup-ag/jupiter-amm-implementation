export type Goatswap = {
  "version": "0.2.0",
  "name": "goatswap",
  "instructions": [
    {
      "name": "createAmmConfig",
      "docs": [
        "# Arguments",
        "",
        "* `ctx`- The accounts needed by instruction.",
        "* `index` - The index of amm config, there may be multiple config.",
        "* `trade_fee_rate` - Trade fee rate, can be changed.",
        "* `protocol_fee_rate` - The rate of protocol fee within tarde fee.",
        "* `fund_fee_rate` - The rate of fund fee within tarde fee.",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Address to be set as protocol owner."
          ]
        },
        {
          "name": "ammConfig",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Initialize config state account to store protocol owner address and fee rates."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "index",
          "type": "u16"
        },
        {
          "name": "tradeFeeRate",
          "type": "u64"
        },
        {
          "name": "protocolFeeRate",
          "type": "u64"
        },
        {
          "name": "fundFeeRate",
          "type": "u64"
        },
        {
          "name": "createPoolFee",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateAmmConfig",
      "docs": [
        "Updates the owner of the amm config",
        "Must be called by the current owner or admin",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `trade_fee_rate`- The new trade fee rate of amm config, be set when `param` is 0",
        "* `protocol_fee_rate`- The new protocol fee rate of amm config, be set when `param` is 1",
        "* `fund_fee_rate`- The new fund fee rate of amm config, be set when `param` is 2",
        "* `new_procotol_owner`- The config's new owner, be set when `param` is 3",
        "* `new_fund_owner`- The config's new fund owner, be set when `param` is 4",
        "* `create_pool_fee`- The config's new owner, be set when `param` is 5",
        "* `disable_create_pool`- The config's new fund owner, be set when `param` is 6",
        "* `param`- The vaule can be 0 | 1 | 2 | 3 | 4 | 5 | 6, otherwise will report a error",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The amm config owner or admin"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Amm config account to be changed"
          ]
        }
      ],
      "args": [
        {
          "name": "param",
          "type": "u8"
        },
        {
          "name": "value",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updatePoolStatus",
      "docs": [
        "Update pool status for given vaule",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `status` - The vaule of status",
        ""
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "status",
          "type": "u8"
        }
      ]
    },
    {
      "name": "updatePoolTaxStatus",
      "docs": [
        "Update pool tax status for given vaule",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `tax_disabled` - The vaule of tax status of pool",
        ""
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "taxDisabled",
          "type": "bool"
        }
      ]
    },
    {
      "name": "updateTax",
      "docs": [
        "Update tax",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `tax_use_token_0` - tax use token0 or token1",
        "* `tax_fee_in_rate` - new tax fee rate of pool when swap in",
        "* `tax_fee_out_rate` - new tax fee rate of pool when swap out",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "taxUseToken0",
          "type": "bool"
        },
        {
          "name": "inTaxRate",
          "type": "u64"
        },
        {
          "name": "outTaxRate",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateLpFee",
      "docs": [
        "Update tax",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `lp_fee_rate` - lp fee rate value",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lpFeeRate",
          "type": "u64"
        }
      ]
    },
    {
      "name": "transferTaxAuthority",
      "docs": [
        "Update tax authority",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `new_authority` - new tax transfer authority",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool or admin"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "newAuthority",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "transferPoolOwner",
      "docs": [
        "Transfer pool owner",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `new_owner` - The new owner of pool",
        ""
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "newOwner",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "collectProtocolFee",
      "docs": [
        "Collect the protocol fee accrued to the pool",
        "",
        "# Arguments",
        "",
        "* `ctx` - The context of accounts",
        "* `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1",
        "* `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Only admin or owner can collect fee now"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool state stores accumulated protocol fee amount"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Amm config account stores owner"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "recipientToken0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_0 protocol fees"
          ]
        },
        {
          "name": "recipientToken1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_1 protocol fees"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program to perform token transfers"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program 2022 to perform token transfers"
          ]
        }
      ],
      "args": [
        {
          "name": "amount0Requested",
          "type": "u64"
        },
        {
          "name": "amount1Requested",
          "type": "u64"
        }
      ]
    },
    {
      "name": "collectFundFee",
      "docs": [
        "Collect the fund fee accrued to the pool",
        "",
        "# Arguments",
        "",
        "* `ctx` - The context of accounts",
        "* `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1",
        "* `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Only admin or fund_owner can collect fee now"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool state stores accumulated protocol fee amount"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Amm config account stores fund_owner"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "recipientToken0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_0 fund fees"
          ]
        },
        {
          "name": "recipientToken1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_1 fund fees"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program to perform token transfers"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program 2022 to perform token transfers"
          ]
        }
      ],
      "args": [
        {
          "name": "amount0Requested",
          "type": "u64"
        },
        {
          "name": "amount1Requested",
          "type": "u64"
        }
      ]
    },
    {
      "name": "collectTax",
      "docs": [
        "Collect the tax accrued to the pool",
        "",
        "# Arguments",
        "",
        "* `ctx` - The context of accounts",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Amm config account stores fund_owner"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "recipientToken0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_0 tax"
          ]
        },
        {
          "name": "recipientToken1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_1 tax"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program to perform token transfers"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program 2022 to perform token transfers"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initialize",
      "docs": [
        "Creates a pool for the given token pair and the initial price",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `init_amount_0` - the initial amount_0 to deposit",
        "* `init_amount_1` - the initial amount_1 to deposit",
        "* `open_time` - the timestamp allowed for swap",
        ""
      ],
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Address paying to create the pool. Can be anyone"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Which config the pool belongs to."
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Initialize an account to store the pool state"
          ]
        },
        {
          "name": "token0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_0 mint, the key must smaller then token_1 mint."
          ]
        },
        {
          "name": "token1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_1 mint, the key must grater then token_0 mint."
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "pool lp mint"
          ]
        },
        {
          "name": "creatorToken0",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "payer token0 account"
          ]
        },
        {
          "name": "creatorToken1",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator token1 account"
          ]
        },
        {
          "name": "creatorLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator lp token account"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "createPoolFee",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create mint account and mint tokens"
          ]
        },
        {
          "name": "token0Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "token1Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create an ATA for receiving position NFT"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "To create a new program account"
          ]
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Sysvar for program account"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "initAmount0",
          "type": "u64"
        },
        {
          "name": "initAmount1",
          "type": "u64"
        },
        {
          "name": "openTime",
          "type": "u64"
        },
        {
          "name": "taxUseToken0",
          "type": "bool"
        },
        {
          "name": "inTaxRate",
          "type": "u64"
        },
        {
          "name": "outTaxRate",
          "type": "u64"
        },
        {
          "name": "lpFeeRate",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "initializeWhitelisted",
      "docs": [
        "(For whitelisted program only) Creates a pool for the given token pair and the initial price",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `init_amount_0` - the initial amount_0 to deposit",
        "* `init_amount_1` - the initial amount_1 to deposit",
        "* `open_time` - the timestamp allowed for swap",
        ""
      ],
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Address paying to create the pool. Can be anyone"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Which config the pool belongs to."
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Initialize an account to store the pool state"
          ]
        },
        {
          "name": "token0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_0 mint, the key must smaller then token_1 mint."
          ]
        },
        {
          "name": "token1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_1 mint, the key must grater then token_0 mint."
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "pool lp mint"
          ]
        },
        {
          "name": "creatorToken0",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "payer token0 account"
          ]
        },
        {
          "name": "creatorToken1",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator token1 account"
          ]
        },
        {
          "name": "creatorLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator lp token account"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create mint account and mint tokens"
          ]
        },
        {
          "name": "token0Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "token1Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create an ATA for receiving position NFT"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "To create a new program account"
          ]
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Sysvar for program account"
          ]
        },
        {
          "name": "instructionSysvarAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "initAmount0",
          "type": "u64"
        },
        {
          "name": "initAmount1",
          "type": "u64"
        },
        {
          "name": "openTime",
          "type": "u64"
        },
        {
          "name": "taxUseToken0",
          "type": "bool"
        },
        {
          "name": "inTaxRate",
          "type": "u64"
        },
        {
          "name": "outTaxRate",
          "type": "u64"
        },
        {
          "name": "lpFeeRate",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "deposit",
      "docs": [
        "Creates a pool for the given token pair and the initial price",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `lp_token_amount` - Pool token amount to transfer. token_a and token_b amount are set by the current exchange rate and size of the pool",
        "* `maximum_token_0_amount` -  Maximum token 0 amount to deposit, prevents excessive slippage",
        "* `maximum_token_1_amount` - Maximum token 1 amount to deposit, prevents excessive slippage",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Pays to mint the position"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ownerLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Owner lp tokan account"
          ]
        },
        {
          "name": "token0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The payer's token account for token_0"
          ]
        },
        {
          "name": "token1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The payer's token account for token_1"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "token Program"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token program 2022"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Lp token mint"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lpTokenAmount",
          "type": "u64"
        },
        {
          "name": "maximumToken0Amount",
          "type": "u64"
        },
        {
          "name": "maximumToken1Amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdraw",
      "docs": [
        "Withdraw lp for token0 ande token1",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `lp_token_amount` - Amount of pool tokens to burn. User receives an output of token a and b based on the percentage of the pool tokens that are returned.",
        "* `minimum_token_0_amount` -  Minimum amount of token 0 to receive, prevents excessive slippage",
        "* `minimum_token_1_amount` -  Minimum amount of token 1 to receive, prevents excessive slippage",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Pays to mint the position"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool state account"
          ]
        },
        {
          "name": "ownerLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Owner lp token account"
          ]
        },
        {
          "name": "token0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The owner's token account for receive token_0"
          ]
        },
        {
          "name": "token1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The owner's token account for receive token_1"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "token Program"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token program 2022"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool lp token mint"
          ]
        },
        {
          "name": "memoProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "memo program"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lpTokenAmount",
          "type": "u64"
        },
        {
          "name": "minimumToken0Amount",
          "type": "u64"
        },
        {
          "name": "minimumToken1Amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "swapBaseInput",
      "docs": [
        "Swap the tokens in the pool base input amount",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `amount_in` -  input amount to transfer, output to DESTINATION is based on the exchange rate",
        "* `minimum_amount_out` -  Minimum amount of output token, prevents excessive slippage",
        ""
      ],
      "accounts": [
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The user performing the swap"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The factory state to read protocol fees"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The program account of the pool in which the swap will be performed"
          ]
        },
        {
          "name": "inputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for input token"
          ]
        },
        {
          "name": "outputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for output token"
          ]
        },
        {
          "name": "inputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for input token"
          ]
        },
        {
          "name": "outputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for output token"
          ]
        },
        {
          "name": "inputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for input token transfers"
          ]
        },
        {
          "name": "outputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for output token transfers"
          ]
        },
        {
          "name": "inputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of input token"
          ]
        },
        {
          "name": "outputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of output token"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minimumAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "swapBaseOutput",
      "docs": [
        "Swap the tokens in the pool base output amount",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `max_amount_in` -  input amount prevents excessive slippage",
        "* `amount_out` -  amount of output token",
        ""
      ],
      "accounts": [
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The user performing the swap"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The factory state to read protocol fees"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The program account of the pool in which the swap will be performed"
          ]
        },
        {
          "name": "inputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for input token"
          ]
        },
        {
          "name": "outputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for output token"
          ]
        },
        {
          "name": "inputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for input token"
          ]
        },
        {
          "name": "outputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for output token"
          ]
        },
        {
          "name": "inputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for input token transfers"
          ]
        },
        {
          "name": "outputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for output token transfers"
          ]
        },
        {
          "name": "inputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of input token"
          ]
        },
        {
          "name": "outputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of output token"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "maxAmountIn",
          "type": "u64"
        },
        {
          "name": "amountOut",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "ammConfig",
      "docs": [
        "Holds the current owner of the factory"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "docs": [
              "Bump to identify PDA"
            ],
            "type": "u8"
          },
          {
            "name": "disableCreatePool",
            "docs": [
              "Status to control if new pool can be create"
            ],
            "type": "bool"
          },
          {
            "name": "index",
            "docs": [
              "Config index"
            ],
            "type": "u16"
          },
          {
            "name": "tradeFeeRate",
            "docs": [
              "The trade fee, denominated in hundredths of a bip (10^-6)"
            ],
            "type": "u64"
          },
          {
            "name": "protocolFeeRate",
            "docs": [
              "The protocol fee"
            ],
            "type": "u64"
          },
          {
            "name": "fundFeeRate",
            "docs": [
              "The fund fee, denominated in hundredths of a bip (10^-6)"
            ],
            "type": "u64"
          },
          {
            "name": "createPoolFee",
            "docs": [
              "Fee for create a new pool"
            ],
            "type": "u64"
          },
          {
            "name": "protocolOwner",
            "docs": [
              "Address of the protocol fee owner"
            ],
            "type": "publicKey"
          },
          {
            "name": "fundOwner",
            "docs": [
              "Address of the fund fee owner"
            ],
            "type": "publicKey"
          },
          {
            "name": "padding",
            "docs": [
              "padding"
            ],
            "type": {
              "array": [
                "u64",
                16
              ]
            }
          }
        ]
      }
    },
    {
      "name": "poolState",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authBump",
            "type": "u8"
          },
          {
            "name": "status",
            "docs": [
              "Bitwise representation of the state of the pool",
              "bit0, 1: disable deposit(vaule is 1), 0: normal",
              "bit1, 1: disable withdraw(vaule is 2), 0: normal",
              "bit2, 1: disable swap(vaule is 4), 0: normal"
            ],
            "type": "u8"
          },
          {
            "name": "lpMintDecimals",
            "type": "u8"
          },
          {
            "name": "mint0Decimals",
            "docs": [
              "mint0 and mint1 decimals"
            ],
            "type": "u8"
          },
          {
            "name": "mint1Decimals",
            "type": "u8"
          },
          {
            "name": "ammConfig",
            "docs": [
              "Which config the pool belongs"
            ],
            "type": "publicKey"
          },
          {
            "name": "poolCreator",
            "docs": [
              "pool creator"
            ],
            "type": "publicKey"
          },
          {
            "name": "token0Vault",
            "docs": [
              "Token A"
            ],
            "type": "publicKey"
          },
          {
            "name": "token1Vault",
            "docs": [
              "Token B"
            ],
            "type": "publicKey"
          },
          {
            "name": "lpMint",
            "docs": [
              "Pool tokens are issued when A or B tokens are deposited.",
              "Pool tokens can be withdrawn back to the original A or B token."
            ],
            "type": "publicKey"
          },
          {
            "name": "token0Mint",
            "docs": [
              "Mint information for token A"
            ],
            "type": "publicKey"
          },
          {
            "name": "token1Mint",
            "docs": [
              "Mint information for token B"
            ],
            "type": "publicKey"
          },
          {
            "name": "token0Program",
            "docs": [
              "token_0 program"
            ],
            "type": "publicKey"
          },
          {
            "name": "token1Program",
            "docs": [
              "token_1 program"
            ],
            "type": "publicKey"
          },
          {
            "name": "lpSupply",
            "docs": [
              "lp mint supply"
            ],
            "type": "u64"
          },
          {
            "name": "protocolFeesToken0",
            "docs": [
              "The amounts of token_0 and token_1 that are owed to the liquidity provider."
            ],
            "type": "u64"
          },
          {
            "name": "protocolFeesToken1",
            "type": "u64"
          },
          {
            "name": "fundFeesToken0",
            "type": "u64"
          },
          {
            "name": "fundFeesToken1",
            "type": "u64"
          },
          {
            "name": "openTime",
            "docs": [
              "The timestamp allowed for swap in the pool."
            ],
            "type": "u64"
          },
          {
            "name": "taxMint",
            "docs": [
              "Tax"
            ],
            "type": "publicKey"
          },
          {
            "name": "taxAuthority",
            "type": "publicKey"
          },
          {
            "name": "inTaxRate",
            "type": "u64"
          },
          {
            "name": "outTaxRate",
            "type": "u64"
          },
          {
            "name": "taxAmount0",
            "docs": [
              "total amount of tax in vault"
            ],
            "type": "u64"
          },
          {
            "name": "taxAmount1",
            "type": "u64"
          },
          {
            "name": "taxDisabled",
            "docs": [
              "tax status"
            ],
            "type": "bool"
          },
          {
            "name": "lpFeeRate",
            "docs": [
              "LP fee rate"
            ],
            "type": "u64"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u64",
                31
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "TradeDirection",
      "docs": [
        "The direction of a trade, since curves can be specialized to treat each",
        "token differently (by adding offsets or weights)"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "ZeroForOne"
          },
          {
            "name": "OneForZero"
          }
        ]
      }
    },
    {
      "name": "RoundDirection",
      "docs": [
        "The direction to round.  Used for pool token to trading token conversions to",
        "avoid losing value on any deposit or withdrawal."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Floor"
          },
          {
            "name": "Ceiling"
          }
        ]
      }
    },
    {
      "name": "PoolStatusBitIndex",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Deposit"
          },
          {
            "name": "Withdraw"
          },
          {
            "name": "Swap"
          }
        ]
      }
    },
    {
      "name": "PoolStatusBitFlag",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Enable"
          },
          {
            "name": "Disable"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "LpChangeEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": true
        },
        {
          "name": "amountLp",
          "type": "u64",
          "index": false
        },
        {
          "name": "amount0",
          "type": "u64",
          "index": false
        },
        {
          "name": "amount1",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve0",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve1",
          "type": "u64",
          "index": false
        },
        {
          "name": "changeType",
          "type": "u8",
          "index": false
        }
      ]
    },
    {
      "name": "SwapEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": true
        },
        {
          "name": "tokenIn",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "tokenOut",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amountIn",
          "type": "u64",
          "index": false
        },
        {
          "name": "amountOut",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve0",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve1",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "TaxConfigUpdatedEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "taxMint",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "taxAuthority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "inTaxRate",
          "type": "u64",
          "index": false
        },
        {
          "name": "outTaxRate",
          "type": "u64",
          "index": false
        },
        {
          "name": "taxDisabled",
          "type": "bool",
          "index": false
        }
      ]
    },
    {
      "name": "TaxCollectEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount0",
          "type": "u64",
          "index": false
        },
        {
          "name": "amount1",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "NotApproved",
      "msg": "Not approved"
    },
    {
      "code": 6001,
      "name": "InvalidOwner",
      "msg": "Input account owner is not the program address"
    },
    {
      "code": 6002,
      "name": "EmptySupply",
      "msg": "Input token account empty"
    },
    {
      "code": 6003,
      "name": "InvalidInput",
      "msg": "InvalidInput"
    },
    {
      "code": 6004,
      "name": "IncorrectLpMint",
      "msg": "Address of the provided lp token mint is incorrect"
    },
    {
      "code": 6005,
      "name": "ExceededSlippage",
      "msg": "Swap instruction exceeds desired slippage limit"
    },
    {
      "code": 6006,
      "name": "ZeroTradingTokens",
      "msg": "Given pool token amount results in zero trading tokens"
    },
    {
      "code": 6007,
      "name": "NotSupportMint",
      "msg": "Not support token_2022 mint extension"
    },
    {
      "code": 6008,
      "name": "InvalidVault",
      "msg": "invaild vault"
    },
    {
      "code": 6009,
      "name": "TaxAmountCalculationFailed",
      "msg": "Tax amount calculation failed"
    },
    {
      "code": 6010,
      "name": "NoPendingTax",
      "msg": "No pending tax"
    },
    {
      "code": 6011,
      "name": "NoPendingFee",
      "msg": "No pending fee"
    },
    {
      "code": 6012,
      "name": "TaxDisabled",
      "msg": "Tax disabled"
    }
  ]
};

export const IDL: Goatswap = {
  "version": "0.2.0",
  "name": "goatswap",
  "instructions": [
    {
      "name": "createAmmConfig",
      "docs": [
        "# Arguments",
        "",
        "* `ctx`- The accounts needed by instruction.",
        "* `index` - The index of amm config, there may be multiple config.",
        "* `trade_fee_rate` - Trade fee rate, can be changed.",
        "* `protocol_fee_rate` - The rate of protocol fee within tarde fee.",
        "* `fund_fee_rate` - The rate of fund fee within tarde fee.",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Address to be set as protocol owner."
          ]
        },
        {
          "name": "ammConfig",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Initialize config state account to store protocol owner address and fee rates."
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "index",
          "type": "u16"
        },
        {
          "name": "tradeFeeRate",
          "type": "u64"
        },
        {
          "name": "protocolFeeRate",
          "type": "u64"
        },
        {
          "name": "fundFeeRate",
          "type": "u64"
        },
        {
          "name": "createPoolFee",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateAmmConfig",
      "docs": [
        "Updates the owner of the amm config",
        "Must be called by the current owner or admin",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `trade_fee_rate`- The new trade fee rate of amm config, be set when `param` is 0",
        "* `protocol_fee_rate`- The new protocol fee rate of amm config, be set when `param` is 1",
        "* `fund_fee_rate`- The new fund fee rate of amm config, be set when `param` is 2",
        "* `new_procotol_owner`- The config's new owner, be set when `param` is 3",
        "* `new_fund_owner`- The config's new fund owner, be set when `param` is 4",
        "* `create_pool_fee`- The config's new owner, be set when `param` is 5",
        "* `disable_create_pool`- The config's new fund owner, be set when `param` is 6",
        "* `param`- The vaule can be 0 | 1 | 2 | 3 | 4 | 5 | 6, otherwise will report a error",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The amm config owner or admin"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Amm config account to be changed"
          ]
        }
      ],
      "args": [
        {
          "name": "param",
          "type": "u8"
        },
        {
          "name": "value",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updatePoolStatus",
      "docs": [
        "Update pool status for given vaule",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `status` - The vaule of status",
        ""
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "status",
          "type": "u8"
        }
      ]
    },
    {
      "name": "updatePoolTaxStatus",
      "docs": [
        "Update pool tax status for given vaule",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `tax_disabled` - The vaule of tax status of pool",
        ""
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "taxDisabled",
          "type": "bool"
        }
      ]
    },
    {
      "name": "updateTax",
      "docs": [
        "Update tax",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `tax_use_token_0` - tax use token0 or token1",
        "* `tax_fee_in_rate` - new tax fee rate of pool when swap in",
        "* `tax_fee_out_rate` - new tax fee rate of pool when swap out",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "taxUseToken0",
          "type": "bool"
        },
        {
          "name": "inTaxRate",
          "type": "u64"
        },
        {
          "name": "outTaxRate",
          "type": "u64"
        }
      ]
    },
    {
      "name": "updateLpFee",
      "docs": [
        "Update tax",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `lp_fee_rate` - lp fee rate value",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lpFeeRate",
          "type": "u64"
        }
      ]
    },
    {
      "name": "transferTaxAuthority",
      "docs": [
        "Update tax authority",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `new_authority` - new tax transfer authority",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool or admin"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "newAuthority",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "transferPoolOwner",
      "docs": [
        "Transfer pool owner",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `new_owner` - The new owner of pool",
        ""
      ],
      "accounts": [
        {
          "name": "authority",
          "isMut": false,
          "isSigner": true
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "newOwner",
          "type": "publicKey"
        }
      ]
    },
    {
      "name": "collectProtocolFee",
      "docs": [
        "Collect the protocol fee accrued to the pool",
        "",
        "# Arguments",
        "",
        "* `ctx` - The context of accounts",
        "* `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1",
        "* `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Only admin or owner can collect fee now"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool state stores accumulated protocol fee amount"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Amm config account stores owner"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "recipientToken0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_0 protocol fees"
          ]
        },
        {
          "name": "recipientToken1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_1 protocol fees"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program to perform token transfers"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program 2022 to perform token transfers"
          ]
        }
      ],
      "args": [
        {
          "name": "amount0Requested",
          "type": "u64"
        },
        {
          "name": "amount1Requested",
          "type": "u64"
        }
      ]
    },
    {
      "name": "collectFundFee",
      "docs": [
        "Collect the fund fee accrued to the pool",
        "",
        "# Arguments",
        "",
        "* `ctx` - The context of accounts",
        "* `amount_0_requested` - The maximum amount of token_0 to send, can be 0 to collect fees in only token_1",
        "* `amount_1_requested` - The maximum amount of token_1 to send, can be 0 to collect fees in only token_0",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Only admin or fund_owner can collect fee now"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool state stores accumulated protocol fee amount"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Amm config account stores fund_owner"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "recipientToken0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_0 fund fees"
          ]
        },
        {
          "name": "recipientToken1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_1 fund fees"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program to perform token transfers"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program 2022 to perform token transfers"
          ]
        }
      ],
      "args": [
        {
          "name": "amount0Requested",
          "type": "u64"
        },
        {
          "name": "amount1Requested",
          "type": "u64"
        }
      ]
    },
    {
      "name": "collectTax",
      "docs": [
        "Collect the tax accrued to the pool",
        "",
        "# Arguments",
        "",
        "* `ctx` - The context of accounts",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "owner of pool"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Amm config account stores fund_owner"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "recipientToken0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_0 tax"
          ]
        },
        {
          "name": "recipientToken1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that receives the collected token_1 tax"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program to perform token transfers"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The SPL program 2022 to perform token transfers"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "initialize",
      "docs": [
        "Creates a pool for the given token pair and the initial price",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `init_amount_0` - the initial amount_0 to deposit",
        "* `init_amount_1` - the initial amount_1 to deposit",
        "* `open_time` - the timestamp allowed for swap",
        ""
      ],
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Address paying to create the pool. Can be anyone"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Which config the pool belongs to."
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Initialize an account to store the pool state"
          ]
        },
        {
          "name": "token0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_0 mint, the key must smaller then token_1 mint."
          ]
        },
        {
          "name": "token1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_1 mint, the key must grater then token_0 mint."
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "pool lp mint"
          ]
        },
        {
          "name": "creatorToken0",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "payer token0 account"
          ]
        },
        {
          "name": "creatorToken1",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator token1 account"
          ]
        },
        {
          "name": "creatorLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator lp token account"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "createPoolFee",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create mint account and mint tokens"
          ]
        },
        {
          "name": "token0Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "token1Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create an ATA for receiving position NFT"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "To create a new program account"
          ]
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Sysvar for program account"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "initAmount0",
          "type": "u64"
        },
        {
          "name": "initAmount1",
          "type": "u64"
        },
        {
          "name": "openTime",
          "type": "u64"
        },
        {
          "name": "taxUseToken0",
          "type": "bool"
        },
        {
          "name": "inTaxRate",
          "type": "u64"
        },
        {
          "name": "outTaxRate",
          "type": "u64"
        },
        {
          "name": "lpFeeRate",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "initializeWhitelisted",
      "docs": [
        "(For whitelisted program only) Creates a pool for the given token pair and the initial price",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `init_amount_0` - the initial amount_0 to deposit",
        "* `init_amount_1` - the initial amount_1 to deposit",
        "* `open_time` - the timestamp allowed for swap",
        ""
      ],
      "accounts": [
        {
          "name": "creator",
          "isMut": true,
          "isSigner": true,
          "docs": [
            "Address paying to create the pool. Can be anyone"
          ]
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Which config the pool belongs to."
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Initialize an account to store the pool state"
          ]
        },
        {
          "name": "token0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_0 mint, the key must smaller then token_1 mint."
          ]
        },
        {
          "name": "token1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token_1 mint, the key must grater then token_0 mint."
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "pool lp mint"
          ]
        },
        {
          "name": "creatorToken0",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "payer token0 account"
          ]
        },
        {
          "name": "creatorToken1",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator token1 account"
          ]
        },
        {
          "name": "creatorLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "creator lp token account"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create mint account and mint tokens"
          ]
        },
        {
          "name": "token0Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "token1Program",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Spl token program or token program 2022"
          ]
        },
        {
          "name": "associatedTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Program to create an ATA for receiving position NFT"
          ]
        },
        {
          "name": "systemProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "To create a new program account"
          ]
        },
        {
          "name": "rent",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Sysvar for program account"
          ]
        },
        {
          "name": "instructionSysvarAccount",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "initAmount0",
          "type": "u64"
        },
        {
          "name": "initAmount1",
          "type": "u64"
        },
        {
          "name": "openTime",
          "type": "u64"
        },
        {
          "name": "taxUseToken0",
          "type": "bool"
        },
        {
          "name": "inTaxRate",
          "type": "u64"
        },
        {
          "name": "outTaxRate",
          "type": "u64"
        },
        {
          "name": "lpFeeRate",
          "type": {
            "option": "u64"
          }
        }
      ]
    },
    {
      "name": "deposit",
      "docs": [
        "Creates a pool for the given token pair and the initial price",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `lp_token_amount` - Pool token amount to transfer. token_a and token_b amount are set by the current exchange rate and size of the pool",
        "* `maximum_token_0_amount` -  Maximum token 0 amount to deposit, prevents excessive slippage",
        "* `maximum_token_1_amount` - Maximum token 1 amount to deposit, prevents excessive slippage",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Pays to mint the position"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "ownerLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Owner lp tokan account"
          ]
        },
        {
          "name": "token0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The payer's token account for token_0"
          ]
        },
        {
          "name": "token1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The payer's token account for token_1"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "token Program"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token program 2022"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Lp token mint"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lpTokenAmount",
          "type": "u64"
        },
        {
          "name": "maximumToken0Amount",
          "type": "u64"
        },
        {
          "name": "maximumToken1Amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "withdraw",
      "docs": [
        "Withdraw lp for token0 ande token1",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `lp_token_amount` - Amount of pool tokens to burn. User receives an output of token a and b based on the percentage of the pool tokens that are returned.",
        "* `minimum_token_0_amount` -  Minimum amount of token 0 to receive, prevents excessive slippage",
        "* `minimum_token_1_amount` -  Minimum amount of token 1 to receive, prevents excessive slippage",
        ""
      ],
      "accounts": [
        {
          "name": "owner",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "Pays to mint the position"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool state account"
          ]
        },
        {
          "name": "ownerLpToken",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Owner lp token account"
          ]
        },
        {
          "name": "token0Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The owner's token account for receive token_0"
          ]
        },
        {
          "name": "token1Account",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The owner's token account for receive token_1"
          ]
        },
        {
          "name": "token0Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_0"
          ]
        },
        {
          "name": "token1Vault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The address that holds pool tokens for token_1"
          ]
        },
        {
          "name": "tokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "token Program"
          ]
        },
        {
          "name": "tokenProgram2022",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "Token program 2022"
          ]
        },
        {
          "name": "vault0Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_0 vault"
          ]
        },
        {
          "name": "vault1Mint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of token_1 vault"
          ]
        },
        {
          "name": "lpMint",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "Pool lp token mint"
          ]
        },
        {
          "name": "memoProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "memo program"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "lpTokenAmount",
          "type": "u64"
        },
        {
          "name": "minimumToken0Amount",
          "type": "u64"
        },
        {
          "name": "minimumToken1Amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "swapBaseInput",
      "docs": [
        "Swap the tokens in the pool base input amount",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `amount_in` -  input amount to transfer, output to DESTINATION is based on the exchange rate",
        "* `minimum_amount_out` -  Minimum amount of output token, prevents excessive slippage",
        ""
      ],
      "accounts": [
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The user performing the swap"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The factory state to read protocol fees"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The program account of the pool in which the swap will be performed"
          ]
        },
        {
          "name": "inputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for input token"
          ]
        },
        {
          "name": "outputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for output token"
          ]
        },
        {
          "name": "inputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for input token"
          ]
        },
        {
          "name": "outputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for output token"
          ]
        },
        {
          "name": "inputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for input token transfers"
          ]
        },
        {
          "name": "outputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for output token transfers"
          ]
        },
        {
          "name": "inputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of input token"
          ]
        },
        {
          "name": "outputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of output token"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "amountIn",
          "type": "u64"
        },
        {
          "name": "minimumAmountOut",
          "type": "u64"
        }
      ]
    },
    {
      "name": "swapBaseOutput",
      "docs": [
        "Swap the tokens in the pool base output amount",
        "",
        "# Arguments",
        "",
        "* `ctx`- The context of accounts",
        "* `max_amount_in` -  input amount prevents excessive slippage",
        "* `amount_out` -  amount of output token",
        ""
      ],
      "accounts": [
        {
          "name": "payer",
          "isMut": false,
          "isSigner": true,
          "docs": [
            "The user performing the swap"
          ]
        },
        {
          "name": "authority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "ammConfig",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The factory state to read protocol fees"
          ]
        },
        {
          "name": "poolState",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The program account of the pool in which the swap will be performed"
          ]
        },
        {
          "name": "inputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for input token"
          ]
        },
        {
          "name": "outputTokenAccount",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The user token account for output token"
          ]
        },
        {
          "name": "inputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for input token"
          ]
        },
        {
          "name": "outputVault",
          "isMut": true,
          "isSigner": false,
          "docs": [
            "The vault token account for output token"
          ]
        },
        {
          "name": "inputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for input token transfers"
          ]
        },
        {
          "name": "outputTokenProgram",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "SPL program for output token transfers"
          ]
        },
        {
          "name": "inputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of input token"
          ]
        },
        {
          "name": "outputTokenMint",
          "isMut": false,
          "isSigner": false,
          "docs": [
            "The mint of output token"
          ]
        },
        {
          "name": "eventAuthority",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "maxAmountIn",
          "type": "u64"
        },
        {
          "name": "amountOut",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "ammConfig",
      "docs": [
        "Holds the current owner of the factory"
      ],
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "bump",
            "docs": [
              "Bump to identify PDA"
            ],
            "type": "u8"
          },
          {
            "name": "disableCreatePool",
            "docs": [
              "Status to control if new pool can be create"
            ],
            "type": "bool"
          },
          {
            "name": "index",
            "docs": [
              "Config index"
            ],
            "type": "u16"
          },
          {
            "name": "tradeFeeRate",
            "docs": [
              "The trade fee, denominated in hundredths of a bip (10^-6)"
            ],
            "type": "u64"
          },
          {
            "name": "protocolFeeRate",
            "docs": [
              "The protocol fee"
            ],
            "type": "u64"
          },
          {
            "name": "fundFeeRate",
            "docs": [
              "The fund fee, denominated in hundredths of a bip (10^-6)"
            ],
            "type": "u64"
          },
          {
            "name": "createPoolFee",
            "docs": [
              "Fee for create a new pool"
            ],
            "type": "u64"
          },
          {
            "name": "protocolOwner",
            "docs": [
              "Address of the protocol fee owner"
            ],
            "type": "publicKey"
          },
          {
            "name": "fundOwner",
            "docs": [
              "Address of the fund fee owner"
            ],
            "type": "publicKey"
          },
          {
            "name": "padding",
            "docs": [
              "padding"
            ],
            "type": {
              "array": [
                "u64",
                16
              ]
            }
          }
        ]
      }
    },
    {
      "name": "poolState",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "authBump",
            "type": "u8"
          },
          {
            "name": "status",
            "docs": [
              "Bitwise representation of the state of the pool",
              "bit0, 1: disable deposit(vaule is 1), 0: normal",
              "bit1, 1: disable withdraw(vaule is 2), 0: normal",
              "bit2, 1: disable swap(vaule is 4), 0: normal"
            ],
            "type": "u8"
          },
          {
            "name": "lpMintDecimals",
            "type": "u8"
          },
          {
            "name": "mint0Decimals",
            "docs": [
              "mint0 and mint1 decimals"
            ],
            "type": "u8"
          },
          {
            "name": "mint1Decimals",
            "type": "u8"
          },
          {
            "name": "ammConfig",
            "docs": [
              "Which config the pool belongs"
            ],
            "type": "publicKey"
          },
          {
            "name": "poolCreator",
            "docs": [
              "pool creator"
            ],
            "type": "publicKey"
          },
          {
            "name": "token0Vault",
            "docs": [
              "Token A"
            ],
            "type": "publicKey"
          },
          {
            "name": "token1Vault",
            "docs": [
              "Token B"
            ],
            "type": "publicKey"
          },
          {
            "name": "lpMint",
            "docs": [
              "Pool tokens are issued when A or B tokens are deposited.",
              "Pool tokens can be withdrawn back to the original A or B token."
            ],
            "type": "publicKey"
          },
          {
            "name": "token0Mint",
            "docs": [
              "Mint information for token A"
            ],
            "type": "publicKey"
          },
          {
            "name": "token1Mint",
            "docs": [
              "Mint information for token B"
            ],
            "type": "publicKey"
          },
          {
            "name": "token0Program",
            "docs": [
              "token_0 program"
            ],
            "type": "publicKey"
          },
          {
            "name": "token1Program",
            "docs": [
              "token_1 program"
            ],
            "type": "publicKey"
          },
          {
            "name": "lpSupply",
            "docs": [
              "lp mint supply"
            ],
            "type": "u64"
          },
          {
            "name": "protocolFeesToken0",
            "docs": [
              "The amounts of token_0 and token_1 that are owed to the liquidity provider."
            ],
            "type": "u64"
          },
          {
            "name": "protocolFeesToken1",
            "type": "u64"
          },
          {
            "name": "fundFeesToken0",
            "type": "u64"
          },
          {
            "name": "fundFeesToken1",
            "type": "u64"
          },
          {
            "name": "openTime",
            "docs": [
              "The timestamp allowed for swap in the pool."
            ],
            "type": "u64"
          },
          {
            "name": "taxMint",
            "docs": [
              "Tax"
            ],
            "type": "publicKey"
          },
          {
            "name": "taxAuthority",
            "type": "publicKey"
          },
          {
            "name": "inTaxRate",
            "type": "u64"
          },
          {
            "name": "outTaxRate",
            "type": "u64"
          },
          {
            "name": "taxAmount0",
            "docs": [
              "total amount of tax in vault"
            ],
            "type": "u64"
          },
          {
            "name": "taxAmount1",
            "type": "u64"
          },
          {
            "name": "taxDisabled",
            "docs": [
              "tax status"
            ],
            "type": "bool"
          },
          {
            "name": "lpFeeRate",
            "docs": [
              "LP fee rate"
            ],
            "type": "u64"
          },
          {
            "name": "padding",
            "type": {
              "array": [
                "u64",
                31
              ]
            }
          }
        ]
      }
    }
  ],
  "types": [
    {
      "name": "TradeDirection",
      "docs": [
        "The direction of a trade, since curves can be specialized to treat each",
        "token differently (by adding offsets or weights)"
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "ZeroForOne"
          },
          {
            "name": "OneForZero"
          }
        ]
      }
    },
    {
      "name": "RoundDirection",
      "docs": [
        "The direction to round.  Used for pool token to trading token conversions to",
        "avoid losing value on any deposit or withdrawal."
      ],
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Floor"
          },
          {
            "name": "Ceiling"
          }
        ]
      }
    },
    {
      "name": "PoolStatusBitIndex",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Deposit"
          },
          {
            "name": "Withdraw"
          },
          {
            "name": "Swap"
          }
        ]
      }
    },
    {
      "name": "PoolStatusBitFlag",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "Enable"
          },
          {
            "name": "Disable"
          }
        ]
      }
    }
  ],
  "events": [
    {
      "name": "LpChangeEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": true
        },
        {
          "name": "amountLp",
          "type": "u64",
          "index": false
        },
        {
          "name": "amount0",
          "type": "u64",
          "index": false
        },
        {
          "name": "amount1",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve0",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve1",
          "type": "u64",
          "index": false
        },
        {
          "name": "changeType",
          "type": "u8",
          "index": false
        }
      ]
    },
    {
      "name": "SwapEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": true
        },
        {
          "name": "tokenIn",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "tokenOut",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amountIn",
          "type": "u64",
          "index": false
        },
        {
          "name": "amountOut",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve0",
          "type": "u64",
          "index": false
        },
        {
          "name": "reserve1",
          "type": "u64",
          "index": false
        }
      ]
    },
    {
      "name": "TaxConfigUpdatedEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "taxMint",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "taxAuthority",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "inTaxRate",
          "type": "u64",
          "index": false
        },
        {
          "name": "outTaxRate",
          "type": "u64",
          "index": false
        },
        {
          "name": "taxDisabled",
          "type": "bool",
          "index": false
        }
      ]
    },
    {
      "name": "TaxCollectEvent",
      "fields": [
        {
          "name": "poolId",
          "type": "publicKey",
          "index": false
        },
        {
          "name": "amount0",
          "type": "u64",
          "index": false
        },
        {
          "name": "amount1",
          "type": "u64",
          "index": false
        }
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "NotApproved",
      "msg": "Not approved"
    },
    {
      "code": 6001,
      "name": "InvalidOwner",
      "msg": "Input account owner is not the program address"
    },
    {
      "code": 6002,
      "name": "EmptySupply",
      "msg": "Input token account empty"
    },
    {
      "code": 6003,
      "name": "InvalidInput",
      "msg": "InvalidInput"
    },
    {
      "code": 6004,
      "name": "IncorrectLpMint",
      "msg": "Address of the provided lp token mint is incorrect"
    },
    {
      "code": 6005,
      "name": "ExceededSlippage",
      "msg": "Swap instruction exceeds desired slippage limit"
    },
    {
      "code": 6006,
      "name": "ZeroTradingTokens",
      "msg": "Given pool token amount results in zero trading tokens"
    },
    {
      "code": 6007,
      "name": "NotSupportMint",
      "msg": "Not support token_2022 mint extension"
    },
    {
      "code": 6008,
      "name": "InvalidVault",
      "msg": "invaild vault"
    },
    {
      "code": 6009,
      "name": "TaxAmountCalculationFailed",
      "msg": "Tax amount calculation failed"
    },
    {
      "code": 6010,
      "name": "NoPendingTax",
      "msg": "No pending tax"
    },
    {
      "code": 6011,
      "name": "NoPendingFee",
      "msg": "No pending fee"
    },
    {
      "code": 6012,
      "name": "TaxDisabled",
      "msg": "Tax disabled"
    }
  ]
};
