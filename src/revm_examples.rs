use anyhow::{Result};
use anyhow::anyhow;
use std::result::Result::Ok;
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
use revm::{self,primitives::Address};
use revm::{
    db::{CacheDB,EmptyDB,EthersDB,InMemoryDB},
    primitives::Bytecode,
    primitives::{
        keccak256,AccountInfo,ExecutionResult,Log,Output,TransactTo,TxEnv,U256 as rU256,B256,B160
    },
    Database,
    EVM,
};


// precompile::Address,
use std::{str::FromStr,sync::Arc};
use crate::constants::SIMULATOR_CODE;
use crate::trace::get_state_diff;




// use alloy_primitives::{address, Address};
// use revm::primitives::alloy_primitives::Address;
// use revm::primitives::alloy_primitives::address;
// use ethers::types::Address;


// use alloy_primitives::{address, Address};
// use revm::primitives::alloy_primitives::Address;

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

//   use ::core::result::Result::Ok;
//   `, `use crate::revm_examples::alloy_primitives::private::Ok;
//   `, `use std::result::Result::Ok;


 pub fn get_token_balance(evm:&mut EVM<InMemoryDB>,token_address:String,account_address:String) -> Result<U256>{
    let erc20_abi = BaseContract::from(parse_abi(&[
        "function balanceOf(address) external view returns (uint256)",
    ])?);
    
    let account = H160::from_str(&account_address)?;
    let token = Address::from_str(&token_address)?;


    //caller: 谁会调用这个函数
    //transact_to:我们调用的函数
    //data: 我们交易的输入数据
    // evm.env.tx.caller = Address::from_str(&account_address);
    evm.env.tx.caller = account.into();
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


 pub async fn geth_and_revm_tracing<M:Middleware + 'static>(
    evm:&mut EVM<InMemoryDB>,
    provider:Arc<M>,
    token:H160,
    account:H160,
 ) -> Result<i32>{
    //create an Eip1559 transaction object:
    let erc20_abi: BaseContract = BaseContract::from(parse_abi(&[
        "function balanceOf(address) external view returns (uint256)",
    ])?);
    let calldata = erc20_abi.encode("balanceOf",account)?;
    
    let block = provider
        .get_block(BlockNumber::Latest)
        .await?
        .ok_or(anyhow::anyhow!("failed to retrieve block"))?;

    let nonce = provider
        .get_transaction_count(account, Some(BlockId::Number(BlockNumber::Latest)))
        .await?;

    let chain_id = provider.get_chainid().await?;

    let tx = Eip1559TransactionRequest{
        chain_id:Some(chain_id.as_u64().into()),
        nonce:Some(nonce),
        from:Some(account),
        to:Some(NameOrAddress::Address(token)),
        gas:None,
        value:None,
        data:Some(calldata),
        max_priority_fee_per_gas:None,
        max_fee_per_gas:None,
        access_list:AccessList::default(),
    };

    //call debug_trace_call
    let geth_trace = get_state_diff(provider.clone(), tx, block.number.unwrap()).await?;
    // println!("geth_trace:{:?}",geth_trace);

    let prestate = match geth_trace{
        GethTrace::Known(known) => match known{
            GethTraceFrame::PreStateTracer(prestate) => match prestate{
                PreStateFrame::Default(prestate_mode) => Some(prestate_mode),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }.unwrap();

    let geth_touched_accs = prestate.0.keys();
    info!("Geth trace: {:?}",geth_touched_accs);

    let token_acc_state = prestate.0.get(&token).ok_or(anyhow::anyhow!("no token key"))?;
    let token_touched_storage = token_acc_state
        .storage
        .clone()
        .ok_or(anyhow::anyhow!("no storage values"))?;

    for i in 0..20{
        let slot = keccak256(&abi::encode(&[
            abi::Token::Address(account.into()),
            abi::Token::Uint(U256::from(i)),
        ]));
        info!("{} {:?}",i,slot);
        // match token_touched_storage.get(&slot){
        //     Some(_) => {
        //         info!("Balance storage slot:{:?} ({:?})",i,slot);
        //         return Ok(i);
        //     }
        //     None => {}
        // }
    }

    Ok(0)
 }

/*
pub async fn revm_contract_deploy_and_tracing<M:Middleware + 'static>(
    evm:&mut EVM<InMemoryDB>,
    provider:Arc<M>,
    token_address:String,
    account_address:String
) -> Result<i32>{

    //deploy contract to EVM
    let account = H160::from_str(&account_address)?;
    let token = Address::from_str(&token_address)?;
    let block = provider
                .get_block(BlockNumber::Latest)
                .await?
                .ok_or(anyhow::anyhow!("failed to retrieve block"))?;
    
    let mut ethersdb = EthersDB::new(provider.clone(),Some(block.number.unwrap().into())).expect("create EthersDB failed");

    let token_acc_info = ethersdb.basic(token).unwrap().unwrap();

    // println!("token_acc_info: {:?}",token_acc_info);
    
    evm.db.as_mut().unwrap().insert_account_info(token, token_acc_info);

    let erc20_abi = BaseContract::from(parse_abi(&[
        "function balanceOf(address) external view returns (uint256)",
    ])?);

    let calldata = erc20_abi.encode("balanceOf",account)?;

    evm.env.tx.caller = Address::from_str(&account_address);
    evm.env.tx.transact_to = TransactTo::Call(token.into());
    evm.env.tx.data= calldata.0.into();

    let result = match evm.transact_ref() {
        Ok(result) => result,
        Err(e) => return Err(anyhow::anyhow!("EVM call failed: {e:?}")),
    };

    let token_acc = result.state.get(&token).unwrap();
    let token_touched_storage = token_acc.storage.clone();
    info!("Touched storage slots: {:?}",token_touched_storage);

    for i in 0..20{
        let slot = keccak256(&abi::encode(&[
            abi::Token::Address(H160::from_str(&account_address)?),
            abi::Token::Uint(U256::from(i)),
        ]));
        let slot:rU256 = slot.into();
        match token_touched_storage.get(&slot){
            Some(_) => {
                info!("Balance storage slot: {:?} ({:?})",i,slot);
                return Ok(i);
            }
            None => {}
        }

    }
    println!("hello");
    Ok(0)
}

 */

/**
 * 通过REVM 实现模拟交易
 */
pub async fn revm_v2_simulate_swap<M: Middleware + 'static>(
    evm: &mut EVM<InMemoryDB>,
    provider: Arc<M>,
    account: H160,
    factory: H160,
    target_pair: H160,
    input_token: H160,
    output_token: H160,
    input_balance_slot: i32,
    output_balance_slot: i32,
    input_token_implementation: Option<H160>,
    output_token_implementation: Option<H160>,
) -> Result<(U256, U256)> {


    //获取区块号
    let block = provider
    .get_block(BlockNumber::Latest)
    .await?
    .ok_or(anyhow!("failed to retrieve block"))?;

    let mut ethersdb = EthersDB::new(provider.clone(), Some(block.number.unwrap().into())).unwrap();

    let db = evm.db.as_mut().unwrap();


    let ten_eth = rU256::from(10)
    .checked_mul(rU256::from(10).pow(rU256::from(18)))
    .unwrap();

    // Set user: give the user enough ETH to pay for gas
    let user_acc_info:AccountInfo = AccountInfo::new(rU256::ZERO, 0, B256::zero(), Bytecode::default());
    db.insert_account_info(account.into(), user_acc_info);

    // Deploy Simulator contract
    // let simulator_address = H160::from_str("0xF2d01Ee818509a9540d8324a5bA52329af27D19E").unwrap();
    // let simulator_acc_info = AccountInfo::new(
    //     rU256::ZERO,
    //     0,
    //     Bytecode::new_raw((*SIMULATOR_CODE.0).into()),
    // );
    // db.insert_account_info(simulator_address.into(), simulator_acc_info);



    Ok((U256::zero(), U256::zero())) // 👈 placeholder for now, will update later
}