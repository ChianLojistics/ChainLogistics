import type { SorobanContractId } from "./soroban";

export type ContractClientConfig = {
  contractId: SorobanContractId;
  rpcUrl: string;
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function createContractClient(_config: ContractClientConfig) {
  return {
    async ping(): Promise<string> {
      return "ok";
    },
  };
}
