use std::collections::HashSet;

use cosmwasm_std::{Addr, Order, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Ownership {
    pub owner: Addr,
    pub approvers: HashSet<Addr>,
}
#[cw_serde]
pub struct Action {
    pub id: String,
    pub template_id: String,
    pub message_template: String,
    pub target_contract: Addr,
    pub allowed_denoms: HashSet<String>,
}

#[cw_serde]
pub struct Template {
    pub id: String,
    pub approved: bool,
    pub private: bool,
    pub publisher: Addr,
}

#[cw_serde]
pub struct Flow {
    pub id: String,
    pub template_id: String,
    pub params: String,
    pub requester: Addr,
}

// ========== OWNERSHIP ==========
pub const OWNERSHIP: Item<Ownership> = Item::new("ownership");

pub fn save_ownership(storage: &mut dyn Storage, ownership: &Ownership) -> StdResult<()> {
    OWNERSHIP.save(storage, ownership)
}

pub fn load_ownership(storage: &dyn Storage) -> StdResult<Ownership> {
    OWNERSHIP.load(storage)
}

pub fn validate_sender_is_approver(
    storage: &dyn Storage,
    info: &cosmwasm_std::MessageInfo,
) -> Result<(), crate::error::ContractError> {
    let state = load_ownership(storage)?;
    if !state.approvers.contains(&info.sender) {
        Err(crate::error::ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub fn validate_sender_is_admin(
    storage: &dyn Storage,
    info: &cosmwasm_std::MessageInfo,
) -> Result<(), crate::error::ContractError> {
    let state = load_ownership(storage)?;
    if info.sender != state.owner {
        Err(crate::error::ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

// ========== FLOWS ==========
pub struct FlowIndexes<'a> {
    // <'a, index key type, struct value, primary key type>
    pub requester: MultiIndex<'a, Addr, Flow, String>,
}
impl<'a> IndexList<Flow> for FlowIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Flow>> + '_> {
        let v: Vec<&dyn Index<Flow>> = vec![
            &self.requester,
        ];
        Box::new(v.into_iter())
    }
}
pub fn flows<'a>() -> IndexedMap<String, Flow, FlowIndexes<'a>> {
    let indexes = FlowIndexes {
        requester: MultiIndex::new(
            |_pk: &[u8], flow: &Flow| flow.requester.clone(),
            "flows",
            "flows__requester",
        ),
    };
    IndexedMap::new("flows", indexes)
}

pub fn save_flow(storage: &mut dyn Storage, flow: &Flow) -> StdResult<()> {
    flows().save(storage, flow.id.clone(), flow)
}

pub fn load_flow(storage: &dyn Storage, id: &str) -> StdResult<Flow> {
    flows().load(storage, id.to_string())
}

pub fn remove_flow(storage: &mut dyn Storage, id: &str) -> StdResult<()> {
    flows().remove(storage, id.to_string())?;
    Ok(())
}

pub fn query_flows_by_requester(storage: &dyn Storage, requester: Addr) -> StdResult<Vec<Flow>> {
    flows()
        .idx
        .requester
        .prefix(requester.clone())
        .range(storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, flow)| flow))
        .collect()
}

// ========== TEMPLATES ==========
pub struct TemplateIndexes<'a> {
    // <'a, index key type, struct value, primary key type>
    pub publisher: MultiIndex<'a, Addr, Template, String>,
}
impl<'a> IndexList<Template> for TemplateIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Template>> + '_> {
        let v: Vec<&dyn Index<Template>> = vec![
            &self.publisher,
        ];
        Box::new(v.into_iter())
    }
}
pub fn templates<'a>() -> IndexedMap<String, Template, TemplateIndexes<'a>> {
    let indexes = TemplateIndexes {
        publisher: MultiIndex::new(
            |_pk: &[u8], template: &Template| template.publisher.clone(),
            "templates",
            "templates__publisher",
        ),
    };
    IndexedMap::new("templates", indexes)
}

pub fn save_template(storage: &mut dyn Storage, template: &Template) -> StdResult<()> {
    templates().save(storage, template.id.clone(), template)
}

pub fn load_template(storage: &dyn Storage, template_id: &str) -> StdResult<Template> {
    templates().load(storage, template_id.to_string())
}

pub fn remove_template(storage: &mut dyn Storage, template_id: &str) -> StdResult<()> {
    templates().remove(storage, template_id.to_string())?;

    let actions_to_remove: StdResult<Vec<(String, String)>> = template_actions()
        .idx
        .template_id
        .prefix(template_id.to_string())
        .range(storage, None, None, Order::Ascending)
        .map(|item| item.map(|(key, _)| key))
        .collect();

    for key in actions_to_remove? {
        template_actions().remove(storage, key)?;
    }

    Ok(())
}

pub fn query_templates_by_publisher(storage: &dyn Storage, publisher: Addr) -> StdResult<Vec<Template>> {
    templates()
        .idx
        .publisher
        .prefix(publisher.clone())
        .range(storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, template)| template))
        .collect()
}

// ========== ACTIONS ==========
pub struct ActionIndexes<'a> {
    pub template_id: MultiIndex<'a, String, Action, (String, String)>,
}

impl<'a> IndexList<Action> for ActionIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Action>> + '_> {
        let v: Vec<&dyn Index<Action>> = vec![&self.template_id];
        Box::new(v.into_iter())
    }
}

pub fn template_actions<'a>() -> IndexedMap<(String, String), Action, ActionIndexes<'a>> {
    let indexes = ActionIndexes {
        template_id: MultiIndex::new(
            |_pk: &[u8], action: &Action| action.template_id.clone(),
            "template_actions",
            "template_actions__template_id",
        ),
    };
    IndexedMap::new("template_actions", indexes)
}

pub fn save_template_action(
    storage: &mut dyn Storage,
    template_id: &str,
    action_id: &str,
    action: &Action,
) -> StdResult<()> {
    template_actions().save(storage, (template_id.to_string(), action_id.to_string()), action)
}

pub fn load_template_action(
    storage: &dyn Storage,
    template_id: &str,
    action_id: &str,
) -> StdResult<Action> {
    template_actions().load(storage, (template_id.to_string(), action_id.to_string()))
}

pub fn remove_template_action(
    storage: &mut dyn Storage,
    template_id: &str,
    action_id: &str,
) -> StdResult<()> {
    template_actions().remove(storage, (template_id.to_string(), action_id.to_string()))?;
    Ok(())
}

pub fn get_template_actions(storage: &dyn Storage, template_id: &str) -> StdResult<Vec<Action>> {
    template_actions()
        .idx
        .template_id
        .prefix(template_id.to_string())
        .range(storage, None, None, Order::Ascending)
        .map(|item| item.map(|(_, action)| action))
        .collect()
}

