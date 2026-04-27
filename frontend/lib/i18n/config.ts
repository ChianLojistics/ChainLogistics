/**
 * i18n configuration for ChainLogistics
 * Uses i18next and react-i18next for internationalization
 */

import i18n from "i18next";
import { initReactI18next } from "react-i18next";

// Import translation files
import en from "./locales/en.json";
import es from "./locales/es.json";
import fr from "./locales/fr.json";
import ar from "./locales/ar.json";
import zh from "./locales/zh.json";
import de from "./locales/de.json";
import ja from "./locales/ja.json";

// Language resources
const resources = {
  en: { translation: en },
  es: { translation: es },
  fr: { translation: fr },
  ar: { translation: ar },
  zh: { translation: zh },
  de: { translation: de },
  ja: { translation: ja },
};

// Language metadata
export const languages = [
  { code: "en", name: "English", nativeName: "English", direction: "ltr" },
  { code: "es", name: "Spanish", nativeName: "Español", direction: "ltr" },
  { code: "fr", name: "French", nativeName: "Français", direction: "ltr" },
  { code: "ar", name: "Arabic", nativeName: "العربية", direction: "rtl" },
  { code: "zh", name: "Chinese", nativeName: "中文", direction: "ltr" },
  { code: "de", name: "German", nativeName: "Deutsch", direction: "ltr" },
  { code: "ja", name: "Japanese", nativeName: "日本語", direction: "ltr" },
];

// RTL languages
export const rtlLanguages = ["ar", "he", "fa", "ur"];

// Get saved language from localStorage or use browser language
const getInitialLanguage = (): string => {
  if (typeof window !== "undefined") {
    const saved = localStorage.getItem("language");
    if (saved && languages.some((lang) => lang.code === saved)) {
      return saved;
    }

    const browserLang = navigator.language.split("-")[0];
    if (languages.some((lang) => lang.code === browserLang)) {
      return browserLang;
    }
  }
  return "en"; // Default to English
};

// Initialize i18next
i18n
  .use(initReactI18next)
  .init({
    resources,
    lng: getInitialLanguage(),
    fallbackLng: "en",
    debug: process.env.NODE_ENV === "development",

    interpolation: {
      escapeValue: false, // React already escapes values
    },

    react: {
      useSuspense: false,
    },
  });

// Save language changes to localStorage
i18n.on("languageChanged", (lng) => {
  if (typeof window !== "undefined") {
    localStorage.setItem("language", lng);
    // Update document direction for RTL support
    const isRTL = rtlLanguages.includes(lng);
    document.documentElement.dir = isRTL ? "rtl" : "ltr";
    document.documentElement.lang = lng;
  }
});

// Set initial document direction
if (typeof window !== "undefined") {
  const currentLang = i18n.language;
  const isRTL = rtlLanguages.includes(currentLang);
  document.documentElement.dir = isRTL ? "rtl" : "ltr";
  document.documentElement.lang = currentLang;
}

export default i18n;

// Helper functions
export const getCurrentLanguage = (): string => i18n.language;

export const isRTL = (): boolean => rtlLanguages.includes(i18n.language);

export const setLanguage = (lang: string): void => {
  i18n.changeLanguage(lang);
};

export const getLanguageInfo = (code: string) => {
  return languages.find((lang) => lang.code === code);
};

export const formatNumber = (value: number, options?: Intl.NumberFormatOptions): string => {
  return new Intl.NumberFormat(i18n.language, options).format(value);
};

export const formatDate = (date: Date, options?: Intl.DateTimeFormatOptions): string => {
  return new Intl.DateTimeFormat(i18n.language, options).format(date);
};

export const formatCurrency = (value: number, currency: string = "USD"): string => {
  return new Intl.NumberFormat(i18n.language, {
    style: "currency",
    currency,
  }).format(value);
};

export const formatRelativeTime = (value: number, unit: Intl.RelativeTimeFormatUnit): string => {
  return new Intl.RelativeTimeFormat(i18n.language, { numeric: "auto" }).format(value, unit);
};
