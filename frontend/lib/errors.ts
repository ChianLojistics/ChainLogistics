export type ErrorCategory =
  | "network"
  | "contract"
  | "wallet"
  | "validation"
  | "authentication"
  | "business_logic"
  | "system"
  | "user"
  | "unknown";

export type ClassifiedError = {
  category: ErrorCategory;
  title: string;
  message: string;
  recoverySteps: string[];
  retryable: boolean;
  original: unknown;
};

const NETWORK_PATTERNS = [
  "network",
  "fetch",
  "timeout",
  "timed out",
  "econnrefused",
  "econnreset",
  "enotfound",
  "dns",
  "offline",
  "internet",
  "connection",
  "503",
  "502",
  "504",
  "429",
  "rate limit",
] as const;

const WALLET_PATTERNS = [
  "wallet",
  "freighter",
  "sign",
  "rejected",
  "denied",
  "cancelled",
  "canceled",
  "user declined",
  "not installed",
  "unlocked",
  "popup",
  "account",
  "WalletError",
] as const;

const CONTRACT_PATTERNS = [
  "contract",
  "soroban",
  "invoke",
  "simulation",
  "wasm",
  "host function",
  "insufficient balance",
  "tx_failed",
  "op_",
  "transaction failed",
] as const;

const VALIDATION_PATTERNS = [
  "invalid",
  "required",
  "must be",
  "cannot be",
  "too long",
  "too short",
  "format",
  "expected",
  "missing",
  "empty",
] as const;

const AUTHENTICATION_PATTERNS = [
  "unauthorized",
  "forbidden",
  "login",
  "authentication",
  "session",
  "permission",
  "credentials",
  "401",
  "403",
] as const;

const BUSINESS_LOGIC_PATTERNS = [
  "workflow",
  "conflict",
  "already exists",
  "not allowed",
  "business rule",
  "constraint",
] as const;

const SYSTEM_PATTERNS = [
  "system",
  "browser",
  "performance",
  "resource",
  "memory",
  "out of memory",
  "compatibility",
  "internal server error",
  "500",
] as const;

function extractMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "An unexpected error occurred.";
}

function extractName(error: unknown): string {
  if (error instanceof Error) return error.name;
  return "";
}

function matchesAny(text: string, patterns: readonly string[]): boolean {
  const lower = text.toLowerCase();
  return patterns.some((p) => lower.includes(p));
}

function classifyCategory(error: unknown): ErrorCategory {
  const message = extractMessage(error);
  const name = extractName(error);
  const combined = `${name} ${message}`;

  if (name === "WalletError" || matchesAny(combined, WALLET_PATTERNS)) {
    return "wallet";
  }
  if (matchesAny(combined, CONTRACT_PATTERNS)) {
    return "contract";
  }
  if (matchesAny(combined, NETWORK_PATTERNS)) {
    return "network";
  }
  if (matchesAny(combined, VALIDATION_PATTERNS)) {
    return "validation";
  }
  if (matchesAny(combined, AUTHENTICATION_PATTERNS)) {
    return "authentication";
  }
  if (matchesAny(combined, BUSINESS_LOGIC_PATTERNS)) {
    return "business_logic";
  }
  if (matchesAny(combined, SYSTEM_PATTERNS)) {
    return "system";
  }

  return "unknown";
}

const CATEGORY_META: Record<
  ErrorCategory,
  { title: string; defaultMessage: string; recoverySteps: string[]; retryable: boolean }
> = {
  network: {
    title: "Network Error",
    defaultMessage: "Unable to reach the server. Please check your internet connection.",
    recoverySteps: [
      "Check your internet connection.",
      "Try again in a few moments.",
      "If the problem persists, the server may be temporarily unavailable.",
    ],
    retryable: true,
  },
  contract: {
    title: "Contract Error",
    defaultMessage: "The smart contract operation could not be completed.",
    recoverySteps: [
      "Verify you have sufficient funds for this transaction.",
      "Check that the contract parameters are correct.",
      "Try the operation again.",
    ],
    retryable: true,
  },
  wallet: {
    title: "Wallet Error",
    defaultMessage: "There was a problem with your wallet.",
    recoverySteps: [
      "Make sure Freighter is installed and unlocked.",
      "Check that your browser did not block the wallet popup.",
      "Reconnect your wallet and try again.",
    ],
    retryable: true,
  },
  validation: {
    title: "Validation Error",
    defaultMessage: "Please check your input and try again.",
    recoverySteps: [
      "Review the form fields for errors.",
      "Ensure all required fields are filled in.",
    ],
    retryable: false,
  },
  authentication: {
    title: "Authentication Error",
    defaultMessage: "There was a problem with your session or permissions.",
    recoverySteps: [
      "Please log in again.",
      "Ensure you have the correct permissions for this action.",
    ],
    retryable: false,
  },
  business_logic: {
    title: "Operation Failed",
    defaultMessage: "The requested operation cannot be completed due to a business rule conflict.",
    recoverySteps: [
      "Review the action you are trying to perform.",
      "Ensure the data you are submitting does not conflict with existing records.",
    ],
    retryable: false,
  },
  system: {
    title: "System Error",
    defaultMessage: "A system or platform error occurred.",
    recoverySteps: [
      "Refresh the page and try again.",
      "Check if your browser is up to date.",
      "Contact support if the issue persists.",
    ],
    retryable: true,
  },
  user: {
    title: "Action Required",
    defaultMessage: "Please complete the required action to proceed.",
    recoverySteps: [],
    retryable: false,
  },
  unknown: {
    title: "Something Went Wrong",
    defaultMessage: "An unexpected error occurred. Please try again.",
    recoverySteps: [
      "Refresh the page and try again.",
      "If the problem persists, contact support.",
    ],
    retryable: true,
  },
};

export function classifyError(error: unknown): ClassifiedError {
  const category = classifyCategory(error);
  const meta = CATEGORY_META[category];
  const message = extractMessage(error);

  return {
    category,
    title: meta.title,
    message: message || meta.defaultMessage,
    recoverySteps: meta.recoverySteps,
    retryable: meta.retryable,
    original: error,
  };
}

export function isRetryableError(error: unknown): boolean {
  return classifyError(error).retryable;
}
