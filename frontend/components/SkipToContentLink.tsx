"use client";

import { useTranslation } from "react-i18next";

export function SkipToContentLink() {
  const { t } = useTranslation();

  return (
    <a
      href="#main-content"
      className="sr-only focus:not-sr-only focus:fixed focus:top-4 focus:left-4 focus:z-[100] focus:rounded-md focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-medium focus:shadow-lg focus:ring-2 focus:ring-blue-500"
    >
      {t("skip_to_main")}
    </a>
  );
}
