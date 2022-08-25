# Bonded Finance Integration Guide

[**Pallet Overview & Workflow**](../bonded-finance.md)

## Integration Status
| Dali | Picasso | Composable |
|------|---------|------------|
| Yes  | Yes     | No         |

## Setup / Configuration 
Bond offers are created and bond to by users. Only admins can cancel offers. Offers have three states:
Created, enabled and disabled. These states transition linearly. 

During the created state offers can be configured but no bonds can be made.
Once an offer is enabled other users can bond to it.
An offer is disabled once all bonds have been bought or the offer has been canceled via admin intervention.

Automatic state transition can occur when if bond offers have been bought.

## RPC & Data Retrieval
N/A

## Subsquid Data Retrieval
N/A

## Locally Consumed Types

### Types
* `NativeCurrency` - Numeric type used to represent protocol native tokens
* `Currency` - Numeric type used to represent some tokens
* `Vesting` - Function for managing vesting transfer of rewards
* `BondOfferID`- Numeric type used to uniquely identify Bond offers (`offer`)
* `Convert` - Function for converting `BalanceOf` for reward computation
* `AdminOrigin` - Function for ensuring the origin of admin calls
* `Weights` - Provider for extrinsic transaction weights

### Constants
* `PalletId` - Unique ID of the pallet
* `Stake` - The amount required to create a bond offer
* `MinReward` - The minimum reward for an offer

## Calculations & Sources of Values
N/A

## Extrinsic Parameter Sources
See the Bonded Finance pallet [extrinsics documentation](./extrinsics.md)

## Pricing Sources
N/A
