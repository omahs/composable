import { ComponentStory } from "@storybook/react";
import { PoolUnstakeForm } from "pablo/components/Organisms";
import BigNumber from "bignumber.js";
import { Asset } from "shared";

export default {
  title: "organisms/PoolDetails/PoolUnstakeForm",
  component: PoolUnstakeForm,
};

const Template: ComponentStory<typeof PoolUnstakeForm> = (args) => (
  <PoolUnstakeForm {...args} />
);

export const Default = Template.bind({});
Default.args = {
  pool: {
    kind: "dualAssetConstantPool",
    poolId: new BigNumber("0"),
    config: {
      lpToken: 106,
      owner: "0abcd",
      assetsWeights: {
        1: 0,
        4: 0,
      },
      assets: [
        new Asset("", "", "", "pica", undefined),
        new Asset("", "", "", "ksm", undefined),
      ],
      feeConfig: {
        feeRate: 0,
        ownerFeeRate: 0,
        protocolFeeRate: 0,
      },
    },
  },
};
