import { useAllParachainProviders } from "@/defi/polkadot/context/hooks";
import { useSelectedAccount } from "@/defi/polkadot/hooks/index";
import { SUBSTRATE_NETWORKS } from "@/defi/polkadot/Networks";
import { useStore } from "@/stores/root";
import BigNumber from "bignumber.js";
import { useSnackbar } from "notistack";
import { getSigner, useExecutor } from "substrate-react";
import { APP_NAME } from "../constants";

export const useTransfer = () => {
  const allProviders = useAllParachainProviders();
  const from = useStore((state) => state.transfers.networks.from);
  const fromProvider = allProviders[from];
  const to = useStore((state) => state.transfers.networks.to);
  const toProvider = allProviders[to];
  const { enqueueSnackbar } = useSnackbar();
  const selectedRecipient = useStore(
    (state) => state.transfers.recipients.selected
  );
  const { hasFeeItem, feeItem } = useStore(({ transfers }) => transfers);
  const amount = useStore((state) => state.transfers.amount);
  const setAmount = useStore((state) => state.transfers.updateAmount);
  const account = useSelectedAccount();
  const providers = useAllParachainProviders();
  const executor = useExecutor();
  const getBalance = useStore(
    (state) => state.transfers.getTransferTokenBalance
  );
  const makeTransferCall = useStore(
    (state) => state.transfers.makeTransferCall
  );

  const TARGET_ACCOUNT_ADDRESS = selectedRecipient.length
    ? selectedRecipient
    : account?.address;

  const transfer = async () => {
    const api = providers[from].parachainApi;

    if (!api || !executor || !account || (hasFeeItem && feeItem.length === 0)) {
      console.error("No API or Executor or account", {
        api,
        executor,
        account,
      });
      return;
    }

    const signerAddress = account.address;
    const call = makeTransferCall(api, TARGET_ACCOUNT_ADDRESS);
    const signer = await getSigner(APP_NAME, signerAddress);
    if (!call) {
      console.error("Unknown error occurred building transfer call");
      return;
    }

    await executor.execute(
      call,
      signerAddress,
      api,
      signer,
      (txHash) => {
        enqueueSnackbar("Transfer executed", {
          persist: true,
          description: `Transaction hash: ${txHash}`,
          variant: "info",
          isCloseable: true,
          url: SUBSTRATE_NETWORKS.picasso.subscanUrl + txHash,
        });
      },
      (txHash) => {
        enqueueSnackbar("Transfer executed successfully.", {
          persist: true,
          variant: "success",
          isCloseable: true,
          url: SUBSTRATE_NETWORKS.picasso.subscanUrl + txHash,
        });
      },
      (err) => {
        enqueueSnackbar("Transfer failed", {
          persist: true,
          description: `Error: ${err}`,
          variant: "error",
          isCloseable: true,
        });
      }
    );

    // clear amount after
    setAmount(new BigNumber(0));
  };

  return {
    transfer,
    amount,
    from,
    to,
    balance: getBalance(),
    account,
    fromProvider,
    toProvider,
    TARGET_ACCOUNT_ADDRESS,
  };
};
