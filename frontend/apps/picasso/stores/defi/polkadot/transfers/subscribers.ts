import { TRANSFER_ASSET_LIST } from "@/defi/config";
import { getAssetOnChainId } from "@/defi/polkadot/Assets";
import { SubstrateNetworkId } from "@/defi/polkadot/types";
import { TokenOption } from "@/stores/defi/polkadot/transfers/transfers";
import { getDefaultToken } from "@/stores/defi/polkadot/transfers/utils";
import { useStore } from "@/stores/root";
import { ApiPromise } from "@polkadot/api";
import BigNumber from "bignumber.js";
import { fromChainIdUnit, toChainIdUnit } from "shared";
import { XcmVersionedMultiLocation } from "@polkadot/types/lookup";
import { SUBSTRATE_NETWORKS } from "@/defi/polkadot/Networks";
import {
  AcalaPrimitivesCurrencyCurrencyId,
  XcmVersionedMultiAsset,
} from "@acala-network/types/interfaces/types-lookup";
import { AllProviders } from "@/defi/polkadot/context/hooks";
import { ParachainApi, RelaychainApi } from "substrate-react";

function extractOptions(from: SubstrateNetworkId): TokenOption[] {
  const list = useStore.getState().substrateBalances.assets[from];
  return Object.values(list.assets).reduce((previousValue, currentValue) => {
    // no duplicates
    if (
      previousValue.find(
        (value: any) => value.tokenId === currentValue.meta.symbol
      )
    ) {
      return previousValue;
    }

    // calculate balance for token
    const isNative =
      "supportedNetwork" in currentValue.meta &&
      currentValue.meta.supportedNetwork[from] === 1;
    const balance = isNative
      ? useStore.getState().substrateBalances.assets[from].native.balance
      : currentValue.balance;

    // only include allowed assets
    if (
      !TRANSFER_ASSET_LIST[from].includes(
        currentValue.meta.symbol.toLowerCase()
      )
    ) {
      return previousValue;
    }

    return [
      ...previousValue,
      {
        tokenId: currentValue.meta.assetId,
        symbol: currentValue.meta.symbol,
        icon: currentValue.meta.icon,
        // disabled: balance.lte(0),
        // balance: balance,
      },
    ];
  }, [] as TokenOption[]);
}

function setOptions(options: TokenOption[]) {
  useStore.setState({
    ...useStore.getState(),
    transfers: {
      ...useStore.getState().transfers,
      tokenOptions: options,
    },
  });
}

export const subscribeTokenOptions = () => {
  return useStore.subscribe(
    (store) => store.transfers.networks.from,
    (from) => {
      const options = extractOptions(from);

      setOptions(options);
    },
    {
      fireImmediately: true,
    }
  );
};

export const subscribeDefaultTransferToken = () => {
  return useStore.subscribe(
    (store) => store.transfers.tokenOptions,
    (tokenOptions) => {
      const defaultToken = getDefaultToken(tokenOptions);

      useStore.setState({
        ...useStore.getState(),
        transfers: {
          ...useStore.getState().transfers,
          selectedToken: defaultToken,
        },
      });
    },
    {
      fireImmediately: true,
    }
  );
};

export const subscribeFeeItemEd = async (api: ApiPromise) => {
  return useStore.subscribe(
    (store) => ({
      feeItem: store.transfers.feeItem,
      sourceChain: store.transfers.networks.from,
    }),
    ({ feeItem, sourceChain }) => {
      const assetId = getAssetOnChainId("picasso", feeItem);
      if (!assetId) {
        return;
      }

      const ed = api.query.currencyFactory.assetEd(assetId);
      const existentialString = ed.toString();
      const existentialValue = fromChainIdUnit(
        new BigNumber(existentialString)
      );
      useStore.setState({
        ...useStore.getState(),
        transfers: {
          ...useStore.getState().transfers,
          feeItemEd: existentialValue.isNaN()
            ? new BigNumber(0)
            : existentialValue,
        },
      });
    },
    {
      fireImmediately: true,
      equalityFn: (a, b) =>
        a.feeItem === b.feeItem && a.sourceChain === b.sourceChain,
    }
  );
};

export const subscribeDestinationMultiLocation = async (
  allProviders: AllProviders,
  targetAddress: string
) => {
  return useStore.subscribe(
    (state) => ({
      targetChain: state.transfers.networks.to,
      sourceChain: state.transfers.networks.from,
      selectedAddress: state.transfers.recipients.selected,
    }),
    ({ sourceChain, targetChain, selectedAddress }, prev) => {
      const api = allProviders[sourceChain]?.parachainApi;
      if (!api) return;

      const targetChainId = SUBSTRATE_NETWORKS[targetChain].parachainId;
      const recipient = selectedAddress.length
        ? selectedAddress
        : targetAddress;
      // Kusama to Picasso uses XCM standard address
      if (sourceChain === "kusama") {
        useStore.getState().transfers.setDestinationMultiLocation(
          api.createType("XcmVersionedMultiLocation", {
            V0: api.createType("XcmV0MultiLocation", {
              X1: api.createType("XcmV0Junction", {
                Parachain: api.createType("u32", targetChainId),
              }),
            }),
          }) as XcmVersionedMultiLocation
        );
      }

      // Picasso to Kusama needs recipient in MultiLocation
      if (sourceChain === "picasso" && targetChain === "kusama" && recipient) {
        // Set destination. Should have 2 Junctions, first to parent and then to wallet
        useStore.getState().transfers.setDestinationMultiLocation(
          api.createType("XcmVersionedMultiLocation", {
            V0: api.createType("XcmV0MultiLocation", {
              X2: [
                api.createType("XcmV0Junction", "Parent"),
                api.createType("XcmV0Junction", {
                  AccountId32: {
                    network: api.createType("XcmV0JunctionNetworkId", "Any"),
                    id: api.createType("AccountId32", recipient),
                  },
                }),
              ],
            }),
          }) as XcmVersionedMultiLocation
        );
      }

      // Karura <> Picasso needs recipient in MultiDestLocation
      if ([sourceChain, targetChain].includes("karura") && recipient) {
        useStore.getState().transfers.setDestinationMultiLocation(
          api.createType("XcmVersionedMultiLocation", {
            V0: api.createType("XcmV0MultiLocation", {
              X3: [
                api.createType("XcmV0Junction", "Parent"),
                api.createType("XcmV0Junction", {
                  Parachain: api.createType("Compact<u32>", targetChainId),
                }),
                api.createType("XcmV0Junction", {
                  AccountId32: {
                    network: api.createType("XcmV0JunctionNetworkId", "Any"),
                    id: api.createType("AccountId32", recipient),
                  },
                }),
              ],
            }),
          }) as XcmVersionedMultiLocation
        );
      }
    },
    {
      fireImmediately: true,
      equalityFn: (
        { sourceChain, targetChain, selectedAddress },
        {
          sourceChain: $sourceChain,
          targetChain: $targetChain,
          selectedAddress: $selectedAddress,
        }
      ) => {
        return (
          sourceChain === $sourceChain &&
          targetChain === $targetChain &&
          selectedAddress === $selectedAddress
        );
      },
    }
  );
};

export const subscribeMultiAsset = async (allProviders: AllProviders) => {
  return useStore.subscribe(
    (store) => ({
      selectedToken: store.transfers.selectedToken,
      hasFeeItem: store.transfers.hasFeeItem,
      feeItem: store.transfers.feeItem,
      from: store.transfers.networks.from,
      to: store.transfers.networks.to,
      amount: store.transfers.amount,
      keepAlive: store.transfers.keepAlive,
      existentialDeposit: store.transfers.existentialDeposit,
    }),
    ({
      keepAlive,
      amount,
      existentialDeposit,
      selectedToken,
      hasFeeItem,
      feeItem,
      from,
      to,
    }) => {
      const api = allProviders[from].parachainApi;
      if (!api) return;
      // feeItem and hasFeeItem only populates if user selected a different token to pay
      const amountToTransfer = useStore
        .getState()
        .transfers.getTransferAmount(api);
      const feeItemId = getAssetOnChainId(from, feeItem);
      const selectedTokenId = getAssetOnChainId(from, selectedToken);

      if (from === "kusama" && to === "picasso") {
        useStore.getState().transfers.setTransferMultiAsset(
          api.createType("XcmVersionedMultiAssets", {
            V0: [
              api.createType("XcmV0MultiAsset", {
                ConcreteFungible: {
                  id: api.createType("XcmV0MultiLocation", "Null"),
                  amount: amountToTransfer,
                },
              }),
            ],
          }) as XcmVersionedMultiAsset
        );
      }

      if (from === "karura" && to === "picasso") {
        useStore.getState().transfers.setTransferMultiAsset(
          api.createType("AcalaPrimitivesCurrencyCurrencyId", {
            Token: api.createType(
              "AcalaPrimitivesCurrencyTokenSymbol",
              selectedToken.toUpperCase()
            ),
          }) as AcalaPrimitivesCurrencyCurrencyId
        );
      }

      if (from === "picasso") {
        if (!hasFeeItem) {
          useStore
            .getState()
            .transfers.setTransferMultiAsset([
              api.createType("u128", selectedTokenId),
              amountToTransfer,
            ]);
        } else {
          useStore.getState().transfers.setTransferMultiAsset([
            [api.createType("u128", selectedTokenId), amountToTransfer],
            [
              api.createType("u128", feeItemId),
              api.createType("u128", toChainIdUnit(1).toString()),
            ], // Asset to be used as fees, minFee should be calculated.
          ]);
        }
      }
    },
    {
      fireImmediately: true,
      equalityFn: (a, b) => {
        // @ts-ignore
        return Object.keys(a).every((key: string) => a[key] === b[key]);
      },
    }
  );
};

export const subscribeTransferApiCall = async (allProviders: AllProviders) => {
  return useStore.subscribe(
    (store) => ({
      from: store.transfers.networks.from,
      to: store.transfers.networks.to,
      hasFeeItem: store.transfers.hasFeeItem,
      selectedToken: store.transfers.selectedToken,
      amount: store.transfers.amount,
    }),
    ({ from, to, hasFeeItem }) => {
      const api = allProviders[from].parachainApi;
      if (!api) return;
      const set = useStore.getState().transfers.setTransferExtrinsic;
      if (from === "kusama" && to === "picasso") {
        try {
          set(api.tx.xcmPallet.reserveTransferAssets);
        } catch (e) {
          console.log("could not create API: xcmPallet not ready");
        }
      }

      if (from === "karura" && to === "picasso") {
        set(api.tx.xTokens.transfer);
      }

      // Both Karura and Kusama as targetChain
      if (from === "picasso") {
        try {
          set(
            !hasFeeItem
              ? api.tx.xTokens.transfer
              : api.tx.xTokens.transferMulticurrencies
          );
        } catch (e) {
          console.log("Could not create API: xTokens not ready.", e);
        }
      }
    },
    {
      fireImmediately: true,
      equalityFn: (a, b) =>
        a.from === b.from && a.to === b.to && a.hasFeeItem === b.hasFeeItem,
    }
  );
};
