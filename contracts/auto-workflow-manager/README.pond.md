# Testing with pond

## CosmWasm feature version

CosmWasm version is different for pond and thorchain

1) Set the correct version in Cargo.toml depending if you want to use pond or thorchain
- pond: cosmwasm_1_4
- thorchain: cosmwasm_2_0

2) Check utils.rs 
- build_authz_msg function has little changes depending version

## Setup pond

Clone https://github.com/Team-Kujira/pond/tree/main and follow install instructions

## Deploy/Instantiate contract

### Build binary

```
$ ./build-optimized.sh
```

### Accounts

```
chain  name     address
kujira deployer kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse ===> Deployer & Contract Admin
kujira test0    kujira1cyyzpxplxdzkeea7kwsydadg87357qnaww84dg ===> Approver
kujira test1    kujira18s5lynnmx37hq4wlrw9gdn68sg2uxp5r39mjh5 ===> Template publisher
kujira test2    kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9 ===> Flow requester
```

### Deploy to pond

```
$ pond deploy artifacts/proxy_auth.wasm

Jul 15 11:52:15 INF code deployed code_id=11
```

### Instantiate

Replace "11" for code_id

```
$ pond tx wasm instantiate 11 '{ "approvers": ["kujira1cyyzpxplxdzkeea7kwsydadg87357qnaww84dg"] }' \
	--label "autorujira.autobidder" \
	--from kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse \
	--admin kujira14gs9zqh8m49yy9kscjqu9h72exyf295asmt7nw \
	--gas auto \
	--gas-adjustment 1.3 \
	--fees 10ukuji

...
txhash: 4BC66EB595AD34F88BADE71FC7E067A59952B6A2A368045CBE98677105B2E6A4
...
```

Get contract address from txhash. Example:
```
$ pond query tx 4BC66EB595AD34F88BADE71FC7E067A59952B6A2A368045CBE98677105B2E6A4

...
- attributes:
  - index: true
    key: _contract_address
    value: kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp
  - index: true
    key: code_id
    value: "11"
...
```

## Publish template

### Request for approval

Let's create a simple template with only one action. The action mints USK's using ETH

```
$ pond tx wasm execute kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp '{
  "request_for_approval": {
    "template": {
      "id": "template_001",
      "actions": [
        {
          "id": "action_001",
          "message_template": "{\"deposit\":{\"address\":\"{{fromAddress}}\"}}",
          "contract_address": "kujira1lcqrewe4fwcs7fx3y2uwxx6afh48833zlcaxjn7tpdragzcmhuhs083qj5",
          "allowed_denoms": [
            "factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth"
          ]
        }
      ],
      "private": false
    }
  }
}' --from test1 --broadcast-mode=sync --yes

...
txhash: 9BF00F140F63E4BB3D57241656D897157E79825BD3B0F297E7B7E5A18268EB03
...
```

### Approve

```
$ pond tx wasm execute kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp '{
  "approve_template": {
    "template_id": "template_001"
  }
}' --from test0 --broadcast-mode=sync --yes
```

### Query templates

```
$ pond query wasm contract-state smart kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp '{
  "get_templates_by_publisher": {
    "publisher_address": "kujira18s5lynnmx37hq4wlrw9gdn68sg2uxp5r39mjh5"
  }
}'
```

## Authz

### Authorize

This contract can execute other contracts on behalf of users

```
$ pond tx authz grant kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp generic --msg-type /cosmwasm.wasm.v1.MsgExecuteContract --from kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9 -y
```

This contract can spend tokens on behalf of users

```
$ pond tx authz grant kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp send --spend-limit 5000000000factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth --from kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9 -y
```

### Revoke
```
$ pond tx authz revoke kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp /cosmwasm.wasm.v1.MsgExecuteContract --from kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9 -y
```

### Query
```
$ pond query authz grants kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9 kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp
```

## Flows

### Register flow execution

```
$ pond tx wasm execute kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp '{
  "execute_flow": {
    "flow_id": "flow_001",
    "template_id": "template_001",
    "params": "{\"recipient\":\"wasm1...\",\"amount\":\"1000000uatom\",\"validator\":\"cosmosvaloper1...\"}"
  }
}' --from test2 --broadcast-mode=sync --yes
```

### Query flows

```
$ pond query wasm contract-state smart kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp '{
  "get_flows_by_requester": {
    "requester_address": "kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9"
  }
}'
```

### Execute action

```
$ pond tx wasm execute kujira1rh4fe5wyd80xme0qey3nckl2wkv8n3eqhsclgdzszmuydjn4kkyqp86xdp '{
  "execute_action": {
    "flow_id": "flow_001",
    "action_id": "action_001",
    "params": {
      "fromAddress": "kujira1qwexv7c6sm95lwhzn9027vyu2ccneaqa5xl0d9"
    },
    "funds": [
      {
        "denom": "factory/kujira1k3g54c2sc7g9mgzuzaukm9pvuzcjqy92nk9wse/aeth",
        "amount": "1000000"
      }
    ]
  }
}' --from deployer --gas auto --gas-adjustment 1.5 --broadcast-mode=sync --yes
```