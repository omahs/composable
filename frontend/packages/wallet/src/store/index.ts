import create from "zustand";
import { immer } from "zustand/middleware/immer";
import { devtools } from "zustand/middleware";

export type WalletStore = {};

export const useStore = create<WalletStore>()(
  immer(
    devtools((...a) => ({})
    )
  )
);
