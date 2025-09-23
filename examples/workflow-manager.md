# Workflow manager

## current contract addresses
local testing (gus): sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z
stagenet: sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z

## grants

### give grants  
```
thornode tx authz grant \
sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z generic \
--msg-type /cosmwasm.wasm.v1.MsgExecuteContract \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--gas auto --gas-adjustment 1.5 \
--node https://stagenet-rpc.ninerealms.com:443
```
### query grants
```
thornode query authz grants sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z --node https://stagenet-rpc.ninerealms.com:443
```

## build
```
cargo run-script optimize
```
## deploy
```
thornode tx wasm store ../artifacts/auto_workflow_manager.wasm \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --node https://stagenet-rpc.ninerealms.com:443 --type hash CE08279EB653DE96816662F7FB7DF721E7078F4C6C07D149F2B853653411E1AF
```
```
key: code_id
value: "915"
```
## migrate

```
thornode tx wasm migrate sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z 915 'null' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```
## instantiate  

```
thornode tx wasm instantiate 872 \
'{
	"allowed_publishers": ["sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr","sthor1tvtt3qv3clu9g4nyu2vr307l2vnrugpxdp5zze","sthor1t78mcm6nhh999lvu3dg4l7lm8976pkakpmw4e3"],
	"allowed_action_executors": ["sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr","sthor1tvtt3qv3clu9g4nyu2vr307l2vnrugpxdp5zze","sthor1t78mcm6nhh999lvu3dg4l7lm8976pkakpmw4e3"],
	"referral_memo": "auto-workflow-manager-stagenet",
	"fee_manager_address": "sthor1whr8522407tmsfa9qhyqcu25t3cq9jvl46cg5meds6stkq4vzwjqzpl5ts",
	"allowance_denom": "rune"
}' \
--from test-stagenet-gus \
--label "auto-workflow-manager-gus" \
--admin test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --type hash 91DC62B732AE1172F1B0AAE41835FDA6ACA5B2C55A8ADAACFA09EFF31C10CD0B --node https://stagenet-rpc.ninerealms.com:443
```
```
key: _contract_address
value: sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z
```
## set payment config

```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"set_user_payment_config": {
		"payment_config": {
			"allowance": "10000000",
			"source": "prepaid"
		}
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --type hash 927DA03CF1379E96A46BCE043CCB03B0ED4832ADAF22C75DF896A55E6FE3F212 --node https://stagenet-rpc.ninerealms.com:443
```
```
thornode query wasm contract-state smart sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z '{ "get_user_payment_config": {"user_address": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr" } }' --node https://stagenet-rpc.ninerealms.com:443
```
## publish workflow
```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"publish_workflow": {
		"workflow": {
		"id": "91c6e3df1bfd6b1e8a9064fbc3c6eeab6439ee3a3e28ebbc6ec8a97edc98da68",
		"start_actions": [
			"claim"
		],
		"end_actions": [
			"stake"
		],
		"visibility": "public",
		"actions": {
			"claim": {
				"params": {
					"templateId": {
						"string": "#ip.claimTemplateId"
					},
					"contractAddress": {
						"string": "#ip.claimContractAddress"
					},
					"userAddress": {
						"string": "#ip.requester"
					},
					"distributionId": {
						"string": "#ip.distributionId"
					}
				},
				"next_actions": [
					"stake"
				],
				"templates": {
					"daodao": {
						"contract": "{{contractAddress}}",
						"message": "{\"claim\":{ \"id\": {{distributionId}} }}",
						"funds": []
					}
				},
				"whitelisted_contracts": [
					"sthor1du9dd7w44dqnadt76dr6pks6m3lma40fttfqxfyz4nm5l7npfg6qx9mqfz"
				]
			},
			"stake": {
				"params": {
					"templateId": {
						"string": "#ip.stakeTemplateId"
					},
					"contractAddress": {
						"string": "#ip.stakeContractAddress"
					},
					"userAddress": {
						"string": "#ip.requester"
					},
					"amount": {
						"string": "#cp.amount"
					},
					"denom": {
						"string": "#cp.denom"
					}
				},
				"next_actions": [],
				"templates": {
					"daodao": {
						"contract": "{{contractAddress}}",
						"message": "{\"stake\":{ }}",
						"funds": [
							[
							"{{amount}}",
							"{{denom}}"
							]
							]
						}
					},
					"whitelisted_contracts": [
						"sthor17yxuu3762ccgq93dsxf2r8r4lwjz5mqz4ztz26e6dljfh0gkqjwq3xug06"
					]
				}
			}
		}
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com:443 \
--gas auto --gas-adjustment 1.5
```

```
thornode query tx --node https://stagenet-rpc.ninerealms.com:443 --type hash 4AD0E299AD2ACE329FA7586D0120D401894ACB9055F2AEDF611110188BD03873
```

```
thornode query wasm contract-state smart sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z '{ "get_workflow_by_id": {"workflow_id": "03f321e152ceef39cef917c241b928173078920b437c75e8ec0358b5c070f1b0" } }' --node https://stagenet-rpc.ninerealms.com:443
```

## execute instance
```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"execute_instance": {
		"instance": {
			"workflow_id": "91c6e3df1bfd6b1e8a9064fbc3c6eeab6439ee3a3e28ebbc6ec8a97edc98da68",
			"onchain_parameters": {
				"claimTemplateId": { "string": "daodao" },
				"claimContractAddress": { "string": "sthor1du9dd7w44dqnadt76dr6pks6m3lma40fttfqxfyz4nm5l7npfg6qx9mqfz" },
				"stakeTemplateId": { "string": "daodao" },
				"stakeContractAddress": { "string": "sthor17yxuu3762ccgq93dsxf2r8r4lwjz5mqz4ztz26e6dljfh0gkqjwq3xug06" },
				"distributionId": { "string": "1" }
			},
			"offchain_parameters": {
				"checkProvider": { "string": "daodao" },
				"minAmountToClaim": { "string": "1" },
				"protocolName": { "string": "AUTO" },
				"notificationType": { "string": "telegram" }
			},
			"execution_type": "recurrent",
			"cron_expression": "*/5 * * * *",
			"expiration_time": "1786448860000000000"
		}
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --type hash 7D960565B935527B4E2FE636D049EDB4CA77CF575EE3FF2CD4FBD7C1E0DB3BF7 --node https://stagenet-rpc.ninerealms.com:443
```

## pause/resume/cancel/cancel-run

```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"pause_schedule": {
		"instance_id": 15
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```

```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"resume_schedule": {
		"instance_id": 14
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```

```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"cancel_instance": {
		"instance_id": 16
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```

```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"cancel_run": {
		"instance_id": 15
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```

```
thornode query tx --type hash 92EB209E2D8A25FD04354054689DA9BCCF1B888181661BE5F0E33C0BC31E1FD1 --node https://stagenet-rpc.ninerealms.com:443
```

## execute action
```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"execute_action": {
		"user_address": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr",
		"instance_id": 1,
		"action_id": "stake",
		"template_id": "daodao",
		"params": {
			"#cp.denom": {
				"string": "x/sthor1trgd94z4j8gyw98eeyq98tm9qzlkvkhjrcyjy8smv8hgvmsz7vwqflutqh/auto"
			},
			"#cp.amount": {
				"string": "2816389937"
			}
		}
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```
```
thornode query tx --type hash 33844EEB0318E1827AC3FD775A6B21FACD546E037D798155838853A177737AD5 --node https://stagenet-rpc.ninerealms.com:443
```

## charge fees
```
thornode tx wasm execute sthor1fs54ndfnetj9aww5guxvvpryf60mft99ra5zh4ezzypa35huuzmqalnv9z \
'{
	"charge_fees": {
		"batch_id": "7895a55b72fc3cd6541c8a37f5c88dc4d07a12497f1f4cb05db1065e6814215a",
		"fees": [
			{
				"address": "sthor1z6m4jukpzelp26f8k7jcua4xxsp2w2lpqzv6nr",
				"totals": [
					{
						"amount": "4",
						"denom": "rune",
						"denom_decimals": 8,
						"fee_type": "execution"
					},
					{
						"amount": "7970546",
						"denom": "x/sthor1trgd94z4j8gyw98eeyq98tm9qzlkvkhjrcyjy8smv8hgvmsz7vwqflutqh/auto",
						"denom_decimals": 10,
						"fee_type": {
							"creator": {
								"instance_id": 1
							}
						}
					}
				]
			}
		]
	}
}' \
--from test-stagenet-gus \
--chain-id thorchain-stagenet-2 \
--node https://stagenet-rpc.ninerealms.com/ \
--gas auto --gas-adjustment 1.5
```  
```
thornode query tx --type hash A9EC578197E3ACF99CBAA430FA31C8AFAAFC270FECFFCD67757562AD96879C41 --node https://stagenet-rpc.ninerealms.com:443
```