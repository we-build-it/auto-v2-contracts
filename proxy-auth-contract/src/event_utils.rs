use cosmwasm_std::{Reply, SubMsgResponse, SubMsgResult, Uint128};

/// Function to extract the value of a specific attribute from events of a specific type in a Reply response.
///
/// # Parameters
/// - `reply`: Reference to the `Reply` structure obtained in the `reply` function.
/// - `event_type`: The event type (`event.ty`) where to look for the attribute.
/// - `attribute_key`: The attribute key (`attribute.key`) you want to find.
///
/// # Returns
/// - `Option<String>`: Returns `Some(value)` if the attribute is found, or `None` if not found.
pub fn extract_attribute_from_reply(
    reply: &Reply,
    event_type: &str,
    attribute_key: &str,
) -> Option<String> {
    if let SubMsgResult::Ok(SubMsgResponse { events, .. }) = &reply.result {
        // Iterate over the events
        for event in events {
            if event.ty == event_type {
                // Only consider events of the specified type
                for attribute in &event.attributes {
                    if attribute.key == attribute_key {
                        // Found the attribute; return its value
                        return Some(attribute.value.clone());
                    }
                }
            }
        }
    }
    // If the attribute was not found, return None
    None
}

use cosmwasm_std::Coin;
use std::str::FromStr;

/// Splits a concatenated string into `amount` and `denom` and creates a `Coin`.
///
/// # Arguments
///
/// * `input` - A string slice that holds the concatenated `amount` and `denom`.
///
/// # Returns
///
/// * `Ok(Coin)` if the parsing is successful.
/// * `Err(String)` if there is an error during parsing.
pub fn split_amount_denom(input: &str) -> Result<Coin, String> {
    // Encuentra el índice donde terminan los dígitos (amount) y comienza el denom.
    let split_index = input
        .find(|c: char| !c.is_digit(10))
        .ok_or_else(|| "Input string does not contain a valid amount.".to_string())?;

    // divide between denom and amount.
    let (amount_str, denom_str) = input.split_at(split_index);

    // parse amount as Uint128.
    let amount =
        Uint128::from_str(amount_str).map_err(|e| format!("Failed to parse amount: {}", e))?;

    // remove wrong chars at start of denom
    let denom = denom_str.trim_start_matches(|c: char| c == '0'); // Ajusta según tu formato específico.

    // verify valid denom
    if denom.is_empty() {
        return Err("Denom part is empty.".to_string());
    }

    Ok(Coin {
        denom: denom.to_string(),
        amount,
    })
}

/**
 * Searches for a specific attribute value within events of a given type in a `Reply`.
 *
 * # Arguments
 *
 * * `reply` - A reference to the `Reply` containing the events.
 * * `event_type` - The type of the event to filter (e.g., "transfer").
 * * `filter_attr_key` - The key of the attribute to filter by (e.g., "recipient").
 * * `filter_attr_value` - The value of the attribute to filter by (e.g., "kujira1cyyzpxplxdzkeea7kwsydadg87357qnaww84dg").
 * * `value_attr_key` - The key of the attribute whose value needs to be retrieved (e.g., "amount").
 *
 * # Returns
 *
 * * `Some(String)` containing the value of the specified attribute if found.
 * * `None` if the event or attributes are not found.
 */
pub fn extract_attribute_from_reply_with_filters(
    reply: &Reply,
    event_type: &str,
    filter_attr_key: &str,
    filter_attr_value: &str,
    value_attr_key: &str,
) -> Option<String> {
    if let SubMsgResult::Ok(SubMsgResponse { events, .. }) = &reply.result {
        // Iterate over the events
        for event in events {
            // Check if the event type matches the specified type
            if event.ty == event_type {
                // Find the filter attribute within the event
                let filter_attr = event
                    .attributes
                    .iter()
                    .find(|attr| attr.key == filter_attr_key);

                // If the filter attribute exists and its value matches
                if let Some(attr) = filter_attr {
                    if attr.value == filter_attr_value {
                        // Find the target attribute whose value needs to be returned
                        if let Some(target_attr) = event
                            .attributes
                            .iter()
                            .find(|attr| attr.key == value_attr_key)
                        {
                            return Some(target_attr.value.clone());
                        }
                    }
                }
            }
        }
    }

    // Return None if the event or attributes are not found
    None
}
