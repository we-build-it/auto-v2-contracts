use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{coins, from_json, Binary};

use echo_contract::contract::{execute, instantiate, query};
use echo_contract::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use echo_contract::state::get_message_count;

#[test]
fn test_instantiate() {
    let mut deps = mock_dependencies();
    let _env = mock_env();
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));

    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };

    let res = instantiate(deps.as_mut(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // Verify admin was set
    let res = query(deps.as_ref(), QueryMsg::Config {}).unwrap();
    let config: echo_contract::msg::ConfigResponse = from_json(&res).unwrap();
    assert_eq!(admin_addr.to_string(), config.admin);
}

#[test]
fn test_echo_simple() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };
    instantiate(deps.as_mut(), info, msg).unwrap();

    // Test echo
    let user_addr = deps.api.addr_make("user");
    let info = cosmwasm_std::testing::message_info(&user_addr, &coins(100, "earth"));
    let message = Binary::from(b"Hello World");
    let msg = ExecuteMsg::Echo { message: message.clone(), attributes: vec![] };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Check that an event was emitted
    assert_eq!(1, res.events.len());
    let event = &res.events[0];
    assert_eq!("echo_message", event.ty);
    
    // Check attributes
    let sender_attr = event.attributes.iter().find(|attr| attr.key == "sender").unwrap();
    assert_eq!(user_addr.to_string(), sender_attr.value);
    
    let count_attr = event.attributes.iter().find(|attr| attr.key == "message_count").unwrap();
    assert_eq!("1", count_attr.value);
    
    let size_attr = event.attributes.iter().find(|attr| attr.key == "message_size").unwrap();
    assert_eq!("11", size_attr.value); // "Hello World" is 11 bytes

    // Verify message count was incremented
    let count = get_message_count(deps.as_ref().storage);
    assert_eq!(1, count);
}

#[test]
fn test_echo_with_attributes() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };
    instantiate(deps.as_mut(), info, msg).unwrap();

    // Test echo with attributes
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("user"), &coins(100, "earth"));
    let message = Binary::from(b"Test message");
    let attributes = vec![
        ("message_type".to_string(), "test".to_string()),
        ("priority".to_string(), "high".to_string()),
    ];
    let msg = ExecuteMsg::Echo { 
        message: message.clone(),
        attributes: attributes.clone(),
    };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Check that an event was emitted
    assert_eq!(1, res.events.len());
    let event = &res.events[0];
    assert_eq!("echo_message", event.ty);
    
    // Check custom attributes were added
    let msg_type_attr = event.attributes.iter().find(|attr| attr.key == "message_type").unwrap();
    assert_eq!("test", msg_type_attr.value);
    
    let priority_attr = event.attributes.iter().find(|attr| attr.key == "priority").unwrap();
    assert_eq!("high", priority_attr.value);

    // Verify message count was incremented
    let count = get_message_count(deps.as_ref().storage);
    assert_eq!(1, count);
}

#[test]
fn test_message_count_query() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };
    instantiate(deps.as_mut(), info, msg).unwrap();

    // Initial count should be 0
    let res = query(deps.as_ref(), QueryMsg::MessageCount {}).unwrap();
    let count_response: echo_contract::msg::MessageCountResponse = from_json(&res).unwrap();
    assert_eq!(0, count_response.count);

    // Send a message
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("user"), &coins(100, "earth"));
    let message = Binary::from(b"Test");
    let msg = ExecuteMsg::Echo { message, attributes: vec![] };
    execute(deps.as_mut(), env, info, msg).unwrap();

    // Count should be 1
    let res = query(deps.as_ref(), QueryMsg::MessageCount {}).unwrap();
    let count_response: echo_contract::msg::MessageCountResponse = from_json(&res).unwrap();
    assert_eq!(1, count_response.count);
}

#[test]
fn test_echo_returns_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };
    instantiate(deps.as_mut(), info, msg).unwrap();

    // Test echo with funds
    let user_addr = deps.api.addr_make("user");
    let funds = coins(100, "earth");
    let info = cosmwasm_std::testing::message_info(&user_addr, &funds);
    let message = Binary::from(b"Hello World");
    let msg = ExecuteMsg::Echo { message: message.clone(), attributes: vec![] };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Check that bank send messages were added to return funds
    assert_eq!(1, res.messages.len());
    
    let bank_msg = &res.messages[0];
    match &bank_msg.msg {
        cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send { to_address, amount }) => {
            assert_eq!(user_addr.to_string(), *to_address);
            assert_eq!(&funds, amount);
        }
        _ => panic!("Expected BankMsg::Send"),
    }
    
    // Check that an event was emitted
    assert_eq!(1, res.events.len());
    let event = &res.events[0];
    assert_eq!("echo_message", event.ty);
    
    // Verify message count was incremented
    let count = get_message_count(deps.as_ref().storage);
    assert_eq!(1, count);
}

#[test]
fn test_echo_returns_multiple_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };
    instantiate(deps.as_mut(), info, msg).unwrap();

    // Test echo with multiple fund types
    let user_addr = deps.api.addr_make("user");
    let funds = vec![
        cosmwasm_std::Coin::new(100u128, "earth"),
        cosmwasm_std::Coin::new(50u128, "moon"),
        cosmwasm_std::Coin::new(200u128, "mars"),
    ];
    let info = cosmwasm_std::testing::message_info(&user_addr, &funds);
    let message = Binary::from(b"Multiple funds test");
    let msg = ExecuteMsg::Echo { message: message.clone(), attributes: vec![] };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Check that bank send messages were added for each fund type
    assert_eq!(3, res.messages.len());
    
    // Verify each fund type is returned
    for (i, bank_msg) in res.messages.iter().enumerate() {
        match &bank_msg.msg {
            cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send { to_address, amount }) => {
                assert_eq!(user_addr.to_string(), *to_address);
                assert_eq!(1, amount.len());
                assert_eq!(&funds[i], &amount[0]);
            }
            _ => panic!("Expected BankMsg::Send"),
        }
    }
    
    // Verify message count was incremented
    let count = get_message_count(deps.as_ref().storage);
    assert_eq!(1, count);
}

#[test]
fn test_echo_no_funds() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    
    // Instantiate first
    let info = cosmwasm_std::testing::message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));
    let admin_addr = deps.api.addr_make("admin");
    let msg = InstantiateMsg {
        admin: admin_addr.to_string(),
    };
    instantiate(deps.as_mut(), info, msg).unwrap();

    // Test echo without funds
    let user_addr = deps.api.addr_make("user");
    let info = cosmwasm_std::testing::message_info(&user_addr, &[]);
    let message = Binary::from(b"No funds test");
    let msg = ExecuteMsg::Echo { message: message.clone(), attributes: vec![] };

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    
    // Check that no bank send messages were added
    assert_eq!(0, res.messages.len());
    
    // Check that an event was still emitted
    assert_eq!(1, res.events.len());
    let event = &res.events[0];
    assert_eq!("echo_message", event.ty);
    
    // Verify message count was incremented
    let count = get_message_count(deps.as_ref().storage);
    assert_eq!(1, count);
}
