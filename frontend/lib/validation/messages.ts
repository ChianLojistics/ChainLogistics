export const VALIDATION_MESSAGES = {
  required: (fieldLabel: string) => `${fieldLabel} is required`,
  minLength: (fieldLabel: string, min: number) => `${fieldLabel} must be at least ${min} characters`,
  maxLength: (fieldLabel: string, max: number) => `${fieldLabel} must be at most ${max} characters`,
  invalidFormat: (fieldLabel: string) => `${fieldLabel} has an invalid format`,

  productIdInvalid:
    "Product ID must be alphanumeric and may include '-' or '_' only",
  productIdLength: (min: number, max: number) =>
    `Product ID must be between ${min} and ${max} characters`,

  stellarAddressInvalid:
    "Address must be a valid Stellar public key (starts with 'G' and is 56 characters)",

  eventTypeInvalid: "Event type must be one of the allowed types",
  timestampFuture: "Timestamp must not be in the future",
  transferRecipientRequired: "Recipient address is required",
} as const;

export type ValidationMessageFormatter = (fieldLabel: string) => string;

export function formatFirstErrorMessage(messages: Array<string | undefined | null>) {
  for (const message of messages) {
    if (typeof message === "string" && message.trim().length > 0) return message;
  }
  return "Invalid value";
}
