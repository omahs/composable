import { expect } from "chai";
import { ApiPromise } from "@polkadot/api";
import { OVERVIEW_STATS, OverviewStats } from "@composable/utils/subsquid/apollo/queries";

import { client } from "@composable/utils/subsquid/apollo/apolloGraphql";
import { getNewConnection } from "@composable/utils/connectionHelper";
import { sendAndWaitForSuccess } from "@composable/utils/polkadotjs";
import { getDevWallets } from "@composable/utils/walletHelper";
import { KeyringPair } from "@polkadot/keyring/types";
import { mintAssetsToWallet } from "@composable/utils/mintingHelper";

describe("Picasso overview stats", () => {
  let api: ApiPromise;
  let sudoKey: KeyringPair, senderWallet: KeyringPair;

  before("Setting up the tests", async function () {
    this.timeout(60 * 1000);
    const { newClient, newKeyring } = await getNewConnection();
    api = newClient;

    const { devWalletAlice, devWalletBob } = getDevWallets(newKeyring);
    sudoKey = devWalletAlice;
    senderWallet = devWalletBob.derive("/tests/assets/transferTestSenderWallet");
  });

  before(async function () {
    this.timeout(2 * 60 * 1000);
  });

  after("Closing the connection", async function () {
    await api.disconnect();
  });

  it("works", async () => {
    const { data: dataBefore } = await client.query<OverviewStats>({ query: OVERVIEW_STATS });

    console.log(dataBefore);

    expect(dataBefore.overviewStats).to.have.keys([
      "__typename",
      "accountHoldersCount",
      "activeUsersCount",
      "totalValueLocked",
      "transactionsCount"
    ]);

    expect(dataBefore.overviewStats.accountHoldersCount).to.equal(0);
    expect(dataBefore.overviewStats.activeUsersCount).to.equal(0);
    expect(dataBefore.overviewStats.totalValueLocked).to.equal("0");
    expect(dataBefore.overviewStats.transactionsCount).to.equal(0);

    const paraAsset = api.createType("u128", 4);
    const paraDest = senderWallet.derive("/tests/assets/transferTestReceiverWallet1").publicKey;
    const paraAmount = api.createType("Balance", 100000000000);
    const paraKeepAlive = api.createType("bool", true);

    await mintAssetsToWallet(api, sudoKey, sudoKey, [1]);
    await mintAssetsToWallet(api, senderWallet, sudoKey, [1, 4]);

    await sendAndWaitForSuccess(
      api,
      senderWallet,
      api.events.balances.Deposit.is,
      api.tx.assets.transfer(paraAsset, paraDest, paraAmount, paraKeepAlive)
    );

    const { data: dataAfter } = await client.query<OverviewStats>({ query: OVERVIEW_STATS });

    console.log(dataAfter);

    expect(dataBefore.overviewStats).to.have.keys([
      "__typename",
      "accountHoldersCount",
      "activeUsersCount",
      "totalValueLocked",
      "transactionsCount"
    ]);

    expect(dataAfter.overviewStats.accountHoldersCount).to.equal(2);
    expect(dataAfter.overviewStats.activeUsersCount).to.equal(2);
    expect(dataAfter.overviewStats.totalValueLocked).to.equal("0");
    expect(dataAfter.overviewStats.transactionsCount).to.equal(2);
  });
});
