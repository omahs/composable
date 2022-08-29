import { BondOffer, BondPrincipalAsset } from "@/defi/types";
import { getBondPrincipalAsset } from "@/defi/utils";
import useStore from "@/store/useStore";
import { useMemo } from "react";
import { useAllLpTokenRewardingPools } from "../pools/useAllLpTokenRewardingPools";

export default function useBondOfferPrincipalAsset(bondOffer: BondOffer): BondPrincipalAsset {
    const {
        supportedAssets
    } = useStore();
    const lpRewardingPools = useAllLpTokenRewardingPools();

    return useMemo(() => {
        return getBondPrincipalAsset(bondOffer, supportedAssets, lpRewardingPools);
    }, [bondOffer, supportedAssets, lpRewardingPools])
}