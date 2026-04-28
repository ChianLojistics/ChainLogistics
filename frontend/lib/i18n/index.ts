import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from '../../locales/en.json';
import es from '../../locales/es.json';
import fr from '../../locales/fr.json';
import ar from '../../locales/ar.json';

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
      fr: { translation: fr },
      ar: { translation: ar },
    },
    lng: getInitialLanguage(), // default language
    fallbackLng: 'en',
    supportedLngs: ['en', 'es', 'fr', 'ar'],
    interpolation: {
      escapeValue: false, // React already does escaping
    },
    react: {
      useSuspense: false,
    },
  });

// Setup RTL logic when language changes
if (typeof window !== 'undefined') {
  i18n.on('languageChanged', (lng: string) => {
    localStorage.setItem('i18nextLng', lng);
    document.documentElement.dir = i18n.dir(lng);
    document.documentElement.lang = lng;
  });
  
  // Set initial direction
  document.documentElement.dir = i18n.dir(getInitialLanguage());
  document.documentElement.lang = getInitialLanguage();
}

export { i18n };
