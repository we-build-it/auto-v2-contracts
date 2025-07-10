#[cfg(test)]

pub mod tests_utils {
    use std::str::FromStr;

    use cosmwasm_std::{
        from_json, testing::message_info, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal256, Deps,
        DepsMut, Env, MessageInfo, Reply, Response, StdError, StdResult, Uint128, WasmMsg,
    };

    use crate::{
        contract::{execute, instantiate, query, reply},
        core::reply_constants::{
            FIN_REPLY_SUBMIT_ORDER_MIN, FIN_REPLY_WITHDRAW_ORDER_MIN,
            ORCA_REPLY_CLAIM_LIQUIDATIONS_MIN, ORCA_REPLY_CREATE_BID, ORCA_REPLY_RETRACT_BID,
        },
        msg::{
            BalanceResponse, BidConfigInput, BidToClaim, ExecuteMsg, InstantiateMsg,
            PostBidToActivate, PostBidToClaim, QueryMsg, UserBidsResponse,
        },
        orca_msg::OrcaExecuteMsg,
        state::PostBidState,
        tests_orca_mocks::tests_orca_mocks::{
            create_fin_submit_order_event, create_fin_withdraw_order_event,
            create_orca_claim_bids_event, create_orca_retract_bid_event,
            create_orca_submit_bid_event_with_params,
        },
    };

    pub fn assert_attr_eq(
        attr: &cosmwasm_std::Attribute,
        expected_key: &str,
        expected_value: &str,
    ) {
        assert_eq!(
			attr.key, expected_key,
			"Assertion failed in assert_attr_eq for key: expected '{}', got '{}'. Called from: {}:{}",
			expected_key, attr.key, file!(), line!()
		);
        assert_eq!(
			attr.value, expected_value,
			"Assertion failed in assert_attr_eq for value: expected '{}', got '{}'. Called from: {}:{}",
			expected_value, attr.value, file!(), line!()
		);
    }

    pub fn instantiate_contract(
        mut deps: DepsMut, // Recibe DepsMut directamente (no como referencia mutable)
        env: Env,
        admin_user_address: Addr,
    ) -> Addr {
        let instantiante_msg_info = message_info(&admin_user_address, &[]);
        let instantiate_res = instantiate(
            deps.branch(),
            env.clone(),
            instantiante_msg_info,
            InstantiateMsg {
                admin: Some(admin_user_address.clone()),
            },
        );
        match instantiate_res {
            Ok(Response { .. }) => {}
            _ => panic!("Error instantiating contract"),
        }

        env.contract.address.clone()
    }

    pub fn create_bid_and_simulate_orca_callback(
        mut deps: DepsMut, // Recibe DepsMut directamente (no como referencia mutable)
        env: Env,
        bid_owner: &str,
        denom: &str,
        amount: u128,
        orca_contract_address: &str,
        orca_idx: Uint128,
        orca_queue_position: u16,
        action: &str,
    ) -> StdResult<(Response, Response)> {
        // create envelope
        let create_info = MessageInfo {
            sender: deps.api.addr_validate(bid_owner)?,
            funds: vec![Coin {
                denom: denom.to_string(),
                amount: Uint128::new(amount),
            }],
        };

        // create bid message
        let create_msg = ExecuteMsg::CreateBid {
            orca_contract_address: orca_contract_address.to_string(),
            orca_queue_position,
            config: BidConfigInput {
                action: action.to_string(),
                profit_percentage: Some(10),
                amount_percentage_to_swap: Some(5),
            },
            collateral_denom: "ukuji".to_string(),
        };

        // execute create bid
        let autobidder_create_bid_rsp: Response =
            execute(deps.branch(), env.clone(), create_info, create_msg)
                .map_err(|e| StdError::generic_err(format!("Error in execute: {:?}", e)))?;

        // verify message sent to Orca
        let sub_msg: &CosmosMsg = &autobidder_create_bid_rsp.messages[0].msg;
        match sub_msg {
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg,
                funds,
            }) => {
                assert_eq!(contract_addr, &orca_contract_address.to_string());
                assert_eq!(funds.len(), 1);
                assert_eq!(funds[0].amount, &Uint128::new(amount));

                let received_msg: OrcaExecuteMsg = from_json(msg).unwrap();

                match received_msg {
                    OrcaExecuteMsg::SubmitBid { premium_slot } => {
                        assert_eq!(premium_slot, orca_queue_position.clone());
                    }
                    _ => panic!("Unexpected message variant"),
                }
            }
            _ => panic!("Unexpected message type"),
        }

        // verify some create attributes
        assert_eq!(autobidder_create_bid_rsp.attributes.len(), 0);

        // simulate call back
        let reply_msg = Reply {
            id: ORCA_REPLY_CREATE_BID,                  // submessage id
            gas_used: 0,                                // total gas
            payload: Binary::from(b"some_binary_data"), // randon binary data
            result: create_orca_submit_bid_event_with_params(
                &orca_idx.to_string(),
                &Uint128::from(amount).to_string(),
            ), // orca submit bid event mock
        };

        // execute reply
        let orca_reply_rsp = reply(deps.branch(), env, reply_msg)
            .map_err(|e| StdError::generic_err(format!("Error in reply: {:?}", e)))?;

        // return both responses
        Ok((autobidder_create_bid_rsp, orca_reply_rsp))
    }

    pub fn retract_bid_and_simulate_orca_callback(
        mut deps: DepsMut,
        env: Env,
        final_user_addr: &Addr,
        requested_amount: Option<Uint128>,
        retracted_amount: Uint128,
        bid_id: Uint128,
    ) -> StdResult<(Response, Response)> {
        // create envelope
        let retract_info = message_info(final_user_addr, &[]);
        let retract_msg = ExecuteMsg::RetractBid {
            id: bid_id,
            amount: requested_amount,
        };
        let retract_rsp = execute(deps.branch(), env.clone(), retract_info, retract_msg).unwrap();

        // simulate call back
        let reply_msg = Reply {
            id: ORCA_REPLY_RETRACT_BID,
            gas_used: 0,
            payload: Binary::from(b"some_binary_data"),
            result: create_orca_retract_bid_event(retracted_amount.to_string()),
        };

        // execute reply
        let orca_reply_rsp = reply(deps.branch(), env.clone(), reply_msg)
            .map_err(|e| StdError::generic_err(format!("Error in reply: {:?}", e)))?;

        // return both responses
        Ok((retract_rsp, orca_reply_rsp))
    }

    pub fn verify_bank_message(
        msg: &CosmosMsg,
        expected_to_address: &str,
        expected_denom: &str,
        expected_amount: Uint128,
    ) {
        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &msg {
            assert_eq!(to_address, &expected_to_address);
            assert_eq!(amount[0].denom, expected_denom.to_string());
            assert_eq!(amount[0].amount, expected_amount.clone());
        } else {
            panic!("Expected BankMsg::Send message");
        }
    }

    pub fn claim_orca_and_simulate_orca_callback(
        mut deps: DepsMut,
        env: Env,
        user_address: &Addr,
        admin_user_address: &Addr,
        bid_id: Uint128,
        post_bid_id: Uint128,
        collateral_claimed: Uint128,
        remaining_bid_amount: Uint128,
        fin_contract_address: String,
        orca_idx: String,
        orca_contract_address: String,
        action: String,
    ) -> StdResult<(Response, Response)> {
        let claim_msg_info = message_info(&admin_user_address, &[]);
        let bid_to_claim: BidToClaim = BidToClaim {
            bid_id,
            current_orca_amount: remaining_bid_amount.clone(),
            fin_contract_address: Some(fin_contract_address.to_string()),
        };

        let claim_msg = ExecuteMsg::AutoClaimBids {
            bids: vec![bid_to_claim],
        };

        let claim_rsp = execute(deps.branch(), env.clone(), claim_msg_info, claim_msg).unwrap();

        assert_eq!(claim_rsp.events.len(), 1);
        let event = &claim_rsp.events[0];
        assert_eq!(event.ty, "autorujira.autobidder");
        assert_eq!(event.attributes.len(), 2);
        assert_attr_eq(&event.attributes[0], "action", "claim_bids");
        assert_attr_eq(
            &event.attributes[1],
            "sender",
            &admin_user_address.to_string(),
        );

        // simulate reply from ORCA contract
        let reply_msg = Reply {
            id: ORCA_REPLY_CLAIM_LIQUIDATIONS_MIN,
            gas_used: 0,
            payload: Binary::from(b"some_binary_data"),
            result: create_orca_claim_bids_event(
                collateral_claimed.to_string().as_str(),
                remaining_bid_amount.clone(),
            ),
        };
        let auto_claim_orca_reply = reply(deps.branch(), env.clone(), reply_msg).unwrap();
        assert_eq!(auto_claim_orca_reply.events.len(), 1);
        let event = &auto_claim_orca_reply.events[0];
        assert_eq!(event.ty, "autorujira.autobidder");
        assert_eq!(event.attributes.len(), 11);
        assert_attr_eq(&event.attributes[0], "action", "claim_orca_bid");
        assert_attr_eq(&event.attributes[1], "bid_id", bid_id.to_string().as_str());
        assert_attr_eq(
            &event.attributes[2],
            "orca_bid_idx",
            orca_idx.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[3],
            "orca_contract_address",
            &orca_contract_address.to_string(),
        );
        assert_attr_eq(
            &event.attributes[4],
            "amount",
            collateral_claimed.to_string().as_str(),
        );
        assert_attr_eq(&event.attributes[5], "denom", "ukuji");
        assert_attr_eq(&event.attributes[6], "owner", &user_address.to_string());
        assert_attr_eq(
            &event.attributes[7],
            "remaining_bid_amount",
            remaining_bid_amount.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[8],
            "post_action",
            action.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[9],
            "post_bid_id",
            post_bid_id.to_string().as_ref(),
        );
        assert_attr_eq(
            &event.attributes[10],
            "fin_contract_address",
            fin_contract_address.to_string().as_str(),
        );

        let res = query_user_bids(deps.as_ref(), env, user_address).unwrap();
        assert_eq!(res.wallet_address, user_address.to_string());
        // assert_eq!(res.bids.len(), 1);
        for bid in res.bids {
            // assert_eq!(bid.post_bids.len(), 1);
            if bid.bid.id == bid_id {
                assert_eq!(bid.bid.id, bid_id.clone());
                assert_eq!(bid.post_bids[0].id, post_bid_id.clone());
                assert_eq!(
                    bid.post_bids[0].fin_contract_address.to_string(),
                    fin_contract_address.clone()
                );
                assert_eq!(bid.post_bids[0].state, PostBidState::Inactive);
                assert_eq!(
                    bid.post_bids[0].original_offer_amount,
                    collateral_claimed.clone()
                );
                assert_eq!(bid.post_bids[0].fin_order_idx, Uint128::new(0));
            }
        }

        Ok((claim_rsp, auto_claim_orca_reply))
    }

    pub fn activate_post_bid_and_simulate_fin_callback(
        mut deps: DepsMut,
        env: Env,
        user_address: &Addr,
        admin_user_address: &Addr,
        bid_id: Uint128,
        post_bid_id: Uint128,
        fin_idx: String,
        fin_contract_address: String,
        price: String,
    ) -> StdResult<(Response, Response)> {
        // activate post bids
        let activate_post_bids_msg_info = message_info(&admin_user_address, &[]);

        let post_bid = PostBidToActivate {
            post_bid_id: post_bid_id.clone(),
            order_price: Some(Decimal256::from_str(price.as_str()).unwrap()),
        };

        let activate_msg = ExecuteMsg::AutoActivatePostBids {
            post_bids: vec![post_bid],
        };
        let activate_rsp = execute(
            deps.branch(),
            env.clone(),
            activate_post_bids_msg_info,
            activate_msg,
        )
        .unwrap();

        assert_eq!(activate_rsp.events.len(), 1);
        let activate_event = &activate_rsp.events[0];
        assert_eq!(activate_event.ty, "autorujira.autobidder");
        assert_eq!(activate_event.attributes.len(), 2);
        assert_attr_eq(
            &activate_event.attributes[0],
            "action",
            "activate_post_bids",
        );
        assert_attr_eq(
            &activate_event.attributes[1],
            "sender",
            &admin_user_address.to_string(),
        );

        // simulate reply from FIN contract
        let fin_reply_msg = Reply {
            id: FIN_REPLY_SUBMIT_ORDER_MIN,
            gas_used: 0,
            payload: Binary::from(b"some_binary_data"),
            result: create_fin_submit_order_event(fin_idx.to_string().as_str()),
        };
        let auto_activate_submit_order_fin_reply =
            reply(deps.branch(), env.clone(), fin_reply_msg).unwrap();
        let activate_event_reply = &auto_activate_submit_order_fin_reply.events[0];
        assert_eq!(activate_event_reply.ty, "autorujira.autobidder");
        assert_eq!(activate_event_reply.attributes.len(), 6);
        assert_attr_eq(
            &activate_event_reply.attributes[0],
            "action",
            "submit_fin_order",
        );
        assert_attr_eq(
            &activate_event_reply.attributes[1],
            "order_idx",
            fin_idx.to_string().as_str(),
        );
        assert_attr_eq(
            &activate_event_reply.attributes[2],
            "fin_contract_address",
            fin_contract_address.to_string().as_str(),
        );
        assert_attr_eq(
            &activate_event_reply.attributes[3],
            "bid_id",
            bid_id.to_string().as_str(),
        );
        assert_attr_eq(
            &activate_event_reply.attributes[4],
            "post_bid_id",
            post_bid_id.to_string().as_str(),
        );
        assert_attr_eq(&activate_event_reply.attributes[5], "price", price.as_str());

        let res: UserBidsResponse = query_user_bids(deps.as_ref(), env.clone(), user_address)?;
        assert_eq!(res.wallet_address, user_address.to_string());
        assert_eq!(res.bids.len(), 1);
        for bid in res.bids {
            // assert_eq!(bid.post_bids.len(), 1);
            if bid.bid.id == bid_id {
                assert_eq!(bid.bid.id, bid_id.clone());
                assert_eq!(bid.post_bids.len(), 1);
                assert_eq!(bid.post_bids[0].id, post_bid_id.clone());
                assert_eq!(
                    bid.post_bids[0].fin_contract_address.to_string(),
                    fin_contract_address.clone()
                );
                assert_eq!(bid.post_bids[0].state, PostBidState::Active);
                assert_eq!(
                    bid.post_bids[0].fin_order_idx.to_string(),
                    fin_idx.to_string()
                );
            }
        }

        Ok((activate_rsp, auto_activate_submit_order_fin_reply))
    }

    pub fn auto_claim_post_bid_and_simulate_fin_callback(
        mut deps: DepsMut,
        env: Env,
        user_address: &Addr,
        admin_user_address: &Addr,
        bid_id: Uint128,
        post_bid_id: Uint128,
        fin_idx: String,
        fin_contract_address: String,
        autobidder_contract_address: String,
        current_offer_amount: Uint128,
        withdraw_amount: Uint128,
    ) -> StdResult<(Response, Response)> {
        let auto_claim_post_bids_msg_info = message_info(&admin_user_address, &[]);
        let post_bid_to_claim = PostBidToClaim {
            current_offer_amount: current_offer_amount.clone(),
            post_bid_id: post_bid_id.clone(),
        };
        let auto_claim_post_bids_msg = ExecuteMsg::AutoClaimPostBids {
            post_bids: vec![post_bid_to_claim],
        };
        let auto_claim_post_bids_rsp = execute(
            deps.branch(),
            env.clone(),
            auto_claim_post_bids_msg_info,
            auto_claim_post_bids_msg,
        )
        .unwrap();

        assert_eq!(auto_claim_post_bids_rsp.events.len(), 1);
        let event = &auto_claim_post_bids_rsp.events[0];
        assert_eq!(event.ty, "autorujira.autobidder");
        assert_eq!(event.attributes.len(), 2);
        assert_attr_eq(&event.attributes[0], "action", "claim_post_bids");
        assert_attr_eq(
            &event.attributes[1],
            "sender",
            &admin_user_address.to_string(),
        );

        // simulate reply from FIN contract
        let fin_withdraw_reply_msg = Reply {
            id: FIN_REPLY_WITHDRAW_ORDER_MIN,
            gas_used: 0,
            payload: Binary::from(b"some_binary_data"),
            result: create_fin_withdraw_order_event(
                (withdraw_amount.to_string()
                    + "factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth")
                    .as_str(),
                autobidder_contract_address.to_string().as_str(),
            ),
        };
        let fin_withdraw_reply_rsp =
            reply(deps.branch(), env.clone(), fin_withdraw_reply_msg).unwrap();

        assert_eq!(fin_withdraw_reply_rsp.events.len(), 1);
        let event = &fin_withdraw_reply_rsp.events[0];
        assert_eq!(event.ty, "autorujira.autobidder");
        assert_eq!(event.attributes.len(), 7);
        assert_attr_eq(&event.attributes[0], "action", "withdraw_fin_order");
        assert_attr_eq(&event.attributes[1], "bid_id", bid_id.to_string().as_str());
        assert_attr_eq(
            &event.attributes[2],
            "post_bid_id",
            post_bid_id.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[3],
            "fin_order_idx",
            fin_idx.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[4],
            "fin_contract_address",
            fin_contract_address.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[5],
            "amount",
            withdraw_amount.to_string().as_str(),
        );
        assert_attr_eq(
            &event.attributes[6],
            "denom",
            "factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth",
        );

        let res: UserBidsResponse = query_user_bids(deps.as_ref(), env.clone(), user_address)?;
        assert_eq!(res.wallet_address, user_address.to_string());
        assert_eq!(res.bids.len(), 1);
        for bid in res.bids {
            // assert_eq!(bid.post_bids.len(), 1);
            if bid.bid.id == bid_id {
                assert_eq!(bid.bid.id, bid_id.clone());
                assert_eq!(bid.post_bids.len(), 1);
                assert_eq!(bid.post_bids[0].id, post_bid_id.clone());
                assert_eq!(bid.post_bids[0].state, PostBidState::Finished);
            }
        }

        Ok((auto_claim_post_bids_rsp, fin_withdraw_reply_rsp))
    }

    pub fn claim_funds(
        deps: DepsMut,
        env: Env,
        user_address: &Addr,
        bid_id: Uint128,
        post_bid_id: Uint128,
        withdraw_amount: Uint128,
    ) -> StdResult<Response> {
        let claim_funds_msg_info = message_info(&user_address, &[]);
        let claim_funds_msg = ExecuteMsg::ClaimFunds {};
        let claim_funds_rsp =
            execute(deps, env.clone(), claim_funds_msg_info, claim_funds_msg).unwrap();

        let claim_funds_rsp_to_return = claim_funds_rsp.clone();

        let msgs = claim_funds_rsp.messages;
        assert_eq!(msgs.len(), 1);
        verify_bank_message(
            &msgs[0].msg,
            user_address.to_string().as_str(),
            "factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth",
            withdraw_amount.clone(),
        );
        assert_eq!(claim_funds_rsp.events.len(), 2);

        let event1 = &claim_funds_rsp.events[0];
        assert_eq!(event1.ty, "autorujira.autobidder");
        assert_eq!(event1.attributes.len(), 2);
        assert_attr_eq(&event1.attributes[0], "action", "claim_funds");
        assert_attr_eq(
            &event1.attributes[1],
            "sender",
            user_address.to_string().as_str(),
        );

        let event2 = &claim_funds_rsp.events[1];
        assert_eq!(event2.ty, "autorujira.autobidder");
        assert_eq!(event2.attributes.len(), 5);

        assert_attr_eq(&event2.attributes[0], "action", "send_funds");
        assert_attr_eq(
            &event2.attributes[1],
            "denom",
            "factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth",
        );
        assert_attr_eq(
            &event2.attributes[2],
            "amount",
            withdraw_amount.to_string().as_str(),
        );
        assert_attr_eq(&event2.attributes[3], "bid_id", bid_id.to_string().as_str());
        assert_attr_eq(
            &event2.attributes[4],
            "post_bid_id",
            post_bid_id.to_string().as_str(),
        );

        Ok(claim_funds_rsp_to_return)
    }

    pub fn get_user_balances(
        deps: Deps,
        env: Env,
        user_address: &Addr,
    ) -> StdResult<BalanceResponse> {
        let query_balance_msg = QueryMsg::GetBalance {
            wallet_address: user_address.to_string(),
        };
        let query_balance_res = query(deps, env.clone(), query_balance_msg).unwrap();
        let balance_res: BalanceResponse = from_json(&query_balance_res).unwrap();

        Ok(balance_res)
    }

    pub fn query_user_bids(
        deps: Deps,
        env: Env,
        user_address: &Addr,
    ) -> StdResult<UserBidsResponse> {
        let query_bids_msg = QueryMsg::GetBidsByUser {
            wallet_address: user_address.to_string(),
        };
        let query_res = query(deps, env.clone(), query_bids_msg).unwrap();
        let res: UserBidsResponse = from_json(&query_res).unwrap();
        Ok(res)
    }
}
