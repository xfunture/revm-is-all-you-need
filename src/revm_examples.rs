use anyhow::{Result};
use bytes::Bytes;
use ethers::{
    abi::{self,parse_abi},
    prelude::*,
    providers::Middleware,
    types::{
        transaction::eip2930::AccessList,BlockId,BlockNumber,Eip1559TransactionRequest,
        NameOrAddress,U256,
    },
};

use log::info;

use futures::io::Empty;
use revm::{self, primitives::{alloy_primitives, address}};
use revm::{
    db::{CacheDB,EmptyDB,EthersDB,InMemoryDB},
    primitives::Bytecode,
    primitives::{
        keccak256,AccountInfo,ExecutionResult,Log,Output,TransactTo,TxEnv,U256 as rU256,
    },
    Database,
    EVM,
};
// precompile::Address,
use std::{str::FromStr,sync::Arc};




// use alloy_primitives::{address, Address};
// use revm::primitives::alloy_primitives::Address;
// use revm::primitives::alloy_primitives::address;
// use ethers::types::Address;


// use alloy_primitives::{address, Address};
use revm::primitives::alloy_primitives::Address;

// let checksummed = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
// let expected = address!("d8da6bf26964af9d7eed9e03e53415d37aa96045");
// let address = Address::parse_checksummed(checksummed, None).expect("valid checksum");
// assert_eq!(address, expected);

// // Format the address with the checksum
// assert_eq!(address.to_string(), checksummed);
// assert_eq!(address.to_checksum(None), checksummed);



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


 /**
  * 在REVM上运行交易有6种方式：
    transact: execute tx without writing to DB, returns the states
    inspect: execute tx with inspector, and without writing to DB, returns the states
    transact_commit: call “transact” and commit the state changes to DB
    inspect_commit: call “inspect” with inspector, and commit the state changes to DB
    transact_ref: call “transact” and do not commit changes to DB
    inspect_ref: call “inspect” with inspector, and do not commit changes to DB
  *
  * 
  */

 pub fn get_token_balance(evm:&mut EVM<InMemoryDB>,token_address:String,account_address:String) -> Result<U256>{
    let erc20_abi = BaseContract::from(parse_abi(&[
        "function balanceOf(address) external view returns (uint256)",
    ])?);
    
    let account = H160::from_str(&account_address)?;
    let token = Address::from_str(&token_address)?;


    //caller: 谁会调用这个函数
    //transact_to:我们调用的函数
    //data: 我们交易的输入数据
    evm.env.tx.caller = Address::from_str(&account_address)?;
    let calldata = erc20_abi.encode("balanceOf",account)?;
    evm.env.tx.transact_to = TransactTo::Call(token);
    evm.env.tx.data = calldata.0.into();
    // println!("calldata:{:?}",calldata);

    // /*

    let result: revm::primitives::ResultAndState = match evm.transact_ref(){
        Ok(result) => result,
        Err(e) => return Err(anyhow::anyhow!("EVM call failed:{e:?}")),
    };

    let tx_result = match result.result{
        ExecutionResult::Success { 
            gas_used,
            gas_refunded,
            output,
            logs,
            ..
                } => match output{
                    Output::Call(o) => TxResult{
                        output:o.into(),
                        logs:Some(logs),
                        gas_used,
                        gas_refunded,
                    },
                    Output::Create(o,_) => TxResult {
                        output:o.into(),
                        logs: Some(logs),
                        gas_used, 
                        gas_refunded }
                },                                           //Reverted  by 'REVERT' opcode that doesn't spend all gas.
        ExecutionResult::Revert {gas_used,              //Reverted for various reasons and spend all gas
                                 output} => {
                                                    return Err(anyhow::anyhow!("Evm REVERT: {:?} / Gas used: {:?}",output,gas_used))
                                                    }
        ExecutionResult::Halt { reason, gas_used ,..    //Halting will spend all the gas,and will be equal to gas_limit
                                                    } => return Err(anyhow::anyhow!("EVM HALT: {:?} Gas used: {:?}",reason,gas_used)),
    };

    let decoded_output = erc20_abi.decode_output("balanceOf",tx_result.output)?;
    Ok(decoded_output)

    // */
    // Ok(())

 }
