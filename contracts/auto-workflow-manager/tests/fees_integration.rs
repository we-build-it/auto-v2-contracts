use cosmwasm_std::{
  coins, Coin, Decimal, Event, Uint128
};
use std::{collections::{HashMap, HashSet}, str::FromStr};


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
    contract::{execute as execute_fee_manager, instantiate as instantiate_fee_manager, query as query_fee_manager, sudo as sudo_fee_manager},
    msg::{AcceptedDenom, ExecuteMsg as FeeManagerExecuteMsg, InstantiateMsg as FeeManagerInstantiateMsg, QueryMsg as FeeManagerQueryMsg, SudoMsg as FeeManagerSudoMsg, UserBalancesResponse as FeeManagerUserBalancesResponse},
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
  let prices = HashMap::from([
    ("uusdc".to_string(), Decimal::from_str("1.0").unwrap()),
    ("rune".to_string(), Decimal::from_str("0.25").unwrap()),
  ]);  
  let fees = vec![
    WorkflowManagerUserFee {
      address: creator_address.to_string(),
      totals: vec![
        WorkflowManagerFeeTotal {
          denom: "uusdc".to_string(),
          amount: Uint128::from(500_000u128),
          fee_type: WorkflowManagerFeeType::Execution,
        },
        WorkflowManagerFeeTotal {
          denom: "rune".to_string(),
          amount: Uint128::from(100_000u128),
          fee_type: WorkflowManagerFeeType::Execution,
        },
      ],
    },
  ];
  let charge_fees_msg = WorkflowManagerExecuteMsg::ChargeFees {
    batch_id: "1".to_string(),
    prices: prices,
    fees: fees,
  };
  let charge_fees_result = app.execute_contract(
    creator_address.clone(), 
    workflow_manager_address.clone(), 
    &charge_fees_msg, 
    &[]
  ).unwrap();
  println!("charge_fees_result: {:#?}", charge_fees_result);

  let uusdc_charged_fee = get_fee_charged_event_amount(&charge_fees_result.events, creator_address.to_string(), "uusdc".to_string(), WorkflowManagerFeeType::Execution);
  println!("usdc_charged_fee: {:?}", uusdc_charged_fee);
  assert_eq!(uusdc_charged_fee.clone().unwrap().original_amount_charged, "500000");
  assert_eq!(uusdc_charged_fee.clone().unwrap().discounted_from_allowance, "500000");
  assert_eq!(uusdc_charged_fee.clone().unwrap().debit_denom, "uusdc");

  let rune_charged_fee = get_fee_charged_event_amount(&charge_fees_result.events, creator_address.to_string(), "rune".to_string(), WorkflowManagerFeeType::Execution);
  println!("rune_charged_fee: {:?}", rune_charged_fee);
  assert_eq!(rune_charged_fee.clone().unwrap().original_amount_charged, "100000");
  assert_eq!(rune_charged_fee.clone().unwrap().discounted_from_allowance, "25000");
  assert_eq!(rune_charged_fee.clone().unwrap().debit_denom, "uusdc");

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

#[derive(Debug)]
#[derive(Clone)]
#[allow(dead_code)]
struct FeeChargedEventAmount {
  original_amount_charged: String,
  discounted_from_allowance: String,
  debit_denom: String,
}

fn get_fee_charged_event_amount(
  events: &Vec<Event>, 
  user_address: String, 
  denom: String,
  fee_type: WorkflowManagerFeeType,
) -> Option<FeeChargedEventAmount> {
  let fee_charged_event = events.iter()
    .find(|event| event.ty == "wasm-autorujira-workflow-manager/fee-charged"
      && event.attributes.iter().any(|attr| attr.key == "user_address" && attr.value == user_address) 
      && event.attributes.iter().any(|attr| attr.key == "original_denom" && attr.value == denom) 
      && event.attributes.iter().any(|attr| attr.key == "fee_type" && attr.value == fee_type.to_string()))
  .unwrap();
  let amount = fee_charged_event.attributes.iter().find(|attr| attr.key == "original_amount_charged").unwrap().value.clone();
  let allowance_charged = fee_charged_event.attributes.iter().find(|attr| attr.key == "discounted_from_allowance").unwrap().value.clone();
  let denom_charged = fee_charged_event.attributes.iter().find(|attr| attr.key == "debit_denom").unwrap().value.clone();
  Some(FeeChargedEventAmount { original_amount_charged: amount, discounted_from_allowance: allowance_charged, debit_denom: denom_charged })
}