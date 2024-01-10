use anyhow::Result;
use ethers::providers::*;
use ethers_core::k256::elliptic_curve::rand_core::block;
use ethers_core::types::Eip1559TransactionRequest;
use ethers_core::types::GethDebugBuiltInTracerType;
use ethers_core::types::GethDebugTracerType;
use ethers_core::types::GethDebugTracingCallOptions;
use ethers_core::types::GethDebugTracingOptions;
use ethers_core::types::U64;
use ethers_core::types::GethTrace;
use ethers_providers::Middleware;
use std::sync::Arc;


/*
{
    'id': 1,
    'method': 'debug_traceTransaction',
    'jsonrpc': '2.0',
    'params': [
        tx_hash,
        {'tracer': 'prestateTracer'}
    ]
}
 */

pub async fn get_state_diff<M:Middleware + 'static>(
    provider:Arc<M>,
    tx:Eip1559TransactionRequest,
    block_number:U64,
) -> Result<GethTrace>{
    let trace = provider
    .debug_trace_call(
        tx,
         Some(block_number.into()),
        GethDebugTracingCallOptions{
            tracing_options:GethDebugTracingOptions{
                disable_storage:None,
                disable_stack:None,
                enable_memory:None,
                enable_return_data:None,
                tracer:Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::PreStateTracer,)),
                tracer_config:None,
                timeout:None,
            },
            state_overrides:None,
            block_overrides:None
        },
    ).await?;

    Ok(trace)
}