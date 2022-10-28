import { ApiPromise } from "@polkadot/api";
import { Executor, getSigner } from "substrate-react";
import { u128 } from "@polkadot/types-codec";
import { AnyComponentMap, EnqueueSnackbar } from "notistack";
import { Assets } from "@/defi/polkadot/Assets";
import { APP_NAME } from "@/defi/polkadot/constants";
import { toChainIdUnit } from "shared";
import { CurrencyId } from "defi-interfaces";
import { XcmVersionedMultiLocation } from "@polkadot/types/lookup";
import BigNumber from "bignumber.js";
import { SUBSTRATE_NETWORKS } from "@/defi/polkadot/Networks";
import { AssetId } from "@/defi/polkadot/types";

export type TransferHandlerArgs = {
  api: ApiPromise;
  targetChain: number | 0;
  targetAccount: string;
  amount: u128;
  executor: Executor;
  enqueueSnackbar: EnqueueSnackbar<AnyComponentMap>;
  signerAddress: string;
  hasFeeItem: boolean;
  feeItemId: number | null;
  weight: BigNumber;
  token: AssetId;
};

export function availableTargetNetwork(
  network: string,
  selectedNetwork: string
) {
  switch (selectedNetwork) {
    case "kusama":
      return network === "picasso";
    case "picasso":
      return network === "kusama" || network === "karura";
    case "karura":
      return network === "picasso";
  }
}
