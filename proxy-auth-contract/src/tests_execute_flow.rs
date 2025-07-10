#[cfg(test)]
mod tests {
    use crate::contract::execute;
    use crate::error::ContractError;
    use crate::msg::ExecuteMsg;
    use crate::state::{query_bids_by_owner, save_post_bid, PostBid, PostBidState};
    use crate::tests_utils::tests_utils::{
        assert_attr_eq, create_bid_and_simulate_orca_callback, retract_bid_and_simulate_orca_callback
    };
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{BankMsg, CosmosMsg, Uint128};

    #[derive(PartialEq, Eq)]
    enum RetractScenarios {
        RetractFullSendParameter,
        RetractFullDontSendParameter,
        RetractPartially,
        RetractFullWithPendingClaims,
        RetractFullWithClaimedClaims,
    }

    fn test_retract_bid_happy_paths(scenario: RetractScenarios) {
        let mut deps = mock_dependencies();
        let api = deps.api;

        // let bid_amount = Uint128::new(1000);
        let max_amount = Uint128::new(1000);

        let (retract_bid_amount_parameter, create_bid_amount_sent): (Option<Uint128>, Uint128) =
            match scenario {
                RetractScenarios::RetractFullSendParameter => (Some(max_amount), max_amount),
                RetractScenarios::RetractFullDontSendParameter => (None, max_amount),
                RetractScenarios::RetractPartially => (Some(Uint128::new(600)), max_amount),
                RetractScenarios::RetractFullWithPendingClaims => (None, max_amount),
                RetractScenarios::RetractFullWithClaimedClaims => (None, max_amount),
            };

        let real_amount_retracted: Uint128 = match retract_bid_amount_parameter {
            Some(value) => value,
            None => max_amount,
        };

        let user_address = api.addr_make("user");
        let orca_contract_address = api.addr_make("orca_contract");

        // let bid_amount = Uint128::new(1000);
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address.to_string(),
            "ukuji",                       // denom
            create_bid_amount_sent.u128(), // amount
            &orca_contract_address.to_string(),
            Uint128::new(97),
            1,
			"Swap",
        )
        .unwrap();

        // if bid has pending claims add them before retract
        if scenario == RetractScenarios::RetractFullWithClaimedClaims
            || scenario == RetractScenarios::RetractFullWithPendingClaims
        {
            let user_bids = query_bids_by_owner(&deps.storage, user_address.clone()).unwrap();
            let bid_pos = 0;

            let post_bid_state = match scenario {
                RetractScenarios::RetractFullWithClaimedClaims => PostBidState::Finished,
                RetractScenarios::RetractFullWithPendingClaims => PostBidState::Active,
                _ => panic!("this can't never happen"),
            };

            save_post_bid(&mut deps.storage, &PostBid {
				id: Uint128::from(1u128),
				bid_id: user_bids[bid_pos].id.clone(),
                state: PostBidState::Finished,
                fin_contract_address: api.addr_make("fin_contract_1"),
                fin_order_idx: Uint128::new(1),
                original_offer_amount: Uint128::new(500),
                current_offer_amount: Uint128::new(500),
            }).unwrap();

            save_post_bid(&mut deps.storage, &PostBid {
				id: Uint128::from(2u128),
				bid_id: user_bids[bid_pos].id.clone(),
                state: post_bid_state,
                fin_contract_address: api.addr_make("fin_contract_2"),
                fin_order_idx: Uint128::new(2),
                original_offer_amount: Uint128::new(500),
                current_offer_amount: Uint128::new(500),
            }).unwrap();
        }

        let (retract_rsp, orca_reply_rsp) = retract_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &user_address,
            retract_bid_amount_parameter.clone(),
            real_amount_retracted,
			Uint128::new(1)
        )
        .unwrap();

        // verify attributes
        assert_eq!(retract_rsp.attributes.len(), 0);

        // verify event
        let event = &retract_rsp.events[0];
        assert_eq!(event.ty, "autorujira.autobidder");
        assert_eq!(event.attributes.len(), 6);
        assert_attr_eq(&event.attributes[0], "action", "retract_bid");
        assert_attr_eq(&event.attributes[1], "sender", &user_address.to_string());
        assert_attr_eq(&event.attributes[2], "bid_id", "1"); // bid ID
        assert_attr_eq(
            &event.attributes[3],
            "orca_contract_address",
            &orca_contract_address.to_string(),
        );
        assert_attr_eq(&event.attributes[4], "orca_bid_idx", "97"); // orca_bid_idx
        assert_attr_eq(
            &event.attributes[5],
            "requested_amount",
            real_amount_retracted.to_string().as_str(),
        ); // retracted amount

        // Verify bid still exists using query_bids_by_user
        let query_msg: crate::msg::QueryMsg = crate::msg::QueryMsg::GetBidsByUser {
            wallet_address: user_address.to_string(),
        };
        let query_res = crate::contract::query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let bids_from_query_rsp: crate::msg::UserBidsResponse =
            cosmwasm_std::from_json(&query_res).unwrap();
        //  let user_state = USER_BIDS.may_load(&deps.storage, &user_address).unwrap();
        let bids_from_state = query_bids_by_owner(&deps.storage, user_address.clone()).unwrap();

        match scenario {
            // bid is deleted from state
            RetractScenarios::RetractFullWithClaimedClaims
            | RetractScenarios::RetractFullSendParameter
            | RetractScenarios::RetractFullDontSendParameter => {
                assert!(bids_from_state.is_empty());
            }
            // bid is present with amount = prev.amount - retracted amount
            RetractScenarios::RetractPartially => {
                // let bids = user_state;
                assert_eq!(bids_from_state.len(), 1);
                let remaining = create_bid_amount_sent - real_amount_retracted;
                assert_eq!(bids_from_query_rsp.bids[0].bid.current_amount, remaining);
                // Remaining amount should be zero
            }
            // bid is present with amount = 0, but has pending claims
            RetractScenarios::RetractFullWithPendingClaims => {
                assert_eq!(bids_from_state.len(), 1);
                // The bid should still exist because it has unclaimed PostBids
                assert_eq!(bids_from_query_rsp.bids.len(), 1);
                assert_eq!(bids_from_query_rsp.bids[0].bid.current_amount, Uint128::zero());
                // Remaining amount should be zero
            }
        };
        // verify funds sent to sender
        // TODO: verify attrs from orca_reply_rsp
        let msgs = orca_reply_rsp.messages;
        assert_eq!(msgs.len(), 1);
        if let CosmosMsg::Bank(BankMsg::Send { to_address, amount }) = &msgs[0].msg {
            assert_eq!(to_address, &user_address.to_string());
            assert_eq!(amount[0].denom, "ukuji");
            assert_eq!(amount[0].amount, real_amount_retracted);
        } else {
            panic!("Expected BankMsg::Send message");
        }
    }

    #[test]
    fn test_retract_bid_not_found() {
        let mut deps = mock_dependencies();
        let api = deps.api;

        let creator_address = api.addr_make("creator");

        let info = message_info(&creator_address, &[]);

        let msg = ExecuteMsg::RetractBid {
            id: Uint128::new(1845),
            amount: None,
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg);

        match res {
            Err(ContractError::BidNotFound { .. }) => {}
            _ => panic!("Expected BidNotFound error"),
        }
    }

    #[test]
    fn test_retract_bid_insufficient_balance() {
        let mut deps = mock_dependencies();
        let api = deps.api;

        let creator_address = api.addr_make("creator");
        let orca_contract_address = api.addr_make("orca_contract");

        // Reutilizamos la función para crear el bid y simular el callback
        let _ = create_bid_and_simulate_orca_callback(
            deps.as_mut(),
            mock_env(),
            &creator_address.to_string(),
            "ukuji", // denom
            1000,    // amount
            &orca_contract_address.to_string(),
            Uint128::new(97),
            1,
			"Swap",
        )
        .unwrap();

        // Intentar retractar más de lo que tiene el bid
        let retract_info = message_info(&creator_address, &[]);
        let retract_msg = ExecuteMsg::RetractBid {
            id: Uint128::new(1),
            amount: Some(Uint128::new(2000)), // Intentar retractar 2000 ukuji, que es más de lo que tiene el bid
        };

        let res = execute(deps.as_mut(), mock_env(), retract_info, retract_msg);

        match res {
            Err(ContractError::BidHasNoBalance { .. }) => {}
            _ => panic!("Expected BidHasNoBalance error"),
        }
    }

    #[test]
    fn test_retract_bid_partial() {
        test_retract_bid_happy_paths(RetractScenarios::RetractPartially);
    }

    #[test]
    fn test_retract_full_bid_with_parameter() {
        test_retract_bid_happy_paths(RetractScenarios::RetractFullSendParameter);
    }

    #[test]
    fn test_retract_full_bid_with_no_parameter() {
        test_retract_bid_happy_paths(RetractScenarios::RetractFullDontSendParameter);
    }

    #[test]
    fn test_retract_bid_with_claimed_post_bids() {
        test_retract_bid_happy_paths(RetractScenarios::RetractFullWithClaimedClaims);
    }

    #[test]
    fn test_retract_bid_with_unclaimed_post_bids() {
        test_retract_bid_happy_paths(RetractScenarios::RetractFullWithPendingClaims);
    }
}
