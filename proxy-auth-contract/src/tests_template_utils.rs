#[cfg(test)]
pub mod tests_template_utils {
    use cosmwasm_std::{
        testing::message_info,
        Addr, DepsMut, Env, Response,
    };
    use std::collections::HashSet;

    use crate::{
        contract::{execute, instantiate},
        msg::{ExecuteMsg, InstantiateMsg, TemplateMsg, ActionMsg},
        state::load_template,
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
        deps: DepsMut,
        env: Env,
        admin_user_address: Addr,
    ) -> Addr {
        let instantiate_msg_info = message_info(&admin_user_address, &[]);
        let instantiate_res = instantiate(
            deps,
            env.clone(),
            instantiate_msg_info,
            InstantiateMsg {
                approvers: HashSet::from([admin_user_address.clone()]),
            },
        );
        match instantiate_res {
            Ok(Response { .. }) => {}
            _ => panic!("Error instantiating contract"),
        }

        env.contract.address.clone()
    }

    pub fn create_test_template_msg(
        template_id: &str,
        publisher: Addr,
        private: bool,
    ) -> TemplateMsg {
        TemplateMsg {
            id: template_id.to_string(),
            publisher,
            actions: vec![
                ActionMsg {
                    id: "action1".to_string(),
                    message_template: "{\"swap\": {\"amount\": \"{{amount}}\", \"denom\": \"{{denom}}\"}}".to_string(),
                    contract_address: Addr::unchecked("contract1"),
                    allowed_denoms: HashSet::from(["ukuji".to_string(), "uluna".to_string()]),
                },
                ActionMsg {
                    id: "action2".to_string(),
                    message_template: "{\"transfer\": {\"to\": \"{{recipient}}\", \"amount\": \"{{amount}}\"}}".to_string(),
                    contract_address: Addr::unchecked("contract2"),
                    allowed_denoms: HashSet::from(["ukuji".to_string()]),
                },
            ],
            private,
        }
    }

    pub fn publish_template(
        deps: DepsMut,
        env: Env,
        publisher: Addr,
        template_msg: TemplateMsg,
    ) -> Result<Response, crate::error::ContractError> {
        let msg_info = message_info(&publisher, &[]);
        let msg = ExecuteMsg::RequestForApproval { template: template_msg };
        execute(deps, env, msg_info, msg)
    }

    pub fn approve_template(
        deps: DepsMut,
        env: Env,
        approver: Addr,
        template_id: String,
    ) -> Result<Response, crate::error::ContractError> {
        let msg_info = message_info(&approver, &[]);
        let msg = ExecuteMsg::ApproveTemplate { template_id };
        execute(deps, env, msg_info, msg)
    }

    pub fn reject_template(
        deps: DepsMut,
        env: Env,
        rejecter: Addr,
        template_id: String,
    ) -> Result<Response, crate::error::ContractError> {
        let msg_info = message_info(&rejecter, &[]);
        let msg = ExecuteMsg::RejectTemplate { template_id };
        execute(deps, env, msg_info, msg)
    }

    pub fn assert_template_state(
        deps: &DepsMut,
        template_id: &str,
        expected_approved: bool,
        expected_publisher: &str,
        expected_private: bool,
    ) {
        let template = load_template(deps.storage, template_id).unwrap();
        assert_eq!(template.approved, expected_approved);
        assert_eq!(template.publisher.to_string(), expected_publisher);
        assert_eq!(template.private, expected_private);
        assert_eq!(template.id, template_id);
    }

    pub fn assert_template_not_exists(deps: &DepsMut, template_id: &str) {
        let result = load_template(deps.storage, template_id);
        assert!(result.is_err());
    }
} 