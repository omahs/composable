


## Order book designs


### CEX

- off0chain design seems assume they can hold whole OB in memory 
- fully permissioned

https://github.com/connorwstein/exchange

https://www.chrisstucchio.com/blog/2012/hft_apology.html


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
