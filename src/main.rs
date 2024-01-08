use anyhow::{Result, Ok};
use bytes::Bytes;
use ethers::{
    abi::{self,parse_abi},
    prelude::*,
    providers::Middleware,
    types::{
        transaction::eip2930::AccessList,BlockId,BlockNumber,Eip1559TransactionRequest,
        NameOrAddress,H160,U256,
    },
};

use log::info;

use futures::io::Empty;
use revm::{
    db::{CacheDB,EmptyDB,EthersDB,InMemoryDB},
    primitives::Bytecode,
    primitives::{
        keccak256,AccountInfo,ExecutionResult,Log,Output,TransactTo,TxEnv,U256 as rU256,
    },
    Database,
    EVM, precompile::Address,
};

use std::{str::FromStr,sync::Arc};

/*
 *  1.create the EVM instance
 *  2.Retrieving ERC-20 token balance
 * 
 */

#[derive(Debug,Clone)]
pub struct TxResult{
    pub output:Bytes,
    pub logs:Option<Vec<Log>>,
    pub gas_used:u64,
    pub gas_refunded:u64,
}




/*
 *创建一个干净的以太坊环境，没有任何的存储值和合约
 */

pub fn create_evm_instance() -> EVM<InMemoryDB>{
    let db = CacheDB::new(EmptyDB::default());
    let mut evm = EVM::new();
    evm.database(db);
    evm
}

/**
 * 以太坊环境配置
 * 为了使测试更简单，覆盖一些默认的配置
 */

 pub fn evm_env_setup(evm:&mut EVM<InMemoryDB>){
     evm.env.cfg.limit_contract_code_size = Some(0x100000);
    //  evm.env.cfg.disable_block_gas_limit = true;
    //  evm.env.cfg.disable_base_fee = true;
 }


 pub fn get_token_balance(evm:&mut EVM<InMemoryDB>,token:Address,account:Address) -> Result<()>{
    let erc20_abi = BaseContract::from(parse_abi(&[
        "function balanceOf(address) external view returns (uint256)",
    ])?);
    let calldata = erc20_abi.encode("balanceOf",account)?;

    evm.env.tx.caller = account;
    evm.env.tx.transact_to = TransactTo::Call(token.into());
    Ok(())
 }

#[tokio::main]
async fn main() ->Result<()> {
    println!("Hello, world!");
    Ok(())
}


