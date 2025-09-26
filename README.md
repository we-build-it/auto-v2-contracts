# Auto V2 Contracts

This repository contains the CosmWasm smart contracts for the AUTO automation system. These contracts form the foundation of the automation ecosystem, providing workflow management and fee handling.

## Contracts

### 🔄 Auto Workflow Manager

The **Auto Workflow Manager** is the core component of the AUTO automation system that manages workflow lifecycle, instance execution, and action orchestration.

**Key Features:**
- **Workflow Management**: Publishing of public and private workflows with defined actions
- **Instance Execution**: Complete lifecycle management from creation to completion
- **Action Orchestration**: Definition of complex workflows with multiple actions and dependencies
- **Access Control**: Role-based authorization for publishers and action executors

**Full Documentation:** [📖 Auto Workflow Manager README](./contracts/auto-workflow-manager/README.md)

### 💰 Auto Fee Manager

The **Auto Fee Manager** handles all aspects of fee logic for workflows executed by the AUTO engine, including user balances, fee collection, and revenue distribution.

**Key Features:**
- **Balance Management**: Multi-denomination support with debt tracking
- **Fee Collection**: Execution and creator fees with batch processing
- **Revenue Distribution**: Subscription-based distribution system for creators
- **Secure Integration**: Specific authorization for automated fee collection

**Full Documentation:** [📖 Auto Fee Manager README](./contracts/auto-fee-manager/README.md)

## SDK

### TypeScript SDK

A TypeScript SDK is available to facilitate integration with the contracts:

**SDK Documentation:** [📖 TypeScript SDK README](./sdks/auto-workflow-manager/README.md)

## Architecture

The contracts are designed to work together:

1. **Auto Workflow Manager** orchestrates workflow execution
2. **Auto Fee Manager** handles all related financial transactions
3. Both contracts integrate securely to provide a complete automation system

## Building

```bash
# Build all contracts
cargo build --target wasm32-unknown-unknown --release

# Optimize for deployment
cargo run-script optimize
```

## Testing

```bash
# Run all tests
cargo test

# Contract-specific tests
cd contracts/auto-workflow-manager && cargo test
cd contracts/auto-fee-manager && cargo test
```

## Examples

Usage examples are available in:

- **Workflow Manager:** [📄 Workflow Manager Examples](./examples/workflow-manager.md)
- **Fee Manager:** [📄 Fee Manager Examples](./examples/fee-manager.md)

## License

This project is licensed under the same terms as the AUTO automation system.
