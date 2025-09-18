
# deploy fee manager

## build
```cargo run-script optimize```

## deploy
```
thornode tx wasm store ../artifacts/auto_fee_manager.wasm \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --node https://stagenet-rpc.ninerealms.com:443 --type hash 00C56EE549A20A82FBC1C39B4CA758D905BAA01C826D7CB09E61B614EE0BD3BA
```
```
key: code_id
value: "881"
```
## instantiate
```
thornode tx wasm instantiate 874 \
'{
	"accepted_denoms": [{
		"denom": "rune",
		"max_debt": "1000000000",
		"min_balance_threshold": "1000"
	}],
	"execution_fees_destination_address": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr",
	"distribution_fees_destination_address": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr",
	"crank_authorized_address": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr",
	"workflow_manager_address": null,
	"creator_distribution_fee": "0"
}' \
--from test-stagenet-gus \
--label "auto-workflow-manager-gus" \
--admin test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --type hash 88C7F054F8F48D69387635A62FAFA8242261447301EDE9020AA35D6EF327B41F --node https://stagenet-rpc.ninerealms.com:443
```
```
key: _contract_address
value: sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts
``` 

## set workflow manager address

### make tx-sudo.txt with a transaction
```
thornode tx wasm execute sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts \
'{
"deposit": {}
}' \
--amount 1rune \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5 \
--generate-only > tx-sudo.json
```
### modify tx-sudo.json

- Change type to "@type":"/cosmwasm.wasm.v1.MsgSudoContract"
- Change msg
- Remove funds
- Change 'sender' for 'authority'
  
```
{"body":{"messages":[{"@type":"/cosmwasm.wasm.v1.MsgSudoContract","authority":"sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr","contract":"sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts","msg":{ "set_workflow_manager_address": { "address": "sthor1gdhtg02g07v84qjvjk7w7md83ckc4xk2gmx7pe4tap0tnaghes7shlqvcu" } }}],"memo":"","timeout_height":"0","unordered":false,"timeout_timestamp":null,"extension_options":[],"non_critical_extension_options":[]},"auth_info":{"signer_infos":[],"fee":{"amount":[],"gas_limit":"111081","payer":"","granter":""},"tip":null},"signatures":[]}
```

## send transaction
```
thornode tx sign tx-sudo.json \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 --node https://stagenet-rpc.ninerealms.com:443 \
--output-document tx-sudo-signed.json
```
```
thornode tx broadcast tx-sudo-signed.json --chain-id thorchain-stagenet-2 --node [https://stagenet-rpc.ninerealms.com:443](https://stagenet-rpc.ninerealms.com:443)
```
```
thornode query tx --type hash 1D37494A05DE7E6F837D426422DA25E567C7568F0495907B9C4286AB5FA99F85 --node https://stagenet-rpc.ninerealms.com:443
```
```
thornode query wasm contract-state smart sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts '{ "get_config": {} }'  --node https://stagenet-rpc.ninerealms.com:443
```

## deposit tokens for prepaid 

```
thornode query bank balances sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr --node https://stagenet-rpc.ninerealms.com
```
```
thornode tx wasm execute sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts \
'{
	"deposit": {}
}' \
--amount 10000000rune \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```

```
thornode query tx --type hash 3A831E4D19E84B5FC6C41B067D0840E9202A204C870D9ADE7360EFCDCF1E8580 --node https://stagenet-rpc.ninerealms.com:443
```
```
thornode query wasm contract-state smart sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts '{ "get_user_balances": { "user": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr" } }'  --node https://stagenet-rpc.ninerealms.com:443
```
```
thornode query wasm contract-state smart sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts '{ "get_creator_fees": { "user": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr" } }'  --node https://stagenet-rpc.ninerealms.com:443
```