import { AmountTokenDropdown } from "@/components/Organisms/Transfer/AmountTokenDropdown";
import { Header } from "@/components/Organisms/Transfer/Header";
import {
  gridContainerStyle,
  gridItemStyle,
} from "@/components/Organisms/Transfer/transfer-styles";
import { TransferExistentialDeposit } from "@/components/Organisms/Transfer/TransferExistentialDeposit";
import { TransferFeeDisplay } from "@/components/Organisms/Transfer/TransferFeeDisplay";
import { TransferKeepAliveSwitch } from "@/components/Organisms/Transfer/TransferKeepAliveSwitch";
import { TransferNetworkSelector } from "@/components/Organisms/Transfer/TransferNetworkSelector";
import { TransferRecipientDropdown } from "@/components/Organisms/Transfer/TransferRecipientDropdown";
import Default from "@/components/Templates/Default";
import { useTransfer } from "@/defi/polkadot/hooks/useTransfer";
import { getDestChainFee } from "@/defi/polkadot/pallets/Transfer";
import { useStore } from "@/stores/root";
import { Button, Grid, Typography } from "@mui/material";
import { NextPage } from "next";
import { useEffect } from "react";
import {
  subscribeDestinationMultiLocation,
  subscribeMultiAsset,
  subscribeTransferApiCall,
} from "@/stores/defi/polkadot/transfers/subscribers";
import { useSelectedAccount } from "@/defi/polkadot/hooks";
import { useAllParachainProviders } from "@/defi/polkadot/context/hooks";

const Transfers: NextPage = () => {
  const { amount, from, balance, transfer } = useTransfer();
  const allProviders = useAllParachainProviders();

  // For now all transactions are done with Picasso target
  // TODO: change this to get the chainApi from target (to) in store
  const fee = useStore((state) => state.transfers.fee);
  const tokenId = useStore((state) => state.transfers.selectedToken);
  const minValue = getDestChainFee(from, "picasso", tokenId).fee.plus(
    fee.partialFee
  );
  const feeTokenId = useStore((state) => state.transfers.getFeeToken(from));
  const selectedAccount = useSelectedAccount();
  const makeTransferCall = useStore(
    (state) => state.transfers.makeTransferCall
  );

  useEffect(() => {
    if (allProviders[from].parachainApi && selectedAccount) {
      let subscriptions: Array<Promise<() => void>> = [];
      subscriptions.push(
        subscribeDestinationMultiLocation(allProviders, selectedAccount.address)
      );
      subscriptions.push(subscribeMultiAsset(allProviders));

      subscriptions.push(subscribeTransferApiCall(allProviders));

      return () => {
        subscriptions.forEach((sub) => sub.then((call) => call()));
      };
    }
  }, [allProviders[from]?.apiStatus]);

  return (
    <Default>
      <Grid
        container
        sx={gridContainerStyle}
        maxWidth={1032}
        columns={10}
        direction="column"
        justifyContent="center"
      >
        <Grid item {...gridItemStyle("6rem")}>
          <Header />
        </Grid>
        <Grid item {...gridItemStyle()}>
          <TransferNetworkSelector />
        </Grid>
        <Grid item {...gridItemStyle()}>
          <AmountTokenDropdown />
        </Grid>
        <Grid item {...gridItemStyle("1.5rem")}>
          <TransferRecipientDropdown />
        </Grid>
        <Grid item {...gridItemStyle("1.5rem")}>
          <TransferFeeDisplay />
        </Grid>
        <Grid item {...gridItemStyle()}>
          <TransferKeepAliveSwitch />
        </Grid>
        <Grid item {...gridItemStyle()}>
          <TransferExistentialDeposit network={from} />
        </Grid>
        <Grid item {...gridItemStyle("1.5rem")}>
          <Button
            variant="contained"
            color="primary"
            disabled={
              amount.lte(0) || amount.gt(balance) || amount.lte(minValue)
            }
            fullWidth
            onClick={transfer}
          >
            <Typography variant="button">Transfer</Typography>
          </Button>
          {!amount.eq(0) && amount.lte(minValue) && (
            <Typography variant="caption" color="error.main">
              At least {minValue.toFormat(12)} {feeTokenId.symbol.toUpperCase()}{" "}
              will be spent for gas fees.
            </Typography>
          )}
        </Grid>
      </Grid>
    </Default>
  );
};

export default Transfers;
