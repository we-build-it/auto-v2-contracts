#[cfg(test)]
mod tests {
    use crate::contract::{execute, reply};
    use crate::core::reply_constants::ORCA_REPLY_ACTIVATE_BIDS;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg;
    use crate::orca_msg::OrcaExecuteMsg;
    use crate::state::{query_bids_by_owner, BidState};
    use crate::tests_orca_mocks::tests_orca_mocks::create_orca_activate_bids_event;
    use crate::tests_utils::tests_utils::{
        assert_attr_eq, create_bid_and_simulate_orca_callback, instantiate_contract,
    };
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_json, Binary, CosmosMsg, Reply, Uint128, WasmMsg};

    #[test]
    fn test_activate_bids_executed_by_not_admin() {
        let mut deps = mock_dependencies();
        let api = deps.api;

        let admin_user_address = api.addr_make("admin-address");
        instantiate_contract(deps.as_mut(), mock_env(), admin_user_address.clone());

        let another_address = api.addr_make("another-address");
        let orca_contract_address = api.addr_make("orca_contract");

        let activate_bids_msg_info = message_info(&another_address, &[]);

        let msg = ExecuteMsg::AutoActivateBids {
            orca_contract_address: orca_contract_address.to_string(),
        };

        let res = execute(deps.as_mut(), mock_env(), activate_bids_msg_info, msg);

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_activate_bids_contract_has_no_bids() {
        let mut deps = mock_dependencies();
        let api = deps.api;

        let admin_user_address = api.addr_make("admin-address");
        instantiate_contract(deps.as_mut(), mock_env(), admin_user_address.clone());

        let orca_contract_address = api.addr_make("orca_contract");

        let activate_bids_msg_info = message_info(&admin_user_address, &[]);

        let msg = ExecuteMsg::AutoActivateBids {
            orca_contract_address: orca_contract_address.to_string(),
        };

        let res = execute(deps.as_mut(), mock_env(), activate_bids_msg_info, msg);

        match res {
            Err(ContractError::NoBidsForAddress(_)) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_activate_bids_send_funds() {
        let mut deps = mock_dependencies();
        let api = deps.api;

        let admin_user_address = api.addr_make("admin-address");
        instantiate_contract(deps.as_mut(), mock_env(), admin_user_address.clone());

        let orca_contract_address = api.addr_make("orca_contract");

        let activate_bids_msg_info = message_info(&admin_user_address, &coins(1000, "uusd"));

        let msg = ExecuteMsg::AutoActivateBids {
            orca_contract_address: orca_contract_address.to_string(),
        };

        let res = execute(deps.as_mut(), mock_env(), activate_bids_msg_info, msg);

        match res {
            Err(ContractError::InvalidFundsReceived {}) => {}
            _ => panic!("Expected Unauthorized error"),
        }
    }

    #[test]
    fn test_activate_existing_bids() {
        let mut deps = mock_dependencies();
        let api = deps.api;

        let admin_user_address = api.addr_make("admin-address");
        instantiate_contract(deps.as_mut(), mock_env(), admin_user_address.clone());

        let user_address_1 = api.addr_make("user-1");
        let user_address_2 = api.addr_make("user-2");
        let orca_contract_address_1 = api.addr_make("orca_contract-1");
        let orca_contract_address_2 = api.addr_make("orca_contract-2");

        // --- put bids in different contracts with different users
        // user 1 puts 2 bids on contract-1 and 1 bid on contract-2
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address_1.to_string(),
            "ukuji",  // denom
            1000u128, // amount
            &orca_contract_address_1.to_string(),
            Uint128::new(1),
            10u16,
			"Swap",
        )
        .unwrap();
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address_1.to_string(),
            "ukuji",  // denom
            2000u128, // amount
            &orca_contract_address_1.to_string(),
            Uint128::new(2),
            10u16,
			"Swap",
        )
        .unwrap();
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address_1.to_string(),
            "ukuji",  // denom
            3000u128, // amount
            &orca_contract_address_2.to_string(),
            Uint128::new(3),
            10u16,
			"Swap",
        )
        .unwrap();
        // user 2 puts 1 bid on each contract
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address_2.to_string(),
            "ukuji",  // denom
            4000u128, // amount
            &orca_contract_address_1.to_string(),
            Uint128::new(4),
            10u16,
			"Swap",
        )
        .unwrap();
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address_2.to_string(),
            "ukuji",  // denom
            5000u128, // amount
            &orca_contract_address_2.to_string(),
            Uint128::new(5),
            10u16,
			"Swap",
        )
        .unwrap();

        // --- execute activate
        let activate_bids_msg_info = message_info(&admin_user_address, &[]);
        let msg = ExecuteMsg::AutoActivateBids {
            orca_contract_address: orca_contract_address_1.to_string(),
        };
        let activate_bids_rsp =
            execute(deps.as_mut(), mock_env(), activate_bids_msg_info, msg).unwrap();

        // --- verify attributes/events for execute(...)
        assert_eq!(activate_bids_rsp.attributes.len(), 0);
        let event = &activate_bids_rsp.events[0];
        assert_eq!(event.ty, "autorujira.autobidder");
        assert_eq!(event.attributes.len(), 3);
        assert_attr_eq(&event.attributes[0], "action", "activate_bids");
        assert_attr_eq(
            &event.attributes[1],
            "sender",
            &admin_user_address.to_string(),
        );
        // assert_attr_eq(&event.attributes[2], "orca_bids_idx", "1"); // bid ID
        assert_attr_eq(
            &event.attributes[2],
            "orca_contract_address",
            &orca_contract_address_1.to_string(),
        );

        // --- check invocation to orca contract
        let sub_msg: &CosmosMsg = &activate_bids_rsp.messages[0].msg;
        match sub_msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr, &orca_contract_address_1.to_string());
                assert_eq!(funds.len(), 0);

                let received_msg: OrcaExecuteMsg = from_json(msg).unwrap();

                match received_msg {
                    OrcaExecuteMsg::ActivateBids { bids_idx } => {
                        assert_eq!(bids_idx, None);
                        // assert_eq!(bids_idx[0], Uint128::new(1));
                    }
                    _ => panic!("Unexpected message variant"),
                }
            }
            _ => panic!("Unexpected message type"),
        }

        // --- simulate reply
        let reply_msg = Reply {
            id: ORCA_REPLY_ACTIVATE_BIDS,
            gas_used: 0,
            payload: Binary::from(b"some_binary_data"),
            result: create_orca_activate_bids_event(),
        };
        let reply_rsp = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();
        let reply_rsp_event = reply_rsp
            .events
            .iter()
            .find(|e| e.ty == "autorujira.autobidder")
            .expect("Event not found");
        assert_eq!(reply_rsp_event.attributes.len(), 1);
        assert_eq!(reply_rsp_event.attributes[0].key, "action");
        assert_eq!(reply_rsp_event.attributes[0].value, "activate_orca_bids");

        // --- check if user-1 bids are with the correct state
        let query_msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetBidsByUser {
            wallet_address: user_address_1.to_string(),
        };
        let query_res = crate::contract::query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let bids_from_query_rsp: crate::msg::UserBidsResponse =
            cosmwasm_std::from_json(&query_res).unwrap();
        let bids_from_state = query_bids_by_owner(&deps.storage, user_address_1.clone()).unwrap();

        assert_eq!(bids_from_state.len(), 3);
        assert_eq!(bids_from_query_rsp.bids.len(), 3);

        assert_eq!(bids_from_state[0].state, BidState::Active);
        assert_eq!(bids_from_state[1].state, BidState::Active);
        assert_eq!(bids_from_state[2].state, BidState::Active);

        assert_eq!(bids_from_query_rsp.bids[0].bid.state, BidState::Active);
        assert_eq!(bids_from_query_rsp.bids[1].bid.state, BidState::Active);
        assert_eq!(bids_from_query_rsp.bids[2].bid.state, BidState::Active);

        // --- check if user-2 bids are with the correct state
        let query_msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetBidsByUser {
            wallet_address: user_address_2.to_string(),
        };
        let query_res = crate::contract::query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let bids_from_query_rsp: crate::msg::UserBidsResponse =
            cosmwasm_std::from_json(&query_res).unwrap();
        let bids_from_state = query_bids_by_owner(&deps.storage, user_address_2.clone()).unwrap();

        assert_eq!(bids_from_state.len(), 2);
        assert_eq!(bids_from_query_rsp.bids.len(), 2);

        assert_eq!(bids_from_state[0].state, BidState::Active);
        assert_eq!(bids_from_state[1].state, BidState::Active);

        assert_eq!(bids_from_query_rsp.bids[0].bid.state, BidState::Active);
        assert_eq!(bids_from_query_rsp.bids[1].bid.state, BidState::Active);
    }
}
