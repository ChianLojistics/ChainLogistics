import { i18n } from "@/lib/i18n";

type DateInput = Date | number;

function getLocale(): string {
  return i18n.resolvedLanguage || i18n.language || "en";
}

function toDate(input: DateInput): Date {
  return input instanceof Date ? input : new Date(input);
}

export function formatNumber(value: number, options?: Intl.NumberFormatOptions): string {
  return new Intl.NumberFormat(getLocale(), options).format(value);
}

export function formatDate(value: DateInput, options?: Intl.DateTimeFormatOptions): string {
  return new Intl.DateTimeFormat(getLocale(), options).format(toDate(value));
}

export function formatTime(value: DateInput, options?: Intl.DateTimeFormatOptions): string {
  return new Intl.DateTimeFormat(getLocale(), {
    hour: "2-digit",
    minute: "2-digit",
    ...options,
  }).format(toDate(value));
}
