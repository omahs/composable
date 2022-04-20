
# Overview


This proposal descrbites flow of adding XCMP channel amid Statemine and Picasso

## Proposal

This proposal aims to open HRMP channel between Picasso & Karura. For more context please read here.

Let me explain the technical details of this call. It is a batch transaction with two calls:

1. A force transfer from Kusama treasury (`F3opxRbN5ZbjJNU511Kj2TLuzFcDq9BGduA9TgiECafpg29`) to Statemine (`F7fq1jSNVTPfJmaHaXCMtatT1EZefCUsa7rRiQVNR5efcah`). The amount is 11 KSM. 10 KSM will be used for deposit to accept (5 KSM) and open (5 KSM) HRMP channel. 1 KSM will be used by Statemine parachain to pay for transaction execution fee on Kusama. Note that 1 KSM is more than enough and unused funds will be trapped in XCM asset trap. But that's totally fine as it can be claimed & used for transaction fee in later XCM executions.

2. Send XCM message to Statemine to execute a transaction with superuser (root) permission.

The XCM message to Statemine is 0x1f00010100020c000400000000070010a5d4e81300000000070010a5d4e800060002286bee5c1802083c01d00700003c00d0070000e803000000900100, which can be decoded on Statemine, and it is polkadotXcm.send. It sends a XCM message back to Kusama, to with 1 KSM for transaction fee and perform a transact of call 0x1802083c01d00700003c00d0070000e803000000900100.

The call is is a batch call that accepts open channel request from Karura, and make an open channel request to Karura.


```json
[
    {
        "callIndex": "0x0402",
        "args": {
            "source": {
                "id": "F3opxRbN5ZbjJNU511Kj2TLuzFcDq9BGduA9TgiECafpg29"
            },
            "dest": {
                "id": "F7fq1jSNVTPfJmaHaXCMtatT1EZefCUsa7rRiQVNR5efcah"
            },
            "value": 11000000000000
        }
    },
    {
        "callIndex": "0x6300",
        "args": {
            "dest": {
                "v1": {
                    "parents": 0,
                    "interior": {
                        "x1": {
                            "parachain": 1000
                        }
                    }
                }
            },
            "message": {
                "v2": [
                    {
                        "transact": {
                            "originType": "Superuser",
                            "requireWeightAtMost": 1000000000,
                            "call": {
                                "encoded": "0x1f00010100020c000400000000070010a5d4e81300000000070010a5d4e800060002286bee5c1802083c01e70700003c00e7070000e803000000900100"
                            }
                        }
                    }
                ]
            }
        }
    }
]
```


## References


- https://kusama.polkassembly.io/referendum/163

- https://kusama.polkassembly.io/referendum/164

Picasso ParaId 

2087