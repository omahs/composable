import create from "zustand";
import { AllSlices } from "./types";
import { immer } from "zustand/middleware/immer";
import { devtools, subscribeWithSelector } from "zustand/middleware";
import createLiquiditySlice from "./liquidity/slice";
import createPoolStatsSlice from "./poolStats/slice";
import createRemoveLiquiditySlice from "./removeLiquidity/slice";
import createPoolSlice from "./createPool/slice";
import createSwapsSlice from "./swaps/slice";
import createAssetsSlice from "./tokens/slice";

const useStore = create<AllSlices>()(
  subscribeWithSelector(
    immer(
      devtools((...a) => ({
        ...createAssetsSlice(...a),
        ...createSwapsSlice(...a),
        ...createLiquiditySlice(...a),
        ...createRemoveLiquiditySlice(...a),
        ...createPoolSlice(...a),
        ...createPoolStatsSlice(...a)
      }))
    )
  )
);

export default useStore;
