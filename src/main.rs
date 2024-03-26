use anyhow::Result;
use ethers::{
    providers::{Middleware,Provider,Ws},
    types::{BlockNumber,H160,Address},
};

use log::info;
use std::{str::FromStr,sync::Arc};

use revm_is_all_you_need::{revm_examples::revm_contract_deploy_and_tracing};
use revm_is_all_you_need::{constants::Env};
use revm_is_all_you_need::revm_examples::{
    create_evm_instance,
    evm_env_setup,
    get_token_balance,
    geth_and_revm_tracing,
    revm_v2_simulate_swap
};

use revm_is_all_you_need::tokens::get_implementation;

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

    let user = H160::from_str("0xE2b5A9c1e325511a227EF527af38c3A7B65AFA1d").unwrap();
    let weth = H160::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap();
    let usdt = H160::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap();
    let usdc = H160::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
    let dai = H160::from_str("0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap();

    // let weth_balance = get_token_balance(&mut evm, weth.to_string(), user.to_string());
    // info!("WETH balance: {:?}",weth_balance);


    let env = Env::new();
    let ws = Ws::connect(&env.wss_url).await.unwrap();
    let provider = Arc::new(Provider::new(ws));

    
    
    match geth_and_revm_tracing(&mut evm, provider.clone(),weth, user).await{
        Ok(_) => {info!("geth_and_revm_tracing")}
        Err(e) => info!("Tracing error: {e:?}"),
    }

    match revm_contract_deploy_and_tracing(&mut evm, provider.clone(), weth, user).await{
        Ok(_) => {println!("revm_contract_deploy_and_tracing success");}
        Err(e) => info!("Tracing error: {e:?}"),
    }

    let block = provider
        .get_block(BlockNumber::Latest)
        .await
        .unwrap()
        .unwrap();

    /***********************************************模拟swap**************************************************/
    let uniswap_v2_factory = H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap();
    let weth_usdt_pair = H160::from_str("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852").unwrap();

    let weth_balance_slot =
        revm_contract_deploy_and_tracing(&mut evm, provider.clone(), weth, user)
            .await
            .unwrap();
    let usdt_balance_slot =
        revm_contract_deploy_and_tracing(&mut evm, provider.clone(), usdt, user)
            .await
            .unwrap();

    let weth_implementation = get_implementation(provider.clone(), weth, block.number.unwrap())
        .await
        .unwrap();
    let usdt_implementation = get_implementation(provider.clone(), usdt, block.number.unwrap())
        .await
        .unwrap();

    info!("WETH proxy: {:?}", weth_implementation);
    info!("USDT proxy: {:?}", usdt_implementation);

    match revm_v2_simulate_swap(
        &mut evm,
        provider.clone(),
        user,
        uniswap_v2_factory,
        weth_usdt_pair,
        weth,
        usdt,
        weth_balance_slot,
        usdt_balance_slot,
        weth_implementation,
        usdt_implementation,
    )
    .await
    {
        Ok(_) => {info!("revm_v2_simulate_swap")}
        Err(e) => info!("v2SimulateSwap revm failed: {e:?}"),
    }


    match foundry_v2_simulate_swap(
        provider.clone(),
        user,
        weth_usdt_pair,
        weth,
        usdt,
        weth_balance_slot,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => info!("v2SimulateSwap foundry evm failed: {e:?}"),
    }

    // match eth_call_get_token_balance(provider.clone(), weth, 3, user).await {
    //     Ok(_) => {}
    //     Err(_) => {}
    // }

    match eth_call_v2_simulate_swap(
        provider.clone(),
        user,
        weth_usdt_pair,
        weth,
        usdt,
        weth_balance_slot,
    )
    .await
    {
        Ok(_) => {}
        Err(e) => info!("v2SimulateSwap eth_call failed: {e:?}"),
    }

    println!("End");


    Ok(())
}


