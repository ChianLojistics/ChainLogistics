import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from '../../locales/en.json';
import es from '../../locales/es.json';

const getInitialLanguage = (): string => {
  if (typeof window !== 'undefined') {
    return localStorage.getItem('i18nextLng') || 'en';
  }
  return 'en';
};

// Initialize i18next
i18n
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      es: { translation: es },
    },
    lng: getInitialLanguage(), // default language
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // React already does escaping
    },
  });

// Setup RTL logic when language changes
if (typeof window !== 'undefined') {
  i18n.on('languageChanged', (lng: string) => {
    localStorage.setItem('i18nextLng', lng);
    document.documentElement.dir = i18n.dir(lng);
  });
  
  // Set initial direction
  document.documentElement.dir = i18n.dir(getInitialLanguage());
}

export { i18n };
