export type SorobanContractId = string;

export type SorobanEvent = {
  type: string;
  timestamp: number;
  data: Record<string, unknown>;
};
