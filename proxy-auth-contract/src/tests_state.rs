#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::mock_dependencies,
        Addr, Uint128,
    };

    use crate::{state::{find_bid_and_validate_owner, save_bid, Bid, BidConfig, BidState, PostAction}, ContractError};

    #[test]
    fn test_find_and_validate_bid_success() {
        let mut deps = mock_dependencies();

        // Crear un Bid de ejemplo
        let bid = Bid {
            id: Uint128::new(1),
            owner: Addr::unchecked("owner1"),
            state: BidState::Active,
            orca_contract_address: Addr::unchecked("orca1"),
            orca_bid_idx: Uint128::new(100),
            orca_queue_position: 10,
            current_amount: Uint128::new(1000),
            bid_denom: "uatom".to_string(),
            config: BidConfig {
                action: PostAction::None,
                profit_percentage: Some(5000),
                amount_percentage_to_swap: Some(2500),
            },
			original_amount: Uint128::new(1000),
			collateral_denom: "ukuji".to_string(),
        };

        // Guardar el Bid
        save_bid(&mut deps.storage, &bid).unwrap();

        // Validar el Bid con el owner correcto
        let result = find_bid_and_validate_owner(
            &deps.storage,
            Uint128::new(1),
            Addr::unchecked("owner1"),
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), bid);
    }

    #[test]
    fn test_find_and_validate_bid_wrong_owner() {
        let mut deps = mock_dependencies();

        // Crear un Bid de ejemplo
        let bid = Bid {
            id: Uint128::new(2),
            owner: Addr::unchecked("owner2"),
            state: BidState::Active,
            orca_contract_address: Addr::unchecked("orca2"),
            orca_bid_idx: Uint128::new(200),
            orca_queue_position: 20,
            current_amount: Uint128::new(2000),
            bid_denom: "uatom".to_string(),
            config: BidConfig {
                action: PostAction::Swap,
                profit_percentage: Some(6000),
                amount_percentage_to_swap: Some(3000),
            },
			original_amount: Uint128::new(2000),
			collateral_denom: "ukuji".to_string(),
        };

        // Guardar el Bid
        save_bid(&mut deps.storage, &bid).unwrap();

        // Intentar validar el Bid con un owner incorrecto
        let result = find_bid_and_validate_owner(
            &deps.storage,
            Uint128::new(2),
            Addr::unchecked("wrong_owner"),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ContractError::BidNotFound { .. } => {}
            _ => panic!("Expected BidNotFound error"),
        }
    }

    #[test]
    fn test_find_and_validate_bid_not_found() {
        let deps = mock_dependencies();

        // Intentar validar un Bid que no existe
        let result = find_bid_and_validate_owner(
            &deps.storage,
			Uint128::new(200),
            Addr::unchecked("owner1"),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            ContractError::BidNotFound { .. } => {}
            _ => panic!("Expected BidNotFound error"),
        }
    }

}
