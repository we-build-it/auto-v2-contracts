# Echo Contract

A simple CosmWasm contract that acts as an "echo" service, receiving any message and emitting an event with the received message.

## Features

- **Echo Messages**: Receive any binary message and emit an event with the message content
- **Custom Attributes**: Support for custom event attributes
- **Message Validation**: Size limit validation (1MB max)
- **Message Counting**: Track total number of messages processed
- **Message Hashing**: Generate MD5 hash of messages for reference

## Messages

### Instantiate
```json
{
  "admin": "kujira1..."
}
```

### Execute

#### Echo
```json
{
  "echo": {
    "message": "base64_encoded_binary_data"
  }
}
```

#### Echo with Custom Attributes
```json
{
  "echo_with_attributes": {
    "message": "base64_encoded_binary_data",
    "attributes": [
      ["custom_key", "custom_value"],
      ["another_key", "another_value"]
    ]
  }
}
```

### Query

#### Config
```json
{
  "config": {}
}
```

#### Message Count
```json
{
  "message_count": {}
}
```

## Events

The contract emits `echo_message` events with the following attributes:

- `sender`: Address of the message sender
- `message_count`: Sequential number of the message
- `message_size`: Size of the message in bytes
- `message_hash`: MD5 hash of the message content
- Custom attributes (if provided)

## Building

```bash
# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Generate schema
cargo run --bin schema_echo

# Optimize for deployment
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0 echo-contract
```

## Usage Examples

### Basic Echo
```bash
# Send a simple text message
echo "Hello World" | base64 | xargs -I {} wasmd tx wasm execute $CONTRACT_ADDR '{"echo":{"message":"{}"}}' --from $KEY_NAME -y
```

### Echo with Custom Attributes
```bash
# Send message with custom attributes
wasmd tx wasm execute $CONTRACT_ADDR '{
  "echo_with_attributes": {
    "message": "SGVsbG8gV29ybGQ=",
    "attributes": [
      ["message_type", "greeting"],
      ["priority", "high"]
    ]
  }
}' --from $KEY_NAME -y
```

### Query Message Count
```bash
wasmd query wasm contract-state smart $CONTRACT_ADDR '{"message_count":{}}'
```
