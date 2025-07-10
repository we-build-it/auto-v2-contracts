#[cfg(test)]
mod tests {
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env},
    };

    use crate::tests_template_utils::tests_template_utils::{
        approve_template, assert_template_not_exists, assert_template_state, create_test_template_msg,
        instantiate_contract, publish_template, reject_template,
    };

    #[test]
    fn test_publish_template_and_reject() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let api = deps.api;

        // Setup contract with admin
        let admin_address = api.addr_make("admin");
        instantiate_contract(deps.as_mut(), env.clone(), admin_address.clone());

        // Create publisher
        let publisher_address = api.addr_make("publisher");

        // Create template message
        let template_msg = create_test_template_msg("template1", publisher_address.clone(), false);

        // Publish template
        let response = publish_template(
            deps.as_mut(),
            env.clone(),
            publisher_address.clone(),
            template_msg,
        )
        .unwrap();

        // Verify response attributes
        assert_eq!(response.attributes.len(), 3);
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &response.attributes[0],
            "method",
            "request_for_approval",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &response.attributes[1],
            "template_id",
            "template1",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &response.attributes[2],
            "publisher",
            &publisher_address.to_string(),
        );

        // Verify template state: approved=false, correct publisher, correct private flag
        assert_template_state(
            &deps.as_mut(),
            "template1",
            false, // approved should be false
            &publisher_address.to_string(),
            false, // private should be false
        );

        // Reject template
        let reject_response = reject_template(
            deps.as_mut(),
            env.clone(),
            admin_address.clone(),
            "template1".to_string(),
        )
        .unwrap();

        // Verify reject response attributes
        assert_eq!(reject_response.attributes.len(), 3);
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &reject_response.attributes[0],
            "method",
            "reject_template",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &reject_response.attributes[1],
            "template_id",
            "template1",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &reject_response.attributes[2],
            "rejecter",
            &admin_address.to_string(),
        );

        // Verify template has been deleted
        assert_template_not_exists(&deps.as_mut(), "template1");
    }

    #[test]
    fn test_publish_template_and_approve() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let api = deps.api;

        // Setup contract with admin
        let admin_address = api.addr_make("admin");
        instantiate_contract(deps.as_mut(), env.clone(), admin_address.clone());

        // Create publisher
        let publisher_address = api.addr_make("publisher");

        // Create template message
        let template_msg = create_test_template_msg("template2", publisher_address.clone(), true);

        // Publish template
        let response = publish_template(
            deps.as_mut(),
            env.clone(),
            publisher_address.clone(),
            template_msg,
        )
        .unwrap();

        // Verify response attributes
        assert_eq!(response.attributes.len(), 3);
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &response.attributes[0],
            "method",
            "request_for_approval",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &response.attributes[1],
            "template_id",
            "template2",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &response.attributes[2],
            "publisher",
            &publisher_address.to_string(),
        );

        // Verify template state: approved=false, correct publisher, correct private flag
        assert_template_state(
            &deps.as_mut(),
            "template2",
            false, // approved should be false initially
            &publisher_address.to_string(),
            true, // private should be true
        );

        // Approve template
        let approve_response = approve_template(
            deps.as_mut(),
            env.clone(),
            admin_address.clone(),
            "template2".to_string(),
        )
        .unwrap();

        // Verify approve response attributes
        assert_eq!(approve_response.attributes.len(), 3);
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &approve_response.attributes[0],
            "method",
            "approve_template",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &approve_response.attributes[1],
            "template_id",
            "template2",
        );
        crate::tests_template_utils::tests_template_utils::assert_attr_eq(
            &approve_response.attributes[2],
            "approver",
            &admin_address.to_string(),
        );

        // Verify template state: approved=true, correct publisher, correct private flag
        assert_template_state(
            &deps.as_mut(),
            "template2",
            true, // approved should now be true
            &publisher_address.to_string(),
            true, // private should still be true
        );
    }
} 