use anyhow::Result;
use ethers::{
    providers::{Middleware,Provider,Ws},
    types::{BlockNumber,H160,Address},
};

use log::info;
use std::{str::FromStr,sync::Arc};

use revm_is_all_you_need::constants::Env;
use revm_is_all_you_need::revm_examples::{
    create_evm_instance,
    evm_env_setup,
    get_token_balance
};

use revm_is_all_you_need::utils::setup_logger;




/*
 *  1.create the EVM instance
 *  2.Retrieving ERC-20 token balance
 * 
 */

//  use ethers::abi::ParamType::Address;
//  use ethers::abi::Token::Address;
 // use ethers::types::AddressOrBytes::Address;
 // use ethers::types::NameOrAddress::Address;
 // use ethers_core::abi::ParamType::Address;
 // use ethers_core::abi::Token::Address;
 // use ethers_core::types::AddressOrBytes::Address;
 // use ethers_core::types::NameOrAddress::Address;
 
 
 


#[tokio::main]
async fn main() ->Result<()> {
    dotenv::dotenv().ok();
    setup_logger()?;

    let mut evm = create_evm_instance();
    evm_env_setup(&mut evm);

    let user = "0xE2b5A9c1e325511a227EF527af38c3A7B65AFA1d".to_string();
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let usdt = H160::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap();
    let usdc =H160::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
    let dai = H160::from_str("0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap();

    let weth_balance = get_token_balance(&mut evm, weth, user);
    info!("WETH balance: {:?}",weth_balance);
    Ok(())
}


