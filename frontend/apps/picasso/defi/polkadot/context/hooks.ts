import {
  ParachainApi,
  RelaychainApi,
  useDotSamaContext,
} from "substrate-react";

export type AllProviders = {
  kusama: RelaychainApi;
  polkadot: RelaychainApi;
  karura: ParachainApi;
  picasso: ParachainApi;
};
export const useAllParachainProviders: () => AllProviders = () => {
  const { parachainProviders, relaychainProviders } = useDotSamaContext();
  return {
    ...parachainProviders,
    ...relaychainProviders,
  };
};
