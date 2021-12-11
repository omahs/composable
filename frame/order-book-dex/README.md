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

### Links

[1]: https://research.csiro.au/blockchainpatterns/general-patterns/blockchain-payment-patterns/token-swap/
