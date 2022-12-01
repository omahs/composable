import { expect } from "chai";
import { ApiPromise } from "@polkadot/api";
import {
  ActiveUsers,
  GET_ACTIVE_USERS,
  OVERVIEW_STATS,
  OverviewStats
} from "@composable/utils/subsquid/apollo/queries";

import { client } from "@composable/utils/subsquid/apollo/apolloGraphql";
import { getNewConnection } from "@composable/utils/connectionHelper";
import { sendAndWaitForSuccess } from "@composable/utils/polkadotjs";
import { getDevWallets } from "@composable/utils/walletHelper";
import { KeyringPair } from "@polkadot/keyring/types";
import { mintAssetsToWallet } from "@composable/utils/mintingHelper";

describe("Picasso overview stats", function () {
  let api: ApiPromise;
  let sudoKey: KeyringPair, senderWallet: KeyringPair;

  before("Setting up the tests", async function () {
    const { newClient, newKeyring } = await getNewConnection();
    api = newClient;

    const { devWalletAlice, devWalletBob } = getDevWallets(newKeyring);
    sudoKey = devWalletAlice;
    senderWallet = devWalletBob.derive("/tests/assets/transferTestSenderWallet");
  });

  after("Closing the connection", async function () {
    await api.disconnect();
  });

  it("Correctly populates overview data", async function () {
    this.timeout(2 * 60 * 1000);
    const paraAsset = api.createType("u128", 4);
    const paraDest = senderWallet.derive("/tests/assets/transferTestReceiverWallet1").publicKey;
    const paraAmount = api.createType("Balance", 100000000000);
    const paraKeepAlive = api.createType("bool", true);

    await mintAssetsToWallet(api, senderWallet, sudoKey, [1, 4]);

    await sendAndWaitForSuccess(
      api,
      senderWallet,
      api.events.balances.Deposit.is,
      api.tx.assets.transfer(paraAsset, paraDest, paraAmount, paraKeepAlive)
    );

    const { data } = await client.query<OverviewStats>({ query: OVERVIEW_STATS });

    expect(data.overviewStats).to.have.keys([
      "__typename",
      "accountHoldersCount",
      "activeUsersCount",
      "totalValueLocked",
      "transactionsCount"
    ]);

    expect(data.overviewStats.accountHoldersCount).to.equal(3);
    expect(data.overviewStats.activeUsersCount).to.equal(3);
    expect(data.overviewStats.totalValueLocked).to.equal("0");
    expect(data.overviewStats.transactionsCount).not.to.equal(0);
  });

  it("Gets active user chart for last day", async function () {
    const { data: dayData } = await client.query<ActiveUsers>({ query: GET_ACTIVE_USERS, variables: { range: "day" } });
    const { activeUsers } = dayData;
    // Should have one entry per hour
    expect(activeUsers.length).to.equal(24);
    // Last hour should have some activity
    expect(activeUsers[activeUsers.length - 1].count).not.to.equal(0);
  });

  it("Gets active user chart for last week", async function () {
    const { data: weekData } = await client.query<ActiveUsers>({
      query: GET_ACTIVE_USERS,
      variables: { range: "week" }
    });
    const { activeUsers } = weekData;
    // Should have one entry per day
    expect(activeUsers.length).to.equal(7);
    // Last day should have some activity
    expect(activeUsers[activeUsers.length - 1].count).not.to.equal(0);
  });

  it("Gets active user chart for last month", async function () {
    const { data: monthData } = await client.query<ActiveUsers>({
      query: GET_ACTIVE_USERS,
      variables: { range: "month" }
    });
    const { activeUsers } = monthData;
    // Should have one entry per day
    expect(activeUsers.length).to.equal(30);
    // Last day should have some activity
    expect(activeUsers[activeUsers.length - 1].count).not.to.equal(0);
  });

  it("Gets active user chart for last year", async function () {
    const { data: yearData } = await client.query<ActiveUsers>({
      query: GET_ACTIVE_USERS,
      variables: { range: "year" }
    });
    const { activeUsers } = yearData;
    // Should have one entry per month
    expect(activeUsers.length).to.equal(12);
    // Last month should have some activity
    expect(activeUsers[activeUsers.length - 1].count).not.to.equal(0);
  });
});
