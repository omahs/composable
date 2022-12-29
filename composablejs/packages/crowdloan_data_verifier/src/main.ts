#!/usr/bin/env ts-node

import "@composable/types/augment-api";
import "@composable/types/augment-types";
import { getNewConnection } from "@composable/utils";
import { verifyCrowdloanData } from "@composable/crowdloan_data_verifier/handler";
import { ApiPromise } from "@polkadot/api";


const main = async () => {
  console.log("Crowdloan Pallet Verifier");

  console.log("Connecting...");
  // Establish connection to the node.
  const endpoint = "wss://rpc.composablenodes.tech";
  // const endpoint = process.env.ENDPOINT ?? "ws://127.0.0.1:9988";
  const { newClient } = await getNewConnection(endpoint);

  const api = newClient;

  await getLockedAccounts(api);
  // Here the actual magic happens
  // @ts-ignore
  // await verifyCrowdloanData(newClient);

  // Disconnecting from the node.
  console.debug("disconnecting...");
  await newClient.disconnect();
};

main()
  .then(() => {
    console.log("Crowdloan data verification finished!");
    process.exit(0);
  })
  .catch(err => {
    console.error(err.toString());
    process.exit(1);
  });

async function getLockedAccounts(api: ApiPromise) {
  const rawPalletData = await api.query.crowdloanRewards.associations.entries();

  console.log(rawPalletData.length);
  // console.log(
  //   (await api.query.crowdloanRewards.associations("5yqktZQVbVcTZ7Vq6JCt42zCkhtRudr2fybAo8Lwig1KWak8")).toHuman()
  // );
  // console.log((await api.query.balances.locks("5yqktZQVbVcTZ7Vq6JCt42zCkhtRudr2fybAo8Lwig1KWak8")).toHuman());

  const locks = (
    await api.queryMulti(
      rawPalletData.map(([k, _v]) => {
        // @ts-ignore
        return [api.query.balances.locks, k.toHuman()[0]];
      })
    )
  )
    .map((x, i) =>
      // @ts-ignore
      x.toHuman()?.length > 0
        ? [
          // @ts-ignore
          [[rawPalletData[i][0].toHuman()[0]], x.toJSON()]
        ]
        : []
    )
    .flat(1);

  const lockedAccounts = Object.keys(Object.fromEntries(locks));
  console.log(lockedAccounts);

  const unlockData = api.tx.sudo.sudo(api.tx.crowdloanRewards.unlockRewardsFor(lockedAccounts)).toHex();

  console.log(unlockData);

  console.log(
    // JSON.stringify(
    //   // @ts-ignore
    //   Object.fromEntries(locks)
    // ),
    locks.length
  );
}
