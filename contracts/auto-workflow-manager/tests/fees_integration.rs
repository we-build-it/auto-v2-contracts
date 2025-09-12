use cosmwasm_std::{
  coins, Coin, Uint128
};
use std::collections::{HashSet};


use auto_workflow_manager::{
    contract::{execute as execute_workflow_manager, instantiate as instantiate_workflow_manager, query as query_workflow_manager, reply as reply_workflow_manager},
    msg::{
      ExecuteMsg as WorkflowManagerExecuteMsg, FeeTotal as WorkflowManagerFeeTotal, FeeType as WorkflowManagerFeeType, GetUserPaymentConfigResponse, InstantiateMsg as WorkflowManagerInstantiateMsg, QueryMsg as WorkflowManagerQueryMsg, UserFee as WorkflowManagerUserFee
    }, 
    state::{
      PaymentConfig as WorkflowManagerPaymentConfig, 
      PaymentSource as WorkflowManagerPaymentSource,
    },
};

use auto_fee_manager::{
    contract::{execute as execute_fee_manager, instantiate as instantiate_fee_manager, sudo as sudo_fee_manager, query as query_fee_manager},
    msg::{AcceptedDenom, ExecuteMsg as FeeManagerExecuteMsg, InstantiateMsg as FeeManagerInstantiateMsg, SudoMsg as FeeManagerSudoMsg, QueryMsg as FeeManagerQueryMsg, UserBalancesResponse as FeeManagerUserBalancesResponse},
};

use cw_multi_test::{App, ContractWrapper, Executor};

/*
Run with:
cargo test --test fees_integration -- --nocapture
*/

#[test]
fn test_charge_fees_ok() {
  let mut app = App::default();
  let creator_address = app.api().addr_make("creator");

  app.init_modules(|router, _, storage| {
    router
        .bank
        .init_balance(storage, &creator_address, coins(5_000_000_000_000, "uusdc"))
        .unwrap();
  });

  let code_id_fee_manager = app.store_code(Box::new(ContractWrapper::new(execute_fee_manager, instantiate_fee_manager, query_fee_manager).with_sudo(sudo_fee_manager)));
  let code_id_workflow_manager = app.store_code(Box::new(ContractWrapper::new(execute_workflow_manager, instantiate_workflow_manager, query_workflow_manager).with_reply(reply_workflow_manager)));

  // Instantiate fee manager
  let accepted_denoms = vec![
    AcceptedDenom {
      denom: "uusdc".to_string(),
      max_debt: Uint128::from(1000u128),
      min_balance_threshold: Uint128::from(100u128),
    },
    AcceptedDenom {
      denom: "uruji".to_string(),
      max_debt: Uint128::zero(),
      min_balance_threshold: Uint128::zero(),
    },
  ];
  let fees_destination_address = app.api().addr_make("fees_destination");
  let crank_address = app.api().addr_make("crank_address");
  let fee_manager_instantiate_msg = FeeManagerInstantiateMsg {
    accepted_denoms: accepted_denoms,
    execution_fees_destination_address: fees_destination_address.clone(),
    distribution_fees_destination_address: fees_destination_address.clone(),
    crank_authorized_address: crank_address.clone(),
    workflow_manager_address: None,
    creator_distribution_fee: Uint128::zero(),
  };  
  let fee_manager_address = app.instantiate_contract(code_id_fee_manager, creator_address.clone(), &fee_manager_instantiate_msg, &[], "fee_manager", None).unwrap();

  // Instantiate workflow manager
  let publisher_address = app.api().addr_make("publisher");
  let executor_address = app.api().addr_make("executor");
  let workflow_manager_instantiate_msg = WorkflowManagerInstantiateMsg {
    allowed_publishers: HashSet::from([publisher_address.clone()]),
    allowed_action_executors: HashSet::from([executor_address.clone()]),
    referral_memo: "test-referral-memo".to_string(),
    fee_manager_address: fee_manager_address.clone(),
    allowance_denom: "uusdc".to_string(),
  };  
  let workflow_manager_address = app.instantiate_contract(code_id_workflow_manager, creator_address.clone(), &workflow_manager_instantiate_msg, &[], "workflow_manager", None).unwrap();

  // Set workflow manager address
  app.wasm_sudo(fee_manager_address.clone(), &FeeManagerSudoMsg::SetWorkflowManagerAddress { address: (workflow_manager_address.clone()) }).unwrap();

  // Get config
  let fee_manager_config: FeeManagerInstantiateMsg = app.wrap().query_wasm_smart(fee_manager_address.clone(), &FeeManagerQueryMsg::GetConfig {}).unwrap();
  let workflow_manager_config: WorkflowManagerInstantiateMsg = app.wrap().query_wasm_smart(workflow_manager_address.clone(), &WorkflowManagerQueryMsg::GetConfig {}).unwrap();
  
  println!("--------------------------------------------------");
  println!("fee_manager_address: {}", fee_manager_address);
  println!("workflow_manager_address: {}", workflow_manager_address);
  println!("--------------------------------------------------");
  println!("fee_manager_config: {:?}", fee_manager_config);
  println!("workflow_manager_config: {:?}", workflow_manager_config);
  println!("--------------------------------------------------");

  // Set user payment config
  let set_user_payment_config_msg = WorkflowManagerExecuteMsg::SetUserPaymentConfig {
    payment_config: WorkflowManagerPaymentConfig {
      allowance: Uint128::from(100_000_000u128),
      source: WorkflowManagerPaymentSource::Prepaid,
    },
  };
  app.execute_contract(
    creator_address.clone(), 
    workflow_manager_address.clone(), 
    &set_user_payment_config_msg, 
    &[]
  ).unwrap();
  println!("User payment config set");

  // Get user payment config
  let user_payment_config: GetUserPaymentConfigResponse = app.wrap().query_wasm_smart(workflow_manager_address.clone(), &WorkflowManagerQueryMsg::GetUserPaymentConfig { user_address: creator_address.to_string() }).unwrap();
  println!("user_payment_config: {:#?}", user_payment_config);
  println!("--------------------------------------------------");

  // Deposit to fee manager
  let deposit_msg = FeeManagerExecuteMsg::Deposit {};
  app.execute_contract(
    creator_address.clone(), 
    fee_manager_address.clone(), 
    &deposit_msg, 
    &[Coin {
      denom: "uusdc".to_string(),
      amount: Uint128::from(100_000_000u128),
    }]
  ).unwrap();
  println!("Deposit to fee manager done");

  // Query user balances
  let user_balances: FeeManagerUserBalancesResponse = app.wrap().query_wasm_smart(fee_manager_address.clone(), &FeeManagerQueryMsg::GetUserBalances { user: creator_address.clone() }).unwrap();
  println!("user_balances: {:#?}", user_balances);
  println!("--------------------------------------------------");

  // Call ChargeFees
  let fees = vec![
    WorkflowManagerUserFee {
      address: creator_address.to_string(),
      totals: vec![
        WorkflowManagerFeeTotal {
          denom: "uusdc".to_string(),
          amount: Uint128::from(500_000u128),
          fee_type: WorkflowManagerFeeType::Execution,
        },
      ],
    },
  ];
  let charge_fees_msg = WorkflowManagerExecuteMsg::ChargeFees {
    batch_id: "1".to_string(),
    fees: fees,
  };
  app.execute_contract(
    creator_address.clone(), 
    workflow_manager_address.clone(), 
    &charge_fees_msg, 
    &[]
  ).unwrap();
  println!("Charge fees done");
  println!("--------------------------------------------------");

  // Query allowance and user balances
  // Get user payment config
  let user_payment_config: GetUserPaymentConfigResponse = app.wrap().query_wasm_smart(workflow_manager_address.clone(), &WorkflowManagerQueryMsg::GetUserPaymentConfig { user_address: creator_address.to_string() }).unwrap();
  println!("user_payment_config: {:#?}", user_payment_config);
  let user_balances: FeeManagerUserBalancesResponse = app.wrap().query_wasm_smart(fee_manager_address.clone(), &FeeManagerQueryMsg::GetUserBalances { user: creator_address.clone() }).unwrap();
  println!("user_balances: {:#?}", user_balances);
  println!("--------------------------------------------------");
}
