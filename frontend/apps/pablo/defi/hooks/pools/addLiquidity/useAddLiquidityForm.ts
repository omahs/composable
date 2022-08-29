import { AssetId } from "@/defi/polkadot/types";
import {
  setSelection,
  useAddLiquiditySlice,
} from "@/store/addLiquidity/addLiquidity.slice";
import { DEFAULT_NETWORK_ID } from "@/defi/utils/constants";
import BigNumber from "bignumber.js";
import { useState, useMemo, useEffect, useCallback } from "react";
import { useParachainApi, useSelectedAccount } from "substrate-react";
import { useAssetBalance } from "../../../../store/assets/hooks";
import { fetchSpotPrice, fromChainUnits, toChainUnits } from "@/defi/utils";
import { useAsset } from "@/defi/hooks/assets/useAsset";
import { useFilteredAssetListDropdownOptions } from "@/defi/hooks/assets/useFilteredAssetListDropdownOptions";
import { useLiquidityByPool } from "../useLiquidityByPool";

export const useAddLiquidityForm = () => {
  const [valid, setValid] = useState<boolean>(false);
  const { parachainApi } = useParachainApi(DEFAULT_NETWORK_ID);
  const selectedAccount = useSelectedAccount(DEFAULT_NETWORK_ID);

  const {
    ui: { assetOne, assetTwo, assetOneAmount, assetTwoAmount },
    pool,
    findPoolManually,
  } = useAddLiquiditySlice();

  const [spotPrice, setSpotPrice] = useState(new BigNumber(0));
  useEffect(() => {
    if (parachainApi && pool) {
      let pair = {
        base: assetOne,
        quote: assetTwo,
      };

      fetchSpotPrice(parachainApi, pair, pool.poolId)
        .then(setSpotPrice)
        .catch(console.error);
    }
  }, [parachainApi, pool, assetOne, assetTwo]);

  const _assetOne = useAsset(assetOne);
  const _assetTwo = useAsset(assetTwo);

  useEffect(() => {
    return function () {
      setSelection({
        assetOneAmount: new BigNumber(0),
        assetTwoAmount: new BigNumber(0),
      })
    }
  }, [])

  const {
    tokenAmounts: { baseAmount, quoteAmount },
  } = useLiquidityByPool(pool);

  const assetList1 = useFilteredAssetListDropdownOptions(assetTwo);
  const assetList2 = useFilteredAssetListDropdownOptions(assetOne);

  const balanceOne = useAssetBalance(DEFAULT_NETWORK_ID, assetOne);
  const balanceTwo = useAssetBalance(DEFAULT_NETWORK_ID, assetTwo);

  const setAmount =
    (key: "assetOneAmount" | "assetTwoAmount") => (v: BigNumber) => {
      setSelection({ [key]: v });
    };

  const setToken = (key: "assetOne" | "assetTwo") => (v: AssetId) => {
    setSelection({ [key]: v });
  };

  const isValidToken1 = assetOne != "none";
  const isValidToken2 = assetTwo != "none";

  const needToSelectToken = () => {
    return !isValidToken1 && !isValidToken2;
  };

  const invalidTokenPair = () => {
    return (
      (!isValidToken1 && isValidToken2) || (isValidToken1 && !isValidToken2)
    );
  };

  const canSupply = useCallback(() => {
    return assetOneAmount.lte(balanceOne) && assetTwoAmount.lte(balanceTwo);
  }, [assetOneAmount, assetTwoAmount, balanceOne, balanceTwo]);

  useEffect(() => {
    setValid(true);
    assetOne == "none" && setValid(false);
    assetTwo == "none" && setValid(false);

    new BigNumber(0).eq(assetOneAmount) && setValid(false);
    new BigNumber(0).eq(assetTwoAmount) && setValid(false);

    balanceOne.lt(assetOneAmount) && setValid(false);
    balanceTwo.lt(assetTwoAmount) && setValid(false);

    pool && pool.poolId === -1 && setValid(false);
  }, [
    assetOne,
    assetTwo,
    assetOneAmount,
    assetTwoAmount,
    balanceOne,
    balanceTwo,
    pool,
  ]);

  const share = useMemo(() => {
    let netAum = new BigNumber(baseAmount).plus(quoteAmount);
    let netUser = new BigNumber(assetOneAmount).plus(assetTwoAmount);

    if (netAum.eq(0)) {
      return new BigNumber(100);
    } else {
      return new BigNumber(netUser)
        .div(new BigNumber(netAum).plus(netUser))
        .times(100);
    }
  }, [baseAmount, quoteAmount, assetOneAmount, assetTwoAmount]);

  const [lpReceiveAmount, setLpReceiveAmount] = useState(new BigNumber(0));

  useEffect(() => {
    if (
      parachainApi &&
      assetOne !== "none" &&
      assetTwo !== "none" &&
      pool &&
      selectedAccount
    ) {
      let isReverse = pool.pair.base.toString() !== assetOne;
      const bnBase = toChainUnits(isReverse ? assetTwoAmount : assetOneAmount);
      const bnQuote = toChainUnits(isReverse ? assetOneAmount : assetTwoAmount);

      if (bnBase.gte(0) && bnQuote.gte(0)) {
        
        let b = isReverse ? pool.pair.quote.toString() : pool.pair.base.toString();
        let q = isReverse ? pool.pair.base.toString() : pool.pair.quote.toString();

        parachainApi.rpc.pablo
          .simulateAddLiquidity(
            parachainApi.createType("AccountId32", selectedAccount.address),
            parachainApi.createType("PalletPabloPoolId", pool.poolId),
            parachainApi.createType(
              "BTreeMap<SafeRpcWrapper, SafeRpcWrapper>",
              {
                [b]: bnBase.toString(),
                [q]: bnQuote.toString(),
              }
            )
          )
          .then((expectedLP: any) => {
            setLpReceiveAmount(fromChainUnits(expectedLP.toString()));
          })
          .catch((err: any) => {
            console.log(err);
          });
      }
    }
  }, [parachainApi, assetOneAmount, assetTwoAmount, assetOne, assetTwo, pool, selectedAccount]);

  return {
    assetOne: _assetOne,
    assetTwo: _assetTwo,
    balanceOne,
    balanceTwo,
    assetOneAmount,
    assetTwoAmount,
    assetList1,
    assetList2,
    share,
    lpReceiveAmount,
    valid,
    isValidToken1,
    isValidToken2,
    setValid,
    setToken,
    setAmount,
    needToSelectToken,
    invalidTokenPair,
    canSupply,
    findPoolManually,
    spotPrice,
    pool,
  };
};