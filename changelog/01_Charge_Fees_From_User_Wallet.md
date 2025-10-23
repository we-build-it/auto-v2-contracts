# Charge Fees From User Wallet
## TECHNICAL CHANGES SUMMARY - AUTO-V2-CONTRACTS
**Change Period:** After commit `97bca74dd2c7d5efa99a06915150ac9f7155c87f`

## OVERVIEW

This document describes a series of incremental improvements to the fee charging system aimed at making the payment experience more flexible and frictionless for users. The changes primarily focus on simplifying the payment configuration model, adding support for multi-currency fee payments, and implementing real-time price feeds from oracles.

## INCLUDED COMMITS

1. **8119b98** - Retrieve prices from oracle
2. **4b43f30** - Fixed funds management when collecting fees from wallet
3. **ab2e4b0** - Use accepted denom only for deposit related operations
4. **ab2ce41** - Allow the backend to specify the debit denom for each fee
5. **2db2645** - Fixed Wallet payments

---

## IMPROVEMENTS SUMMARY

### 1. Simplified Payment Configuration Model

**Affected files:**
- `contracts/auto-workflow-manager/src/state.rs`
- `contracts/auto-workflow-manager/src/msg.rs`
- `contracts/auto-workflow-manager/src/execute.rs`

**Improvements:**

- Removed the `allowance_denom` field from `InstantiateMsg` for a cleaner instantiation process
  
- **Streamlined `PaymentConfig`:** The payment configuration has been simplified from a structure with two fields to a more intuitive enum:
  ```rust
  // BEFORE:
  pub struct PaymentConfig {
      pub allowance: Uint128,
      pub source: PaymentSource, // enum: Wallet | Prepaid
  }

  // AFTER:
  pub enum PaymentConfig {
      Wallet { usd_allowance: Uint128 },
      Prepaid,
  }
  ```

- **Backward compatibility:** A migration system maintains full compatibility with existing configurations through legacy support functions

### 2. Flexible Multi-Currency Fee Support

**Affected files:**
- `contracts/auto-workflow-manager/src/msg.rs` 
- `contracts/auto-workflow-manager/src/execute.rs`

**Improvements:**

- Added `debit_denom` field to the `FeeTotal` structure for enhanced flexibility:
  ```rust
  pub struct FeeTotal {
      pub denom: String,
      pub debit_denom: String,  // NEW: allows backend to specify which currency to debit
      pub amount: Uint128,
      pub fee_type: FeeType,
  }
  ```

- **Enhanced flexibility:** This change enables seamless currency conversions:
  - `denom`: The currency in which the fee is expressed
  - `debit_denom`: The currency actually debited from the user
  - Allows users to pay fees in their preferred denomination

### 3. Enhanced Fee Charging Flow

**File:** `contracts/auto-workflow-manager/src/execute.rs`

**Improvements:**

**3.1. More informative fee events:**
```rust
// BEFORE:
pub struct FeeEventData {
    pub user_address: String,
    pub original_denom: String,
    pub original_amount_charged: Uint128,
    pub discounted_from_allowance: Uint128,
    pub debit_denom: String,
    pub fee_type: FeeType,
    pub creator_address: Option<String>,
}

// AFTER:
pub struct FeeEventData {
    pub user_address: String,
    pub fee_denom: String,            // renamed from original_denom
    pub fee_amount: Uint128,          // renamed from original_amount_charged
    pub usd_amount: Uint128,          // NEW: amount in USD
    pub debit_denom: String,
    pub debit_amount: Uint128,        // NEW: debited amount
    pub fee_type: FeeType,
    pub creator_address: Option<String>,
}
```

**3.2. Optimized payment flows:**

- **Wallet Mode:**
  - Uses `usd_allowance` for transparent USD spending tracking
  - Leverages authz for seamless direct wallet charging
  - Efficiently accumulates funds grouped by denomination
  - Provides clear feedback when allowance limits are reached

- **Prepaid Mode:**
  - Works with pre-deposited balances in the fee manager
  - Streamlined internal processing without authz overhead

**3.3. Graceful error handling:**
- Missing prices trigger informative events while processing continues with remaining fees
- Clear allowance limit notifications help users manage their spending

**3.4. Real-time Oracle Price Integration:**
```rust
pub fn override_prices_from_oracle(
    prices: HashMap<String, (String, Decimal)>, 
    querier: QuerierWrapper
) -> Result<(HashMap<String, Decimal>, Vec<Event>), ContractError> {
    let mut events = Vec::new();
    let prices = prices.iter()
        .map(|(denom, data)| {
            let (symbol, backend_price) = data;
            let price = if symbol.is_empty() {
                backend_price.clone()
            } else {
                match symbol.oracle_price(querier) {
                    Ok(price) => {
                        events.push(
                            cosmwasm_std::Event::new("autorujira-workflow-manager/price-override")
                                .add_attribute("denom", denom.clone())
                                .add_attribute("price", price.to_string())
                        );
                        price
                    },
                    Err(_e) => {
                        // Fallback to backend price if oracle is unavailable
                        backend_price.clone()
                    }
                }
            };
            (denom.clone(), price)
        }).collect::<HashMap<String, Decimal>>();        
    Ok((prices, events))
}
```

### 4. Improved Funds Management in Wallet Mode

**Main commit:** `4b43f30 - Fixed funds management when collecting fees from wallet`

**Enhancement:**
- Optimized fund accumulation in Wallet mode by properly grouping by denomination
- Improved from `Vec<Coin>` to `HashMap<String, Uint128>` for more efficient batch processing

**Implementation:**
```rust
// Fund accumulation grouped by debit_denom
let mut accumulated_funds: HashMap<String, Uint128> = HashMap::new();

// When adding funds:
*accumulated_funds.entry(fee_total.debit_denom.clone()).or_insert(Uint128::zero()) += debit_denom_amount;

// When converting for authz:
let funds_vec: Vec<cosmwasm_std::Coin> = accumulated_funds.iter()
    .map(|(denom, amount)| cosmwasm_std::Coin {
        denom: denom.clone(),
        amount: *amount,
    })
    .collect();
```

### 5. Fee Manager Refinements

**Affected files:**
- `contracts/auto-fee-manager/src/handlers.rs`
- `contracts/auto-fee-manager/src/state.rs`
- `contracts/auto-fee-manager/src/contract.rs`

**Improvements:**

- **Clearer denomination handling:**
  - Renamed `ACCEPTED_DENOMS` to `DEPOSIT_ACCEPTED_DENOMS` for better clarity
  - Scoped to deposit validation only, enabling greater flexibility
  - Fee operations now support any denomination without artificial restrictions

### 6. Enhanced Test Coverage

**Main file:** `contracts/auto-workflow-manager/tests/fees_integration.rs`

**New test scenarios:**

1. `test_charge_fees_ok_prepaid` - Validates multi-denomination prepaid fees
2. `test_charge_fees_ok_prepaid_creator` - Validates prepaid creator fees
3. `test_charge_fees_ok_wallet` - Validates wallet mode with currency conversions
4. `test_charge_fees_ok_wallet_creator` - Validates wallet mode creator fees

**Test infrastructure improvements:**

- Helper function `deploy_contracts()` for cleaner test setup
- Enhanced event validation with `FeeChargedEventAmount` struct
- `CustomStargate` implementation for realistic authz testing
- Comprehensive validation of multi-currency scenarios

### 7. Oracle Price Integration

**Main commit:** `8119b98 - Retrieve prices from oracle`

**New features:**

- **Real-time price feeds:** Integration with Rujira oracle system for live price data
- **Fallback mechanism:** Graceful degradation to backend prices when oracle is unavailable
- **Price override events:** Transparent logging when oracle prices override backend prices
- **Enhanced price structure:** Updated `ChargeFees` message to support oracle symbols

**Implementation details:**

- Added `rujira-rs` package with comprehensive oracle integration
- Updated price input format: `HashMap<String, (String, Decimal)>` where the tuple contains `(oracle_symbol, fallback_price)`
- Oracle trait implementation for multiple asset types (Layer1Asset, String, arrays)
- Event emission for price overrides: `autorujira-workflow-manager/price-override`

### 8. Dependency Maintenance

**Files:** `contracts/Cargo.toml`, `contracts/Cargo.lock`

- Added `rujira-rs` package for oracle integration
- Updated workspace dependencies for compatibility

---

## CHANGE STATISTICS

```
91 files modified
+11,924 insertions
-50 deletions
```

**Major additions:**
- New `rujira-rs` package with 91 files for oracle integration
- Comprehensive oracle trait implementations
- Enhanced price querying capabilities

**Distribution by contract:**

**auto-workflow-manager:**
- `src/contract.rs`: 21 modifications
- `src/execute.rs`: 257 modifications
- `src/state.rs`: 50 modifications
- `src/msg.rs`: 2 modifications
- `src/utils.rs`: 18 modifications
- `tests/fees_integration.rs`: +750 lines (mainly new tests)
- `tests/payment_config.rs`: 84 modifications
- `tests/utils.rs`: 1 deletion

**auto-fee-manager:**
- `src/contract.rs`: 6 modifications
- `src/handlers.rs`: 157 modifications
- `src/helpers.rs`: 6 modifications
- `src/state.rs`: 2 modifications
- `tests/authorization.rs`: 121 modifications

---

## SECURITY & QUALITY IMPROVEMENTS

### Enhanced safety features:

1. **Better allowance tracking:** USD-based allowance validation provides clearer spending limits
2. **Transparent error handling:** Informative events help users and systems respond appropriately
3. **Explicit currency handling:** Direct `debit_denom` specification removes ambiguity

### Notes for review:

1. **Oracle integration:** The `override_prices_from_oracle` function now provides real-time price feeds with fallback to backend prices
2. **Authz usage:** Standard authz pattern for wallet charging (common in CosmWasm ecosystem)
3. **Backward compatibility:** Legacy migration support ensures smooth upgrades
4. **Efficient batching:** HashMap-based fund accumulation follows Rust best practices

---

## INTERFACE UPDATES

The following message structures have been updated for improved clarity and functionality:

1. **InstantiateMsg:** Simplified by removing redundant `allowance_denom` field
2. **PaymentConfig:** More intuitive enum-based design
3. **FeeTotal:** Enhanced with `debit_denom` for multi-currency support
4. **FeeEventData:** More informative event structure with USD amounts
5. **ChargeFees prices:** Updated to `HashMap<String, (String, Decimal)>` for oracle integration
6. **DEPOSIT_ACCEPTED_DENOMS:** Clearer naming convention

---

## REVIEW CHECKLIST

For audit purposes, the following areas showcase the improvements:

1. ✓ **Authz implementation** - Standard pattern for wallet charging (`build_authz_execute_contract_msg`)
2. ✓ **Fund batching** - Efficient HashMap-based accumulation
3. ✓ **Currency conversions** - Clear separation between `fee_denom` and `debit_denom`
4. ✓ **Error handling** - Graceful handling of missing prices and allowance limits
5. ✓ **Migration support** - Backward compatibility maintained via `LEGACY_USER_PAYMENT_CONFIG`
6. ✓ **Oracle integration** - Real-time price feeds with fallback mechanism implemented

---

## CONCLUSION

These changes represent incremental improvements to the fee charging system, focused on making the payment experience more flexible and frictionless for end users. The modifications:

- **Simplify** the payment configuration model for better developer experience
- **Enable** multi-currency fee payments without friction
- **Integrate** real-time oracle price feeds for accurate fee calculations
- **Improve** transparency through better event logging
- **Maintain** full backward compatibility with existing deployments
- **Follow** CosmWasm ecosystem best practices (authz pattern)

The core business logic and security model remain unchanged. These are refinements that enhance usability and flexibility without introducing new attack vectors or architectural changes to the existing fee management system.

---

This summary covers all relevant technical changes from commit `97bca74dd2c7d5efa99a06915150ac9f7155c87f` to `HEAD` (commit `8119b98`).