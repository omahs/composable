# Overview

XCM is cross chain message standard. 


This proposal allows 


## Why IBC and Cosmwasm?

There is ongoing development of IBC Bridge

There is CosmwasmVM which runs insided Substeat.

Plans of Parity for v4

IBC and Cosmwasm are designeg to be cross chain and asyncrnois. So it is natural choise for XCM.

IBC is rich cross chain ecosystem of blockchains any of which can run XCM.


## About author

Author has experinence setting up XCM for one of Parachains.

Also has experience in developing bits for fully cross chain authort.



## Deliver

XCM compiled in cosmwasm contract

`xcm-executor` - corresponds to parity xcm-exeuctro
`xcm-coniguraiton` - simular to usual configuraiton in substrate chain. Maps addresses, IBC connection/ports to and from XCM MultiLocaiton, barrier
`xcm-transactor` - uses `xcm-coniguraiton` and `cw20-ics20` to transact assets 
`xcm-trap-claim` - lost assets traps and claims
`xcm-runtime` - governance of all contracts (like treasurys) 
`xcm-simulator` helper tools to run `xcm` message amid several contracts on top of `cosmwasm-vm-test`. For each case there will be at least 1 positive test.
`xcm-sender` - sends message to IBC



## Will not deliver
- UI/FE
- Deploying and operating conract on any IBC chain
- Interpretation of `Transact` instruction 
- Interpretation of subscription/query to version, only V2(or if V3 will be released in time and compiled under wasm - v3) will be support
- Serde of XCM in Json/PB as used Cosmos.
- Security audits and specificaition of governance model


## Maitaince

Will maintain solution with latest XCM and Coswasm crates and APIs within 3year.

Will apply proposal to Composable, InfromatSystems and Osmososig to fit contracts itno their own ecosystem. 


## Disclosere

I work on Composalbe which developers XCVM as direct competitor of XCM. 

By own believe that XCM can be bring good.

I will not use Composable time, money or other resoruces to deliver on grant proposal.
