"use client";

import { useEffect, useState } from "react";
import type { ChangeEvent } from "react";
import { useTranslation } from "react-i18next";
import { i18n } from "../lib/i18n"; // Ensure this runs to initialize i18next

export function LanguageSelector() {
  const { t } = useTranslation();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const toggleLanguage = async () => {
    const newLang = i18n.language === "en" ? "es" : "en";
    if (newLang !== "en" && !i18n.hasResourceBundle(newLang, "translation")) {
      const locale = (await import(`../locales/${newLang}.json`)).default;
      i18n.addResourceBundle(newLang, "translation", locale, true, true);
    }
    await i18n.changeLanguage(newLang);
  };

  if (!mounted) return null;

  return (
    <label className="inline-flex items-center gap-2">
      <span className="sr-only">{t("language")}</span>
      <select
        value={i18n.resolvedLanguage || i18n.language}
        onChange={(e: ChangeEvent<HTMLSelectElement>) => changeLanguage(e.target.value)}
        className="px-3 py-1 bg-zinc-100 hover:bg-zinc-200 text-zinc-800 text-sm font-medium rounded-md transition-colors"
        aria-label={t("language")}
        title={t("language")}
      >
        <option value="en">EN</option>
        <option value="es">ES</option>
        <option value="fr">FR</option>
        <option value="ar">AR</option>
      </select>
    </label>
  );
}
