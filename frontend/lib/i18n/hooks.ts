/**
 * Custom hooks for i18n functionality
 */

import { useTranslation } from "react-i18next";
import {
  isRTL as isRTLUtil,
  setLanguage as setLanguageUtil,
  getLanguageInfo as getLanguageInfoUtil,
  getCurrentLanguage as getCurrentLanguageUtil,
} from "@/lib/i18n/config";

/**
 * Hook for accessing i18n translation function
 */
export function useI18n() {
  const { t, i18n } = useTranslation();

  return {
    t,
    i18n,
    locale: i18n.language,
    changeLanguage: i18n.changeLanguage,
    isRTL: isRTLUtil(),
  };
}

/**
 * Hook for formatting numbers according to current locale
 */
export function useNumberFormat() {
  const { locale } = useI18n();

  const formatNumber = (value: number, options?: Intl.NumberFormatOptions) => {
    return new Intl.NumberFormat(locale, options).format(value);
  };

  const formatCurrency = (value: number, currency: string = "USD") => {
    return new Intl.NumberFormat(locale, {
      style: "currency",
      currency,
    }).format(value);
  };

  const formatPercent = (value: number, options?: Intl.NumberFormatOptions) => {
    return new Intl.NumberFormat(locale, {
      style: "percent",
      ...options,
    }).format(value / 100);
  };

  const formatDecimal = (value: number, maximumFractionDigits: number = 2) => {
    return new Intl.NumberFormat(locale, {
      maximumFractionDigits,
    }).format(value);
  };

  return {
    formatNumber,
    formatCurrency,
    formatPercent,
    formatDecimal,
  };
}

/**
 * Hook for formatting dates according to current locale
 */
export function useDateFormat() {
  const { locale } = useI18n();

  const formatDate = (date: Date, options?: Intl.DateTimeFormatOptions) => {
    return new Intl.DateTimeFormat(locale, options).format(date);
  };

  const formatTime = (date: Date, options?: Intl.DateTimeFormatOptions) => {
    return new Intl.DateTimeFormat(locale, {
      hour: "numeric",
      minute: "numeric",
      ...options,
    }).format(date);
  };

  const formatDateTime = (date: Date, options?: Intl.DateTimeFormatOptions) => {
    return new Intl.DateTimeFormat(locale, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "numeric",
      minute: "numeric",
      ...options,
    }).format(date);
  };

  const formatRelativeTime = (
    value: number,
    unit: Intl.RelativeTimeFormatUnit
  ) => {
    return new Intl.RelativeTimeFormat(locale, { numeric: "auto" }).format(value, unit);
  };

  const formatShortDate = (date: Date) => {
    return new Intl.DateTimeFormat(locale, {
      month: "short",
      day: "numeric",
      year: "numeric",
    }).format(date);
  };

  const formatLongDate = (date: Date) => {
    return new Intl.DateTimeFormat(locale, {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
    }).format(date);
  };

  return {
    formatDate,
    formatTime,
    formatDateTime,
    formatRelativeTime,
    formatShortDate,
    formatLongDate,
  };
}

/**
 * Hook for RTL/LTR support
 */
export function useRTL() {
  const { locale } = useI18n();
  const isRTLValue = isRTLUtil();

  return {
    isRTL: isRTLValue,
    direction: isRTLValue ? "rtl" : "ltr",
    locale,
  };
}

/**
 * Hook for language management
 */
export function useLanguage() {
  const { i18n, locale } = useI18n();

  const changeLanguage = (langCode: string) => {
    setLanguageUtil(langCode);
  };

  const getCurrentLanguage = () => {
    return getCurrentLanguageUtil();
  };

  const getLanguageInfo = (code: string) => {
    return getLanguageInfoUtil(code);
  };

  return {
    currentLanguage: locale,
    changeLanguage,
    getCurrentLanguage,
    getLanguageInfo,
  };
}

/**
 * Hook for pluralization
 */
export function usePlural() {
  const { t, locale } = useI18n();

  const pluralize = (key: string, count: number, options?: Record<string, any>) => {
    // Get the plural form based on count
    const pluralKey = count === 1 ? `${key}_one` : `${key}_other`;
    return t(pluralKey, { count, ...options });
  };

  return {
    pluralize,
  };
}
