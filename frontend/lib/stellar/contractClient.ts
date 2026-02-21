import type { SorobanContractId } from "./soroban";

export type ContractClientConfig = {
  contractId: SorobanContractId;
  rpcUrl: string;
};

export function createContractClient(_config: ContractClientConfig) {
  return {
    async ping(): Promise<string> {
      return "ok";
    },
  };
}
