import { EventHandlerContext } from "@subsquid/substrate-processor";
import Big from "big.js";
import { Store } from "@subsquid/typeorm-store";
import { randomUUID } from "crypto";
import {
  PabloLiquidityAddedEvent,
  PabloLiquidityRemovedEvent,
  PabloPoolCreatedEvent,
  PabloPoolDeletedEvent,
  PabloSwappedEvent,
} from "../types/events";
import {
  get,
  getLatestPoolByPoolId,
  getOrCreate,
  storeCurrentLockedValue,
  storeHistoricalLockedValue,
  storeHistoricalVolume,
} from "../dbHelper";
import {
  Event,
  EventType,
  LockedSource,
  PabloPool,
  PabloPoolAsset,
  PabloTransaction,
} from "../model";
import { CurrencyPair, Fee } from "../types/v2402";
import { encodeAccount } from "../utils";

function createEvent(
  ctx: EventHandlerContext<Store, { event: true }>,
  who: string,
  eventType: EventType
) {
  return new Event({
    id: ctx.event.id,
    accountId: who,
    blockNumber: BigInt(ctx.block.height),
    timestamp: BigInt(new Date().valueOf()),
    eventType,
  });
}

function createPabloTransaction(
  event: Event,
  pool: PabloPool,
  spotPrice: string,
  baseAssetId: string,
  baseAssetAmount: bigint,
  quoteAssetId: string,
  quoteAssetAmount: bigint,
  fee?: string
) {
  return new PabloTransaction({
    id: randomUUID(),
    pool,
    spotPrice,
    baseAssetId,
    baseAssetAmount,
    quoteAssetId,
    quoteAssetAmount,
    fee: fee || "0.0",
    event,
  });
}

function createAsset(
  pool: PabloPool,
  assetId: string,
  ctx: EventHandlerContext<Store, { event: true }>,
  timestamp: bigint
) {
  return new PabloPoolAsset({
    id: createPoolAssetId(ctx.event.id, pool.poolId, assetId),
    assetId,
    pool,
    blockNumber: BigInt(ctx.block.height),
    totalLiquidity: BigInt(0),
    totalVolume: BigInt(0),
    calculatedTimestamp: timestamp,
  });
}

interface PoolCreatedEvent {
  owner: Uint8Array;
  poolId: bigint;
  assets: CurrencyPair;
}

function getPoolCreatedEvent(event: PabloPoolCreatedEvent): PoolCreatedEvent {
  const { owner, poolId, assets } = event.asV2402;
  return { owner, poolId, assets };
}

export async function processPoolCreatedEvent(
  ctx: EventHandlerContext<Store, { event: true }>,
  event: PabloPoolCreatedEvent
): Promise<void> {
  console.debug("processing PoolCreatedEvent", ctx.event.id);
  const poolCreatedEvt = getPoolCreatedEvent(event);
  const owner = encodeAccount(poolCreatedEvt.owner);
  const pool = await getOrCreate(ctx.store, PabloPool, ctx.event.id);
  // only set values if the owner was missing, i.e a new pool
  if (pool.owner == null) {
    const timestamp = BigInt(new Date().getTime());
    pool.id = ctx.event.id;
    pool.eventId = ctx.event.id;
    pool.owner = owner;
    pool.poolId = poolCreatedEvt.poolId;
    pool.baseAssetId = poolCreatedEvt.assets.base.toString();
    pool.quoteAssetId = poolCreatedEvt.assets.quote.toString();
    pool.transactionCount = 1;
    pool.totalLiquidity = "0.0";
    pool.totalVolume = "0.0";
    pool.totalFees = "0.0";
    pool.calculatedTimestamp = timestamp;
    pool.lpIssued = BigInt(0);
    pool.blockNumber = BigInt(ctx.block.height);

    let tx = await ctx.store.get(Event, ctx.event.id);
    if (tx != undefined) {
      console.error("Unexpected event in db", tx);
      throw new Error("Unexpected event in db");
    }

    const eventEntity = createEvent(ctx, owner, EventType.PABLO_POOL_CREATED);
    const pabloTransaction = createPabloTransaction(
      eventEntity,
      pool,
      // Following fields are irrelevant for CREATE_POOL
      "0",
      poolCreatedEvt.assets.base.toString(),
      BigInt(0),
      poolCreatedEvt.assets.quote.toString(),
      BigInt(0)
    );

    let quoteAsset = await get(
      ctx.store,
      PabloPoolAsset,
      createPoolAssetId(
        ctx.event.id,
        pool.poolId,
        poolCreatedEvt.assets.quote.toString()
      )
    );
    let baseAsset = await get(
      ctx.store,
      PabloPoolAsset,
      createPoolAssetId(
        ctx.event.id,
        pool.poolId,
        poolCreatedEvt.assets.base.toString()
      )
    );
    if (quoteAsset != undefined || baseAsset != undefined) {
      console.error("Unexpected assets for pool in db", quoteAsset, baseAsset);
      throw new Error("Unexpected assets found");
    }
    quoteAsset = createAsset(
      pool,
      poolCreatedEvt.assets.quote.toString(),
      ctx,
      timestamp
    );
    baseAsset = createAsset(
      pool,
      poolCreatedEvt.assets.base.toString(),
      ctx,
      timestamp
    );

    await ctx.store.save(pool);
    await ctx.store.save(baseAsset);
    await ctx.store.save(quoteAsset);
    await ctx.store.save(eventEntity);
    await ctx.store.save(pabloTransaction);
  }
}

export function createPoolAssetId(
  eventId: string,
  poolId: bigint,
  assetId: string
): string {
  return `${eventId}-${poolId}-${assetId}`;
}

interface LiquidityAddedEvent {
  who: Uint8Array;
  poolId: bigint;
  assets: [bigint, bigint][];
  mintedLp: bigint;
}

function getLiquidityAddedEvent(
  event: PabloLiquidityAddedEvent
): LiquidityAddedEvent {
  const { who, poolId, assets, mintedLp } = event.asV2402;
  return { who, poolId, assets, mintedLp };
}

export async function processLiquidityAddedEvent(
  ctx: EventHandlerContext<Store, { event: true }>,
  event: PabloLiquidityAddedEvent
): Promise<void> {
  console.debug("processing LiquidityAddedEvent", ctx.event.id);
  const liquidityAddedEvt = getLiquidityAddedEvent(event);
  const who = encodeAccount(liquidityAddedEvt.who);
  const pool = await getLatestPoolByPoolId(ctx.store, liquidityAddedEvt.poolId);

  const [base, quote] = liquidityAddedEvt.assets;
  const [baseId, baseAmount] = base;
  const [quoteId, quoteAmount] = quote;

  // only set values if the owner was missing, i.e a new pool
  if (pool !== undefined) {
    const timestamp = BigInt(new Date().getTime());
    pool.id = ctx.event.id;
    pool.eventId = ctx.event.id;
    pool.transactionCount += 1;
    pool.totalLiquidity = Big(pool.totalLiquidity)
      // multiplying by 2 to account for base amount being added
      .add(Big(quoteAmount.toString()).mul(2))
      .toString();
    pool.calculatedTimestamp = timestamp;
    pool.blockNumber = BigInt(ctx.block.height);
    pool.lpIssued += liquidityAddedEvt.mintedLp;

    const baseAsset = pool.poolAssets.find(
      (asset) => asset.assetId === baseId.toString()
    );
    if (baseAsset === undefined) {
      throw new Error("baseAsset not found");
    }
    baseAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      baseAsset.assetId
    );
    baseAsset.pool = pool;
    baseAsset.totalLiquidity += baseAmount;
    baseAsset.calculatedTimestamp = timestamp;
    baseAsset.blockNumber = BigInt(ctx.block.height);
    // find quoteAsset
    const quoteAsset = pool.poolAssets.find(
      (asset) => asset.assetId === pool.quoteAssetId
    );
    if (quoteAsset === undefined) {
      throw new Error("quoteAsset not found");
    }
    quoteAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      quoteAsset.assetId
    );
    quoteAsset.pool = pool;
    quoteAsset.totalLiquidity += quoteAmount;
    quoteAsset.calculatedTimestamp = timestamp;
    quoteAsset.blockNumber = BigInt(ctx.block.height);

    let tx = await ctx.store.get(Event, ctx.event.id);
    if (tx != undefined) {
      throw new Error("Unexpected event in db");
    }

    const eventEntity = createEvent(ctx, who, EventType.PABLO_LIQUIDITY_ADDED);

    const pabloTransaction = createPabloTransaction(
      eventEntity,
      pool,
      Big(baseAmount.toString()).div(Big(quoteAmount.toString())).toString(),
      baseAsset.assetId,
      baseAmount,
      quoteAsset.assetId,
      quoteAmount
    );

    await ctx.store.save(pool);
    await ctx.store.save(baseAsset);
    await ctx.store.save(quoteAsset);
    await ctx.store.save(eventEntity);
    await ctx.store.save(pabloTransaction);

    await storeHistoricalLockedValue(
      ctx,
      {
        [baseId.toString()]: baseAmount,
        [quoteId.toString()]: quoteAmount,
      },
      LockedSource.Pablo
    );

    await storeCurrentLockedValue(
      ctx,
      {
        [baseId.toString()]: baseAmount,
        [quoteId.toString()]: quoteAmount,
      },
      LockedSource.Pablo
    );
  } else {
    throw new Error("Pool not found");
  }
}

interface LiquidityRemovedEvent {
  who: Uint8Array;
  poolId: bigint;
  assets: [bigint, bigint][];
  totalIssuance: bigint;
}

function getLiquidityRemovedEvent(
  event: PabloLiquidityRemovedEvent
): LiquidityRemovedEvent {
  const { who, poolId, assets, totalIssuance } = event.asV2402;
  return { who, poolId, assets, totalIssuance };
}

export async function processLiquidityRemovedEvent(
  ctx: EventHandlerContext<Store, { event: true }>,
  event: PabloLiquidityRemovedEvent
): Promise<void> {
  console.debug("processing LiquidityAddedEvent", ctx.event.id);
  const liquidityRemovedEvt = getLiquidityRemovedEvent(event);
  const who = encodeAccount(liquidityRemovedEvt.who);
  const pool = await getLatestPoolByPoolId(
    ctx.store,
    liquidityRemovedEvt.poolId
  );
  const [base, quote] = liquidityRemovedEvt.assets;
  const [baseId, baseAmount] = base;
  const [quoteId, quoteAmount] = quote;

  // only set values if the owner was missing, i.e a new pool
  if (pool !== undefined) {
    const timestamp = BigInt(new Date().getTime());
    pool.id = ctx.event.id;
    pool.eventId = ctx.event.id;
    pool.transactionCount += 1;
    pool.totalLiquidity = Big(pool.totalLiquidity)
      // multiplying by 2 to account for base amount being removed
      .sub(Big(quoteAmount.toString()).mul(2))
      .toString();
    pool.calculatedTimestamp = timestamp;
    pool.blockNumber = BigInt(ctx.block.height);
    pool.lpIssued = BigInt(liquidityRemovedEvt.totalIssuance);

    // find baseAsset: Following is only valid for dual asset pools
    const baseAsset = pool.poolAssets.find(
      (asset) => asset.assetId === baseId.toString()
    );
    if (baseAsset === undefined) {
      throw new Error("baseAsset not found");
    }
    baseAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      baseAsset.assetId
    );
    baseAsset.pool = pool;
    baseAsset.totalLiquidity -= baseAmount;
    baseAsset.calculatedTimestamp = timestamp;
    baseAsset.blockNumber = BigInt(ctx.block.height);
    // find quoteAsset
    const quoteAsset = pool.poolAssets.find(
      (asset) => asset.assetId === pool.quoteAssetId
    );
    if (quoteAsset === undefined) {
      throw new Error("quoteAsset not found");
    }
    quoteAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      quoteAsset.assetId
    );
    quoteAsset.pool = pool;
    quoteAsset.totalLiquidity -= quoteAmount;
    quoteAsset.calculatedTimestamp = timestamp;
    quoteAsset.blockNumber = BigInt(ctx.block.height);

    let tx = await ctx.store.get(Event, ctx.event.id);
    if (tx != undefined) {
      throw new Error("Unexpected event in db");
    }

    const eventEntity = createEvent(
      ctx,
      who,
      EventType.PABLO_LIQUIDITY_REMOVED
    );
    const pabloTransaction = createPabloTransaction(
      eventEntity,
      pool,
      Big(baseAmount.toString()).div(Big(quoteAmount.toString())).toString(),
      baseAsset.assetId,
      baseAmount,
      pool.quoteAssetId,
      quoteAmount
    );

    await ctx.store.save(pool);
    await ctx.store.save(baseAsset);
    await ctx.store.save(quoteAsset);
    await ctx.store.save(eventEntity);
    await ctx.store.save(pabloTransaction);

    await storeHistoricalLockedValue(
      ctx,
      {
        [baseId.toString()]: -baseAmount,
        [quoteId.toString()]: -quoteAmount,
      },
      LockedSource.Pablo
    );

    await storeCurrentLockedValue(
      ctx,
      {
        [baseId.toString()]: -baseAmount,
        [quoteId.toString()]: -quoteAmount,
      },
      LockedSource.Pablo
    );
  } else {
    throw new Error("Pool not found");
  }
}

interface SwappedEvent {
  poolId: bigint;
  who: Uint8Array;
  baseAsset: string;
  quoteAsset: string;
  baseAmount: bigint;
  quoteAmount: bigint;
  fee: Fee;
}

function getSwappedEvent(event: PabloSwappedEvent): SwappedEvent {
  const { poolId, who, baseAsset, quoteAsset, baseAmount, quoteAmount, fee } =
    event.asV2402;
  return {
    poolId,
    who,
    baseAsset: baseAsset.toString(),
    quoteAsset: quoteAsset.toString(),
    baseAmount,
    quoteAmount,
    fee,
  };
}

export async function processSwappedEvent(
  ctx: EventHandlerContext<Store, { event: true }>,
  event: PabloSwappedEvent
): Promise<void> {
  console.debug("processing SwappedEvent", ctx.event.id);
  const swappedEvt = getSwappedEvent(event);
  const who = encodeAccount(swappedEvt.who);
  const pool = await getLatestPoolByPoolId(ctx.store, swappedEvt.poolId);
  // only set values if the owner was missing, i.e a new pool
  if (pool !== undefined) {
    const isReverse: boolean = pool.quoteAssetId !== swappedEvt.quoteAsset;
    const timestamp = BigInt(new Date().getTime());
    pool.id = ctx.event.id;
    pool.eventId = ctx.event.id;
    pool.transactionCount += 1;
    pool.calculatedTimestamp = timestamp;
    pool.blockNumber = BigInt(ctx.block.height);
    // find baseAsset: Following is only valid for dual asset pools
    const baseAsset = pool.poolAssets.find(
      (asset) => asset.assetId !== pool.quoteAssetId
    );
    if (baseAsset === undefined) {
      throw new Error("baseAsset not found");
    }
    // find quoteAsset
    const quoteAsset = pool.poolAssets.find(
      (asset) => asset.assetId === pool.quoteAssetId
    );
    if (quoteAsset === undefined) {
      throw new Error("quoteAsset not found");
    }
    const feesLeavingPool = swappedEvt.fee.fee - swappedEvt.fee.lpFee;
    const spotPrice = isReverse
      ? Big(swappedEvt.baseAmount.toString()).div(
          Big(swappedEvt.quoteAmount.toString())
        )
      : Big(swappedEvt.quoteAmount.toString()).div(
          Big(swappedEvt.baseAmount.toString())
        );
    if (isReverse) {
      console.debug("Reverse swap");
      // volume
      pool.totalVolume = Big(pool.totalVolume)
        .add(Big(swappedEvt.baseAmount.toString()))
        .toString();
      baseAsset.totalVolume += swappedEvt.quoteAmount;
      quoteAsset.totalVolume += swappedEvt.baseAmount;

      // for reverse exchange "default quote" (included as the base amount in the evt) amount leaves the pool
      baseAsset.totalLiquidity += swappedEvt.quoteAmount;
      quoteAsset.totalLiquidity -= swappedEvt.baseAmount;
      quoteAsset.totalLiquidity -= feesLeavingPool;
    } else {
      console.debug("Normal swap");
      // volume
      pool.totalVolume = Big(pool.totalVolume)
        .add(Big(swappedEvt.quoteAmount.toString()))
        .toString();
      baseAsset.totalVolume += swappedEvt.baseAmount;
      quoteAsset.totalVolume += swappedEvt.quoteAmount;

      // for normal exchange "default quote" amount gets into the pool
      baseAsset.totalLiquidity -= swappedEvt.baseAmount;
      baseAsset.totalLiquidity -= feesLeavingPool;
      quoteAsset.totalLiquidity += swappedEvt.quoteAmount;
    }
    // fee and liquidity
    pool.totalLiquidity = Big(pool.totalLiquidity)
      .sub(
        calculateFeeInQuoteAsset(
          spotPrice,
          quoteAsset.assetId,
          swappedEvt.fee.assetId.toString(),
          feesLeavingPool
        )
      )
      .toString();
    const fee = calculateFeeInQuoteAsset(
      spotPrice,
      quoteAsset.assetId,
      swappedEvt.fee.assetId.toString(),
      swappedEvt.fee.fee
    );
    pool.totalFees = Big(pool.totalFees).add(fee).toString();
    baseAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      baseAsset.assetId
    );
    baseAsset.pool = pool;
    baseAsset.calculatedTimestamp = timestamp;
    baseAsset.blockNumber = BigInt(ctx.block.height);
    quoteAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      quoteAsset.assetId
    );
    quoteAsset.pool = pool;
    quoteAsset.calculatedTimestamp = timestamp;
    quoteAsset.blockNumber = BigInt(ctx.block.height);

    let tx = await ctx.store.get(Event, ctx.event.id);
    if (tx != undefined) {
      throw new Error("Unexpected event in db");
    }

    const eventEntity = createEvent(ctx, who, EventType.PABLO_SWAPPED);
    const pabloTransaction = createPabloTransaction(
      eventEntity,
      pool,
      spotPrice.toString(),
      swappedEvt.baseAsset,
      swappedEvt.baseAmount,
      swappedEvt.quoteAsset,
      swappedEvt.quoteAmount,
      fee.toString()
    );

    await ctx.store.save(pool);
    await ctx.store.save(baseAsset);
    await ctx.store.save(quoteAsset);
    await ctx.store.save(eventEntity);
    await ctx.store.save(pabloTransaction);

    await storeHistoricalVolume(
      ctx,
      quoteAsset.assetId,
      swappedEvt.quoteAmount
    );
  } else {
    throw new Error("Pool not found");
  }
}

interface PoolDeletedEvent {
  poolId: bigint;
  assets: [bigint, bigint][];
}

function getPoolDeletedEvent(event: PabloPoolDeletedEvent): PoolDeletedEvent {
  const { poolId, assets } = event.asV2402;
  return { poolId, assets };
}

export async function processPoolDeletedEvent(
  ctx: EventHandlerContext<Store, { event: true }>,
  event: PabloPoolDeletedEvent
): Promise<void> {
  const poolDeletedEvent = getPoolDeletedEvent(event);
  const pool = await getLatestPoolByPoolId(ctx.store, poolDeletedEvent.poolId);

  const [base, quote] = poolDeletedEvent.assets;
  const [baseId, baseAmount] = base;
  const [quoteId, quoteAmount] = quote;

  // only set values if the owner was missing, i.e a new pool
  if (pool !== undefined) {
    const who = pool.owner;
    const timestamp = BigInt(new Date().getTime());
    pool.id = ctx.event.id;
    pool.eventId = ctx.event.id;
    pool.transactionCount += 1;
    pool.totalLiquidity = "0.0";
    pool.calculatedTimestamp = timestamp;
    pool.blockNumber = BigInt(ctx.block.height);

    const baseAsset = pool.poolAssets.find(
      (asset) => asset.assetId === baseId.toString()
    );
    if (baseAsset === undefined) {
      throw new Error("baseAsset not found");
    }
    baseAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      baseAsset.assetId
    );
    baseAsset.pool = pool;
    baseAsset.totalLiquidity -= baseAmount;
    baseAsset.calculatedTimestamp = timestamp;
    baseAsset.blockNumber = BigInt(ctx.block.height);
    // find quoteAsset
    const quoteAsset = pool.poolAssets.find(
      (asset) => asset.assetId === pool.quoteAssetId
    );
    if (quoteAsset === undefined) {
      throw new Error("quoteAsset not found");
    }
    quoteAsset.id = createPoolAssetId(
      ctx.event.id,
      pool.poolId,
      quoteAsset.assetId
    );
    quoteAsset.pool = pool;
    quoteAsset.totalLiquidity -= quoteAmount;
    quoteAsset.calculatedTimestamp = timestamp;
    quoteAsset.blockNumber = BigInt(ctx.block.height);

    let tx = await ctx.store.get(Event, ctx.event.id);
    if (tx != undefined) {
      throw new Error("Unexpected event in db");
    }

    const eventEntity = createEvent(ctx, who, EventType.PABLO_POOL_DELETED);
    const pabloTransaction = createPabloTransaction(
      eventEntity,
      pool,
      Big(baseAmount.toString()).div(Big(quoteAmount.toString())).toString(),
      pool.baseAssetId,
      baseAmount,
      pool.quoteAssetId,
      quoteAmount
    );

    await ctx.store.save(pool);
    await ctx.store.save(baseAsset);
    await ctx.store.save(quoteAsset);
    await ctx.store.save(eventEntity);
    await ctx.store.save(pabloTransaction);
  } else {
    throw new Error("Pool not found");
  }
}

function calculateFeeInQuoteAsset(
  spotPrice: Big,
  quoteAsset: string,
  feeAsset: string,
  fee: bigint
): Big {
  // calculate the quote amount based on the exchange rate if the fees are in the base asset
  return feeAsset === quoteAsset
    ? Big(fee.toString())
    : spotPrice.mul(fee.toString());
}
