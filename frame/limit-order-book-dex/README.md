# Overview

The exchange allows placing buy and sell orders at specific price levels, or at market level. The market level price can be provided by a combination of `pallet-oracle` and the future AMM DEX

Here is we design cross chain DEX. It will have interfaces like if it is on chain for pallets, but token swaps managed asynchronously by parachain (bridges). This pallet has only API to be called from bridge callbacks, not calling it.

Our DEX represents SELL side of traditional OB.

## Order book designs

### HydraDX

https://github.com/galacticcouncil/Basilisk-node/tree/master/pallets/exchange

- Intention to sell (a,b) and buy (b,a) are added during block
- Each block cleaned, so no data retained in block about intentions
- If exact matches found, than sell via OB
- If not exact found, sell remaining on AMM
- Can be used without AMM if set AMM allowance to low percentage or disable on runtime

### PolkaDex

https://github.com/Polkadex-Substrate/Documentation/blob/master/polkadex-lightpaper.md
https://docs.polkadex.trade/orderbookArchitecture

- Allows to inject AMM bots
- Any OB order is sold on AMM, if AMM provides better price
- People pay fees only for ddosing attacks (like wrong assets, bad input)
- Issues trade order into TEE or onto on chain. TEE devices find matches and issues swaps.
- Closed source, so cannot research code. But docs are awesome.
- It sorts all orders by size and fills in order until it full. It matches (Sell, Buy), (Buy, Sell), (Sell, Sell), (Buy, Buy).

### Example in Solidity

https://github.com/PacktPublishing/Blockchain-Development-for-Finance-Projects/blob/master/Chapter%208/contracts/orderbook.sol

- There are 2 collections of Sells and Buys
- There is transaction which targets specific Sell or Buy
- So it assumes external seller or buyer observers Orderbook 
- And issues transaction for equal or greater amount to swap
- Owner can clean up all orders
- Only direct swap by oder id

### Serum DEX

https://docs.projectserum.com/appendix/philosophy
https://docs.projectserum.com/appendix/serum-core

- based on cranker, so external off chain agent or on chain program matches orders
- has queue inside

## What it is about?

First, what is exchanges of tokens across change?

It is based on protocol of token transfer, where A token is trusted(or proven) to be burn on A and minted on B.

Exchange, when A burns token x and mints y, and B mints x and burns y, and there is data sharing to agree on rate.

### DEX based liquidation

Sell the collateral on the DEX for the best price possible once the collateral passes some price point(collateral to borrow factor). Optimal is return back obtain at least the lent out coin(borrow principal) as return value from DEX.

External exchange is a trusted order book based exchange by trusted account id.

Fast it that there are up to few blocks allowed to liquidate.

Can be faster if untrusted, we will trust agent to burn amount.

For untrusted actors, more slow and complex schemas are needed.

Untrusted user must transfer borrow currency and buy collateral. There are [hash time locked swap][1](requires prove) and [reserver transfer via polkadot relay][2]. (they actually trust some third party consensus). And bridge some deposit first.

Important - assuming our parachain to be anemic - so it set states and allows  other to read that, not directly send message.

So that proffered account is of same level of trust as usual for now.

### CEX

https://github.com/connorwstein/exchange

### Links

[1]: https://research.csiro.au/blockchainpatterns/general-patterns/blockchain-payment-patterns/token-swap/


/// see for examples:
/// - https://github.com/galacticcouncil/Basilisk-node/blob/master/pallets/exchange/src/lib.rs
/// - https://github.com/Polkadex-Substrate/polkadex-aura-node/blob/master/pallets/polkadex/src/lib.rs
/// expected that failed exchanges are notified by events.


// orderdboox dex = amm + order book + matcher
// amm - to sell if price is better or to sell after failed ob sell with some slippages
// orderbook - can be fully  off chain (i did not found at all in hydra storage in their dex pallet - so store order only for one block and delete on finalization),  or on chain
// matchmaker  - can operate only if there is off chain component (so it matches only there orders which likely to success onto onchain)
// all 3 ob work like that (hydra, polkadex - closed source, only as per docs, and examples from solidity)
// matcher can be of different logic - who is served first? biggest ask/bid, fifo, etc...
// sell - i have exactly X and can receive approximately Y. and buy - i want exactly Y, can spend approximately X. so these are very symmetrical up to slippage.
// thats is by order book can be 2 collection of sell and buy by asset id, or it can be one collection of intentions (from, too) => amount, type.
//  9. i tried to find and read code of more on chain order books, like solana serum, but their codebase and patters are way complicated (but seems cool)
//  10. hydradx code is opinionated about matched order priority, not sure if that is good order.
// so for liqudations ordebook is very simple, just sells and buys, and any caller from on chain can take any of these if observers  good position. no matcher on chain.
// documing all this along the way.

// so we have simple on chain order book with external matcher (anybody can observer and take)



Dzmitry Lahoda:fire:  12:58 AM
i am still trying to make sanity of am doing, conclusion somebody should pay for each attempt to match to make order book safe(pass audit and do not stall production without paying for that)
either users call of chain and pay to match exact orders, with height degree of concurrency
or users allow to slash for each block they hold order active (that seems true for off chain and on chain matcher - this is equivalent of ` - but more optimal is that there is no concurrency and global matcher)
or each user should put stake before participate in order book and each user is limited by some know limit  (currency based permission), kind of centralized configuration.  for very trusted  accounts can have larger limit or allow trade without stake. but that is fully permissioned.
so amm or market price may help in terms of solving orders with high probability.
(edited)

1 reply
Today at 12:58 AMView thread

Dzmitry Lahoda:fire:  1:00 AM
1 which i am coding seem should still work for not too much concurrent bidders. plan is get it asap. and than contact hydra. so can ask them about xcmp we have and about this thing.

Dzmitry Lahoda:fire:  1:22 AM
seem 1 can be solved too. each tx is stored into list (payed for), and than during finalization highest/lowers price to take order is satisfied. kind of order book becomes "closed bid" auction with transaction fee as permission (so not first win, but highest bid above asked wins). so best way to trade would be trade as part of node of parachain seem (so kind of naturally goes to off chain design and on chain strategies, so can put strategy into TEE and allow hidden strategies from even us) UPDATE: So each order will take part of bid/ask order until it finished. Alternative is to fill in array and sort on finalization. Adding item is O(N), while sorting is N log N. No sure it is possible to predict weight, but at least these. But that is what Hydra does - it has (asset, asset) -> vec, here will be order-> vec  per on block (so that vector is not stored in hash). (edited) 


on chain order permissionless order book with order matcher:
all orders stored on chain
limit of orders to be know
if must not be infinite. either it will eat all block weight. so audit will not pass it through.
so some how we must limit how long order stays in order book or amount of orders.
solution, when order is created it asks either sell in this block or delete - weight paid for trx or putting order.  either i am ok holding order for some time, but burn some amount from mee as fee to be in queue, actual it can be equal to weight of adding order. order will be removed after some blocks amount passed. each block will slash (pay like if order was put again with Wight).
https://github.com/project-serum/serum-dex/blob/537a8c576f4006446b7e2515667ff5d186443282/dex/src/matching.rs#L42 - they have something like this. so Serum is different that solana calculates fees and there rent for storage. may be even fee for holding order.
this way can predict weight on initialize more or less.
it is ok for public byers. what about our lendng being seller? could vault be given composable native token to liquidate?
also weigh per block is not W but W log W for matching.
--------
permission based order book with order matcher:
have white list of trusted sellers, like auction.
where is hole? attack can be done via creating many loans on doge coin so that many of these liquaidated and making orderbook to stall block production for a while. solution could be collect liqudation amounts into batches.
actually we prevent all external sellers. but was about buyers ? (buy for this price or less?) it should be some kind of permissionless and may be live one block. so that buyer can bid again.
off chain designs seems assume they can hold whole OB in memory and fully permissioned.
----
on chain order book without matching:
order book can be very long
somebody offchain observers it, and puts precises orders to trade for specific items.
so there is no loop over orders.
---
hydra dx assumes that somebody observers chain and put precise orders. all trade in on finalization or removed. all traded on amm as final step. they even do not store orders. kind of observers should
polkadex promises such observers as close to chain as possible and never stopping to try - TEE
--
https://github.com/igormuba/DEX/blob/master/Class%2011/contracts/exchange.sol - found another solidi attempt (also freaking from book). improve on previous. they it seems like relies on fact that fees are estimated during call, kind you can burn a lot looping over orders,
--
design i am doing - no matcher.
so there is possibly infinite list of orders (so how much bad having too big history - will that pollute our chain?) probably not so much as order to not change from block to block - no transactions to record into chain. (not sure why hydra was afraid of that- they have matcher)
somebody off chain looks onto chain and places takes on orders. if unlucky, several bids will go to take same order. so they can always trying to take best. so burning through weigth (or if we return error weight is not burn?). kind of if return no error - pay for weight not success. if return error and error execution unpayed - we at least hope that quick short circuit check is fast. external matcher to much conflict so. but network is safe - audit can be passed may be.
not sure how polkadex solves several tee matchers (or they have only one TEE to manage order book?)  seems still can be attacked.