use anybuf::Bufany;
use cosmwasm_std::{
  coin, testing::{MockApi, MockStorage}, Addr, AnyMsg, Api, BlockInfo, Coin, CosmosMsg, CustomMsg, CustomQuery, Decimal, Empty, Event, Storage, Uint128, WasmMsg
};
use serde::de::DeserializeOwned;
use std::{collections::{HashMap, HashSet}, str::FromStr};


use auto_workflow_manager::{
    contract::{
      execute as execute_workflow_manager, 
      instantiate as instantiate_workflow_manager, 
      query as query_workflow_manager, 
      reply as reply_workflow_manager,
      sudo as sudo_workflow_manager
    },
    msg::{
      ExecuteMsg as WorkflowManagerExecuteMsg, 
      FeeTotal as WorkflowManagerFeeTotal, 
      FeeType as WorkflowManagerFeeType, 
      GetUserPaymentConfigResponse, 
      InstantiateMsg as WorkflowManagerInstantiateMsg, 
      QueryMsg as WorkflowManagerQueryMsg, 
      UserFee as WorkflowManagerUserFee,
      SudoMsg as WorkflowManagerSudoMsg
    }, 
    state::{
      PaymentConfig as WorkflowManagerPaymentConfig, 
    },
};

use auto_fee_manager::{
    contract::{execute as execute_fee_manager, instantiate as instantiate_fee_manager, query as query_fee_manager, sudo as sudo_fee_manager},
    msg::{AcceptedDenomValue, ExecuteMsg as FeeManagerExecuteMsg, InstantiateMsg as FeeManagerInstantiateMsg, QueryMsg as FeeManagerQueryMsg, SudoMsg as FeeManagerSudoMsg, UserBalancesResponse as FeeManagerUserBalancesResponse},
};

use cw_multi_test::{error::AnyResult, App, AppResponse, BankKeeper, BasicAppBuilder, ContractWrapper, CosmosRouter, Executor, FailingModule, GovFailingModule, IbcFailingModule, Stargate, WasmKeeper};
use anyhow;

/*
Run with:
cargo test --test fees_integration -- --nocapture
*/

#[test]
fn test_charge_fees_ok_prepaid() {
  let mut app= BasicAppBuilder::new()
    .with_stargate(CustomStargate {})
    .build(|_, _, _| {});
  let addresses = deploy_contracts(&mut app);
  // println!("addresses: {:#?}", addresses);
  // println!("--------------------------------------------------");

  // Set user payment config
  let set_user_payment_config_msg = WorkflowManagerExecuteMsg::SetUserPaymentConfig {
    payment_config: WorkflowManagerPaymentConfig::Prepaid,
  };
  app.execute_contract(
    addresses.workflow_executor.clone(), 
    addresses.contract_workflow_manager.clone(), 
    &set_user_payment_config_msg, 
    &[]
  ).unwrap();
  // println!("User payment config set");

  // Get user payment config
  let _user_payment_config: GetUserPaymentConfigResponse = app.wrap().query_wasm_smart(
    addresses.contract_workflow_manager.clone(), 
    &WorkflowManagerQueryMsg::GetUserPaymentConfig { user_address: addresses.workflow_executor.to_string() }).unwrap();
  // println!("user_payment_config: {:#?}", user_payment_config);
  // println!("--------------------------------------------------");

  // Deposit to fee manager
  let deposit_msg = FeeManagerExecuteMsg::Deposit {};
  app.execute_contract(
    addresses.workflow_executor.clone(), 
    addresses.contract_fee_manager.clone(), 
    &deposit_msg, 
    &[Coin {
      denom: "uusdc".to_string(),
      amount: Uint128::from(100_000_000u128),
    }]
  ).unwrap();
  // println!("Deposit to fee manager done");

  // Query user balances
  let _user_balances: FeeManagerUserBalancesResponse = app.wrap().query_wasm_smart(addresses.contract_fee_manager.clone(), &FeeManagerQueryMsg::GetUserBalances { user: addresses.workflow_executor.clone() }).unwrap();
  // println!("user_balances: {:#?}", user_balances);
  // println!("--------------------------------------------------");

  // Call ChargeFees
  let prices = HashMap::from([
    ("uusdc".to_string(), Decimal::from_str("1.0").unwrap()),
    ("rune".to_string(), Decimal::from_str("0.25").unwrap()),
  ]);  
  let fees = vec![
    WorkflowManagerUserFee {
      address: addresses.workflow_executor.to_string(),
      totals: vec![
        WorkflowManagerFeeTotal {
          denom: "uusdc".to_string(),
          debit_denom: "uusdc".to_string(),
          amount: Uint128::from(500_000u128),
          fee_type: WorkflowManagerFeeType::Execution,
        },
        WorkflowManagerFeeTotal {
          denom: "rune".to_string(),
          debit_denom: "uusdc".to_string(),
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
    addresses.crank.clone(), 
    addresses.contract_workflow_manager.clone(), 
    &charge_fees_msg, 
    &[]
  ).unwrap();
  // println!("charge_fees_result: {:#?}", charge_fees_result);

  let uusdc_charged_fee = get_fee_charged_event_amount(&charge_fees_result.events, addresses.workflow_executor.to_string(), "uusdc".to_string(), WorkflowManagerFeeType::Execution);
  // println!("usdc_charged_fee: {:?}", uusdc_charged_fee);
  assert_eq!(uusdc_charged_fee.clone().unwrap().fee_denom, "uusdc");
  assert_eq!(uusdc_charged_fee.clone().unwrap().fee_amount, "500000");
  assert_eq!(uusdc_charged_fee.clone().unwrap().usd_amount, "500000");
  assert_eq!(uusdc_charged_fee.clone().unwrap().debit_denom, "uusdc");
  assert_eq!(uusdc_charged_fee.clone().unwrap().debit_amount, "500000");
    
  let rune_charged_fee = get_fee_charged_event_amount(&charge_fees_result.events, addresses.workflow_executor.to_string(), "rune".to_string(), WorkflowManagerFeeType::Execution);
  // println!("rune_charged_fee: {:?}", rune_charged_fee);
  assert_eq!(rune_charged_fee.clone().unwrap().fee_denom, "rune");
  assert_eq!(rune_charged_fee.clone().unwrap().fee_amount, "100000");
  assert_eq!(rune_charged_fee.clone().unwrap().usd_amount, "25000");
  assert_eq!(rune_charged_fee.clone().unwrap().debit_denom, "uusdc");
  assert_eq!(rune_charged_fee.clone().unwrap().debit_amount, "25000");

  // println!("Charge fees done");
  // println!("--------------------------------------------------");
  
  // Query allowance and user balances
  // Get user payment config
  let _user_payment_config: GetUserPaymentConfigResponse = app.wrap().query_wasm_smart(
    addresses.contract_workflow_manager.clone(), 
    &WorkflowManagerQueryMsg::GetUserPaymentConfig { user_address: addresses.workflow_executor.to_string() }).unwrap();
  // println!("user_payment_config: {:#?}", user_payment_config);
  let _user_balances: FeeManagerUserBalancesResponse = app.wrap().query_wasm_smart(
    addresses.contract_fee_manager.clone(), 
    &FeeManagerQueryMsg::GetUserBalances { user: addresses.workflow_executor.clone() }).unwrap();
  // println!("user_balances: {:#?}", user_balances);
  // println!("--------------------------------------------------");
}

#[test]
fn test_charge_fees_ok_wallet() {
  let mut app= BasicAppBuilder::new()
    .with_stargate(CustomStargate {})
    .build(|_, _, _| {});

  let addresses = deploy_contracts(&mut app);
  // println!("addresses: {:#?}", addresses);
  // println!("--------------------------------------------------");

  // Set user payment config
  let set_user_payment_config_msg = WorkflowManagerExecuteMsg::SetUserPaymentConfig {
    payment_config: WorkflowManagerPaymentConfig::Wallet {
      usd_allowance: Uint128::from(100_000_000u128),
    },
  };
  app.execute_contract(
    addresses.workflow_executor.clone(), 
    addresses.contract_workflow_manager.clone(), 
    &set_user_payment_config_msg, 
    &[]
  ).unwrap();
  // println!("User payment config set");

  // Get user payment config
  let _user_payment_config: GetUserPaymentConfigResponse = app.wrap().query_wasm_smart(
    addresses.contract_workflow_manager.clone(), 
    &WorkflowManagerQueryMsg::GetUserPaymentConfig { user_address: addresses.workflow_executor.to_string() }).unwrap();
  // println!("user_payment_config: {:#?}", user_payment_config);
  // println!("--------------------------------------------------");

  // Deposit to fee manager
  // println!("No deposit to fee manager needed as we are using wallet");

  // Query user balances
  let _user_balances: FeeManagerUserBalancesResponse = app.wrap().query_wasm_smart(addresses.contract_fee_manager.clone(), &FeeManagerQueryMsg::GetUserBalances { user: addresses.workflow_executor.clone() }).unwrap();
  // println!("user_balances: {:#?}", user_balances);
  // println!("--------------------------------------------------");

  // Call ChargeFees
  let prices = HashMap::from([
    ("uusdc".to_string(), Decimal::from_str("1.0").unwrap()),
    ("rune".to_string(), Decimal::from_str("0.25").unwrap()),
  ]);  
  let fees = vec![
    WorkflowManagerUserFee {
      address: addresses.workflow_executor.to_string(),
      totals: vec![
        WorkflowManagerFeeTotal {
          denom: "uusdc".to_string(),
          debit_denom: "rune".to_string(),
          amount: Uint128::from(500_000u128),
          fee_type: WorkflowManagerFeeType::Execution,
        },
        WorkflowManagerFeeTotal {
          denom: "rune".to_string(),
          debit_denom: "rune".to_string(),
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
    addresses.crank.clone(), 
    addresses.contract_workflow_manager.clone(), 
    &charge_fees_msg, 
    &[]
  ).unwrap();
  // println!("charge_fees_result: {:#?}", charge_fees_result);

  let uusdc_charged_fee = get_fee_charged_event_amount(&charge_fees_result.events, addresses.workflow_executor.to_string(), "uusdc".to_string(), WorkflowManagerFeeType::Execution);
  // println!("usdc_charged_fee: {:?}", uusdc_charged_fee);
  assert_eq!(uusdc_charged_fee.clone().unwrap().fee_denom, "uusdc");
  assert_eq!(uusdc_charged_fee.clone().unwrap().fee_amount, "500000");
  assert_eq!(uusdc_charged_fee.clone().unwrap().usd_amount, "500000");
  assert_eq!(uusdc_charged_fee.clone().unwrap().debit_denom, "rune");
  assert_eq!(uusdc_charged_fee.clone().unwrap().debit_amount, "2000000");

  let rune_charged_fee = get_fee_charged_event_amount(&charge_fees_result.events, addresses.workflow_executor.to_string(), "rune".to_string(), WorkflowManagerFeeType::Execution);
  // println!("rune_charged_fee: {:?}", rune_charged_fee);
  assert_eq!(rune_charged_fee.clone().unwrap().fee_denom, "rune");
  assert_eq!(rune_charged_fee.clone().unwrap().fee_amount, "100000");
  assert_eq!(rune_charged_fee.clone().unwrap().usd_amount, "25000");
  assert_eq!(rune_charged_fee.clone().unwrap().debit_denom, "rune");
  assert_eq!(rune_charged_fee.clone().unwrap().debit_amount, "100000");

  // println!("Charge fees done");
  // println!("--------------------------------------------------");
  
  // Query allowance and user balances
  // Get user payment config
  let _user_payment_config: GetUserPaymentConfigResponse = app.wrap().query_wasm_smart(
    addresses.contract_workflow_manager.clone(), 
    &WorkflowManagerQueryMsg::GetUserPaymentConfig { user_address: addresses.workflow_executor.to_string() }).unwrap();
  // println!("user_payment_config: {:#?}", user_payment_config);
  let _user_balances: FeeManagerUserBalancesResponse = app.wrap().query_wasm_smart(
    addresses.contract_fee_manager.clone(), 
    &FeeManagerQueryMsg::GetUserBalances { user: addresses.workflow_executor.clone() }).unwrap();
  // println!("user_balances: {:#?}", user_balances);
  // println!("--------------------------------------------------");
}

#[derive(Debug)]
#[derive(Clone)]
#[allow(dead_code)]
struct Addresses {
  contracts_creator: Addr,
  fees_destination: Addr,
  crank: Addr,
  workflow_publisher: Addr,
  workflow_executor: Addr,
  contract_fee_manager: Addr,
  contract_workflow_manager: Addr,
}

fn deploy_contracts(app: &mut CustomApp) -> Addresses {
  // Addresses
  let contracts_creator_addr = app.api().addr_make("creator");
  let fees_destination_addr = app.api().addr_make("fees_destination");
  let crank_addr = app.api().addr_make("crank");
  let publisher_addr = app.api().addr_make("publisher");
  let executor_addr = app.api().addr_make("executor");

  app.init_modules(|router, _, storage| {
    router
        .bank
        .init_balance(storage, &executor_addr, vec![
          coin(5_000_000_000_000, "uusdc"),
          coin(5_000_000_000_000, "uruji"),
          coin(5_000_000_000_000, "rune"),])
        .unwrap();
  });

  let code_id_fee_manager = app
    .store_code(Box::new(ContractWrapper::new(execute_fee_manager, instantiate_fee_manager, query_fee_manager)
    .with_sudo(sudo_fee_manager)));
  let code_id_workflow_manager = app
    .store_code(Box::new(ContractWrapper::new(execute_workflow_manager, instantiate_workflow_manager, query_workflow_manager)
    .with_sudo(sudo_workflow_manager)
    .with_reply(reply_workflow_manager)));

  // Instantiate fee manager
  let accepted_denoms: HashMap<String, AcceptedDenomValue> = vec![
    ("uusdc".to_string(),
    AcceptedDenomValue {
        max_debt: Uint128::from(1000u128),
        min_balance_threshold: Uint128::from(100u128),
    }),
    ("uruji".to_string(),
      AcceptedDenomValue {
        max_debt: Uint128::zero(),
        min_balance_threshold: Uint128::zero(),
      })
  ].into_iter().collect();

  let fee_manager_instantiate_msg = FeeManagerInstantiateMsg {
    accepted_denoms: accepted_denoms,
    execution_fees_destination_address: fees_destination_addr.clone(),
    distribution_fees_destination_address: fees_destination_addr.clone(),
    crank_authorized_address: crank_addr.clone(),
    workflow_manager_address: None,
    creator_distribution_fee: Uint128::zero(),
  };  
  let fee_manager_address = app.instantiate_contract(code_id_fee_manager, contracts_creator_addr.clone(), &fee_manager_instantiate_msg, &[], "fee_manager", None).unwrap();

  // Instantiate workflow manager
  let workflow_manager_instantiate_msg = WorkflowManagerInstantiateMsg {
    allowed_publishers: HashSet::from([publisher_addr.clone()]),
    allowed_action_executors: HashSet::from([crank_addr.clone()]),
    referral_memo: "test-referral-memo".to_string(),
    fee_manager_address: fee_manager_address.clone(),
  };  
  let workflow_manager_address = app.instantiate_contract(code_id_workflow_manager, contracts_creator_addr.clone(), &workflow_manager_instantiate_msg, &[], "workflow_manager", None).unwrap();

  // Set workflow manager owner as the crank
  app.wasm_sudo(workflow_manager_address.clone(), &WorkflowManagerSudoMsg::SetOwner(crank_addr.clone())).unwrap();

  // Set workflow manager address in fee manager
  app.wasm_sudo(fee_manager_address.clone(), &FeeManagerSudoMsg::SetWorkflowManagerAddress { address: (workflow_manager_address.clone()) }).unwrap();

  // Get config
  let _fee_manager_config: FeeManagerInstantiateMsg = app.wrap().query_wasm_smart(fee_manager_address.clone(), &FeeManagerQueryMsg::GetConfig {}).unwrap();
  let _workflow_manager_config: WorkflowManagerInstantiateMsg = app.wrap().query_wasm_smart(workflow_manager_address.clone(), &WorkflowManagerQueryMsg::GetConfig {}).unwrap();
  
  // println!("--------------------------------------------------");
  // println!("fee_manager_address: {}", fee_manager_address);
  // println!("workflow_manager_address: {}", workflow_manager_address);
  // println!("--------------------------------------------------");
  // println!("fee_manager_config: {:?}", fee_manager_config);
  // println!("workflow_manager_config: {:?}", workflow_manager_config);
  // println!("--------------------------------------------------");

  Addresses {
    contracts_creator: contracts_creator_addr,
    fees_destination: fees_destination_addr,
    crank: crank_addr,
    workflow_publisher: publisher_addr,
    workflow_executor: executor_addr,
    contract_fee_manager: fee_manager_address,
    contract_workflow_manager: workflow_manager_address,
  }
}
#[derive(Debug)]
#[derive(Clone)]
#[allow(dead_code)]
struct FeeChargedEventAmount {
  fee_denom: String,
  fee_amount: String,
  usd_amount: String,
  debit_denom: String,
  debit_amount: String,
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
      && event.attributes.iter().any(|attr| attr.key == "denom" && attr.value == denom) 
      && event.attributes.iter().any(|attr| attr.key == "fee_type" && attr.value == fee_type.to_string()))
  .unwrap();
  let amount = fee_charged_event.attributes.iter().find(|attr| attr.key == "amount").unwrap().value.clone();
  let usd_amount = fee_charged_event.attributes.iter().find(|attr| attr.key == "usd_amount").unwrap().value.clone();
  let debit_denom = fee_charged_event.attributes.iter().find(|attr| attr.key == "debit_denom").unwrap().value.clone();
  let debit_amount = fee_charged_event.attributes.iter().find(|attr| attr.key == "debit_amount").unwrap().value.clone();
  Some(FeeChargedEventAmount { 
    fee_denom: denom,
    fee_amount: amount,
    usd_amount: usd_amount,
    debit_denom: debit_denom,
    debit_amount: debit_amount,
  })
}

pub type CustomApp = App<
    BankKeeper,
    MockApi,
    MockStorage,
    // Custom
    FailingModule<Empty, Empty, Empty>,
    WasmKeeper<Empty, Empty>,
    // SDK Staking
    FailingModule<Empty, Empty, Empty>,
    // SDK Distribution
    FailingModule<Empty, Empty, Empty>,
    IbcFailingModule,
    GovFailingModule,
    CustomStargate,
>;

#[derive(Default)]
pub struct CustomStargate {}

impl Stargate for CustomStargate {

    fn execute_any<ExecC, QueryC>(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        router: &dyn CosmosRouter<ExecC = ExecC, QueryC = QueryC>,
        block: &BlockInfo,
        sender: Addr,
        msg: AnyMsg,
    ) -> AnyResult<AppResponse>
    where
        ExecC: CustomMsg + DeserializeOwned + 'static,
        QueryC: CustomQuery + DeserializeOwned + 'static,
    {
        let type_url = msg.type_url.clone();
        let serialized = msg.value.to_vec();
        let buf = Bufany::deserialize(&serialized)?;
        match type_url.as_str() {
            "/cosmos.authz.v1beta1.MsgExec" => {
              let _authz_grantee   = buf.string(1).unwrap();
              let buf_2 = buf.message(2).unwrap();
              let _message_type_url = buf_2.string(1).unwrap();
              let buf_3 = buf_2.message(2).unwrap();
              let authz_granter = buf_3.string(1).unwrap();
              let contract_to_call = buf_3.string(2).unwrap();
              let msg_to_call = buf_3.string(3).unwrap();

              let buf_funds = buf_3.repeated_message(5).unwrap();
              let mut funds = vec![];
              for fund in buf_funds {
                let fund_denom = fund.string(1).unwrap();
                let fund_amount = fund.string(2).unwrap();
                funds.push(Coin {
                  denom: fund_denom.into(),
                  amount: Uint128::from_str(fund_amount.as_str()).unwrap(),
                });
              }

              let wasm_msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_to_call.into(),
                msg: msg_to_call.as_bytes().into(),
                funds: funds.clone(),
              });

              let sender_addr = Addr::unchecked(authz_granter.as_str());
              let res = router.execute(api, storage, block, sender_addr, wasm_msg)?;
              Ok(res)              
            }            
            _ => {
                anyhow::bail!("Unexpected any execute: msg={:?} from {}", msg, sender)
            }
        }
    }

}
