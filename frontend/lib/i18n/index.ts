import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import en from '../../locales/en.json';

const loadLocale = async (lng: string) => {
  switch (lng) {
    case 'es':
      return (await import('../../locales/es.json')).default;
    default:
      return en;
  }
};

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
    },
    lng: getInitialLanguage(), // default language
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // React already does escaping
    },
  });

// Setup RTL logic when language changes
if (typeof window !== 'undefined') {
  void (async () => {
    const lng = getInitialLanguage();
    if (lng !== 'en' && !i18n.hasResourceBundle(lng, 'translation')) {
      const locale = await loadLocale(lng);
      i18n.addResourceBundle(lng, 'translation', locale, true, true);
    }
  })();

  i18n.on('languageChanged', (lng: string) => {
    localStorage.setItem('i18nextLng', lng);
    document.documentElement.dir = i18n.dir(lng);

    if (!i18n.hasResourceBundle(lng, 'translation')) {
      void loadLocale(lng).then((locale) => {
        i18n.addResourceBundle(lng, 'translation', locale, true, true);
      });
    }
  });
  
  // Set initial direction
  document.documentElement.dir = i18n.dir(getInitialLanguage());
}

export { i18n };
