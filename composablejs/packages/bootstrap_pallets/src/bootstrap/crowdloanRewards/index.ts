import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import config from "@composable/bootstrap_pallets/constants/config.json";
import rewards from "@composable/bootstrap_pallets/constants/rewards.json";
import { PalletCrowdloanRewardsModelsRemoteAccount } from "@composable/types";
import { u128, u32 } from "@polkadot/types";
import { addFundsToCrowdloan, initialize, logger, sendAndWaitForSuccess, toChainUnits } from "../..";
import BigNumber from "bignumber.js";

function toPalletCrowdloanRewardsModelsRemoteAccount(
  api: ApiPromise,
  account: string,
  reward: string,
  vestingPeriod: string
): [PalletCrowdloanRewardsModelsRemoteAccount, u128, u32] {
  if (account.startsWith("0x")) {
    return [
      api.createType("PalletCrowdloanRewardsModelsRemoteAccount", {
        Ethereum: account
      }),
      api.createType("u128", toChainUnits(reward).toFixed(0)),
      api.createType("u32", vestingPeriod)
    ] as [PalletCrowdloanRewardsModelsRemoteAccount, u128, u32];
  } else {
    return [
      api.createType("PalletCrowdloanRewardsModelsRemoteAccount", {
        RelayChain: api.createType("AccountId32", account).toU8a()
      }),
      api.createType("u128", toChainUnits(reward).toFixed(0)),
      api.createType("u32", vestingPeriod)
    ] as [PalletCrowdloanRewardsModelsRemoteAccount, u128, u32];
  }
}

export async function bootstrapCrowdloanRewards(api: ApiPromise, walletSudo: KeyringPair): Promise<void> {
  const allRewards = Object.entries(rewards);

  const STEP = 1000;
  let amount = new BigNumber(0);
  const txCalls = [];
  for (let i = 0; i < allRewards.length; i += STEP) {
    const accountsOfBatch: [PalletCrowdloanRewardsModelsRemoteAccount, u128, u32][] = [];

    let accIndex = i;
    while (accIndex < allRewards.length && accIndex < STEP + i) {
      amount = amount.plus(allRewards[accIndex][1]);
      accountsOfBatch.push(
        toPalletCrowdloanRewardsModelsRemoteAccount(
          api,
          allRewards[accIndex][0],
          allRewards[accIndex][1],
          config.crowdloanRewards.vestingPeriod
        )
      );
      accIndex = accIndex + 1;
    }

    txCalls.push(api.tx.sudo.sudoUncheckedWeight(api.tx.crowdloanRewards.populate(accountsOfBatch), 1));
  }

  logger.info(`Populating Accounts: ${allRewards.length}`);
  await sendAndWaitForSuccess(api, walletSudo, api.events.utility.BatchCompleted.is, api.tx.utility.batch(txCalls));

  logger.info(`Adding Funds to Crowdloan: ${toChainUnits(amount).toFixed(0)}`);
  const totalPICA = amount.toFixed(0);
  await addFundsToCrowdloan(
    api,
    walletSudo,
    api.createType("u128", toChainUnits(totalPICA).toFixed(0)),
    config.crowdloanRewards.palletAccountId
  );

  logger.info(`Initializing Crowdloan Rewards`);
  await initialize(api, walletSudo);
}
