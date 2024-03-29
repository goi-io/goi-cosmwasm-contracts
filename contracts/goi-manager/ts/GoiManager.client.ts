/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee } from "@cosmjs/amino";
import { ExecuteMsg, ManagedStatus, Addr, AssetTypes, ExecuteMsg1, FeeType, Decimal, Positions, ManagedStatusChangedHookMsg, ManagedStatusUpdate, Member, ManagementFee, Fee, PlayerInfo, InstantiateMsg, QueryMsg, State } from "./GoiManager.types";
export interface GoiManagerReadOnlyInterface {
  contractAddress: string;
  getManagementInfo: () => Promise<GetManagementInfoResponse>;
  getManagedContract: ({
    contract
  }: {
    contract: Addr;
  }) => Promise<GetManagedContractResponse>;
}
export class GoiManagerQueryClient implements GoiManagerReadOnlyInterface {
  client: CosmWasmClient;
  contractAddress: string;

  constructor(client: CosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
    this.getManagementInfo = this.getManagementInfo.bind(this);
    this.getManagedContract = this.getManagedContract.bind(this);
  }

  getManagementInfo = async (): Promise<GetManagementInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_management_info: {}
    });
  };
  getManagedContract = async ({
    contract
  }: {
    contract: Addr;
  }): Promise<GetManagedContractResponse> => {
    return this.client.queryContractSmart(this.contractAddress, {
      get_managed_contract: {
        contract
      }
    });
  };
}
export interface GoiManagerInterface extends GoiManagerReadOnlyInterface {
  contractAddress: string;
  sender: string;
  managedStatusChangedHook: ({
    change
  }: {
    change: ManagedStatusUpdate;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  addManagedContract: ({
    contractAddress,
    contractType
  }: {
    contractAddress: Addr;
    contractType: AssetTypes;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  groupAdminHooks: ({
    groupAdminHooksMsg
  }: {
    groupAdminHooksMsg: ExecuteMsg;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  updateFees: ({
    add,
    remove
  }: {
    add?: ManagementFee[];
    remove?: number[];
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  addPlayersToTeam: ({
    players
  }: {
    players: PlayerInfo[];
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
  createNewAsset: ({
    assetType,
    user
  }: {
    assetType: AssetTypes;
    user: Addr;
  }, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]) => Promise<ExecuteResult>;
}
export class GoiManagerClient extends GoiManagerQueryClient implements GoiManagerInterface {
  client: SigningCosmWasmClient;
  sender: string;
  contractAddress: string;

  constructor(client: SigningCosmWasmClient, sender: string, contractAddress: string) {
    super(client, contractAddress);
    this.client = client;
    this.sender = sender;
    this.contractAddress = contractAddress;
    this.managedStatusChangedHook = this.managedStatusChangedHook.bind(this);
    this.addManagedContract = this.addManagedContract.bind(this);
    this.groupAdminHooks = this.groupAdminHooks.bind(this);
    this.updateFees = this.updateFees.bind(this);
    this.addPlayersToTeam = this.addPlayersToTeam.bind(this);
    this.createNewAsset = this.createNewAsset.bind(this);
  }

  managedStatusChangedHook = async ({
    change
  }: {
    change: ManagedStatusUpdate;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      managed_status_changed_hook: {
        change
      }
    }, fee, memo, funds);
  };
  addManagedContract = async ({
    contractAddress,
    contractType
  }: {
    contractAddress: Addr;
    contractType: AssetTypes;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      add_managed_contract: {
        contract_address: contractAddress,
        contract_type: contractType
      }
    }, fee, memo, funds);
  };
  groupAdminHooks = async ({
    groupAdminHooksMsg
  }: {
    groupAdminHooksMsg: ExecuteMsg;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      group_admin_hooks: {
        group_admin_hooks_msg: groupAdminHooksMsg
      }
    }, fee, memo, funds);
  };
  updateFees = async ({
    add,
    remove
  }: {
    add?: ManagementFee[];
    remove?: number[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      update_fees: {
        add,
        remove
      }
    }, fee, memo, funds);
  };
  addPlayersToTeam = async ({
    players
  }: {
    players: PlayerInfo[];
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      add_players_to_team: {
        players
      }
    }, fee, memo, funds);
  };
  createNewAsset = async ({
    assetType,
    user
  }: {
    assetType: AssetTypes;
    user: Addr;
  }, fee: number | StdFee | "auto" = "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> => {
    return await this.client.execute(this.sender, this.contractAddress, {
      create_new_asset: {
        asset_type: assetType,
        user
      }
    }, fee, memo, funds);
  };
}