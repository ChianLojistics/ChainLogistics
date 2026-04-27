# Internationalization (i18n) Guide

This guide provides comprehensive documentation for the ChainLogistics frontend internationalization system.

## Overview

The i18n system uses `i18next` and `react-i18next` to provide multi-language support for the ChainLogistics application. It includes:

- **7 supported languages**: English, Spanish, French, Arabic (RTL), Chinese, German, Japanese
- **RTL support**: Automatic right-to-left layout for Arabic and other RTL languages
- **Localized formatting**: Date, number, and currency formatting based on locale
- **Language persistence**: User's language choice saved in localStorage
- **Browser detection**: Automatic language detection from browser settings

## Supported Languages

| Code | Language | Native Name | Direction |
|------|----------|-------------|-----------|
| `en` | English | English | LTR |
| `es` | Spanish | Español | LTR |
| `fr` | French | Français | LTR |
| `ar` | Arabic | العربية | RTL |
| `zh` | Chinese | 中文 | LTR |
| `de` | German | Deutsch | LTR |
| `ja` | Japanese | 日本語 | LTR |

## Architecture

### Core Components

1. **Configuration** (`lib/i18n/config.ts`)
   - i18next initialization
   - Language resource loading
   - RTL language detection
   - Document direction management
   - Formatting utilities

2. **Translation Files** (`lib/i18n/locales/*.json`)
   - JSON files for each language
   - Organized by namespace (common, search, product, etc.)
   - Support for interpolation and pluralization

3. **Components**
   - `I18nProvider`: React context provider for i18n
   - `LanguageSwitcher`: UI component for language selection

4. **Custom Hooks** (`lib/i18n/hooks.ts`)
   - `useI18n`: Translation function access
   - `useNumberFormat`: Number/currency formatting
   - `useDateFormat`: Date/time formatting
   - `useRTL`: RTL/LTR detection
   - `useLanguage`: Language management

## Installation

The required dependencies are already installed in the project:

```json
{
  "i18next": "^25.10.10",
  "react-i18next": "^16.6.6"
}
```

## Setup

### 1. Wrap Your App with I18nProvider

```tsx
// app/layout.tsx or pages/_app.tsx
import { I18nProvider } from "@/components/i18n/I18nProvider";

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <I18nProvider>
          {children}
        </I18nProvider>
      </body>
    </html>
  );
}
```

### 2. Add Language Switcher to Your UI

```tsx
import { LanguageSwitcher } from "@/components/i18n/LanguageSwitcher";

function Header() {
  return (
    <header>
      <LanguageSwitcher />
    </header>
  );
}
```

## Usage Guide

### Basic Translation

```tsx
import { useI18n } from "@/lib/i18n/hooks";

function MyComponent() {
  const { t } = useI18n();

  return (
    <div>
      <h1>{t("common.welcome")}</h1>
      <p>{t("product.title")}</p>
    </div>
  );
}
```

### Translation with Interpolation

```tsx
const { t } = useI18n();

// In translation file: "validation.minLength": "Must be at least {{count}} characters"
<p>{t("validation.minLength", { count: 5 })}</p>
```

### Number Formatting

```tsx
import { useNumberFormat } from "@/lib/i18n/hooks";

function PriceDisplay({ price }: { price: number }) {
  const { formatCurrency } = useNumberFormat();

  return <span>{formatCurrency(price, "USD")}</span>;
  // Output: $1,234.56 (en), 1.234,56 $ (de), ¥1,235 (ja)
}
```

### Date Formatting

```tsx
import { useDateFormat } from "@/lib/i18n/hooks";

function DateDisplay({ date }: { date: Date }) {
  const { formatDate, formatRelativeTime } = useDateFormat();

  return (
    <div>
      <span>{formatDate(date)}</span>
      <span>{formatRelativeTime(-2, "days")}</span>
    </div>
  );
}
```

### RTL Support

```tsx
import { useRTL } from "@/lib/i18n/hooks";

function MyComponent() {
  const { isRTL, direction } = useRTL();

  return (
    <div dir={direction} className={isRTL ? "rtl-specific-class" : ""}>
      {/* Content */}
    </div>
  );
}
```

### Language Switching

```tsx
import { useLanguage } from "@/lib/i18n/hooks";

function LanguageSelector() {
  const { currentLanguage, changeLanguage } = useLanguage();

  return (
    <select
      value={currentLanguage}
      onChange={(e) => changeLanguage(e.target.value)}
    >
      <option value="en">English</option>
      <option value="es">Español</option>
      <option value="fr">Français</option>
    </select>
  );
}
```

## Translation File Structure

Translation files are organized by namespace for better maintainability:

```json
{
  "common": {
    // Common UI elements used across the app
    "search": "Search",
    "save": "Save",
    "cancel": "Cancel"
  },
  "search": {
    // Search-specific translations
    "placeholder": "Search...",
    "advancedSearch": "Advanced Search"
  },
  "product": {
    // Product-related translations
    "title": "Product",
    "addProduct": "Add Product"
  },
  "event": {
    // Event-related translations
    "title": "Event",
    "shipped": "Shipped"
  },
  // ... more namespaces
}
```

## Adding a New Language

### 1. Create Translation File

Create a new JSON file in `lib/i18n/locales/`:

```json
// lib/i18n/locales/pt.json
{
  "common": {
    "search": "Pesquisar",
    "save": "Salvar",
    // ... other translations
  },
  // ... other namespaces
}
```

### 2. Add Language to Configuration

Update `lib/i18n/config.ts`:

```typescript
import pt from "./locales/pt.json";

const resources = {
  // ... existing languages
  pt: { translation: pt },
};

const languages = [
  // ... existing languages
  { code: "pt", name: "Portuguese", nativeName: "Português", direction: "ltr" },
];
```

### 3. Add RTL Support (if needed)

If the new language is RTL, add it to the RTL languages list:

```typescript
export const rtlLanguages = ["ar", "he", "fa", "ur", "your-language"];
```

## RTL Support

RTL (Right-to-Left) support is automatic for languages marked as RTL. The system:

1. Sets `document.documentElement.dir` to "rtl" when an RTL language is selected
2. Updates the `dir` attribute dynamically on language change
3. Provides a `useRTL` hook for component-level RTL detection

### RTL Styling

```css
/* In your CSS/Tailwind */
[dir="rtl"] .margin-left {
  margin-left: 0;
  margin-right: 1rem;
}

/* Or using Tailwind logical properties */
[dir="rtl"] .my-element {
  margin-inline-start: 1rem; /* Automatically flips based on direction */
}
```

## Date and Number Formatting

The system uses the native JavaScript `Intl` API for formatting:

### Number Formatting

```typescript
const { formatNumber } = useNumberFormat();

formatNumber(1234.56); // 1,234.56 (en), 1.234,56 (de), 1,235 (ja)
formatNumber(1234.56, { style: "currency", currency: "USD" });
```

### Date Formatting

```typescript
const { formatDate, formatDateTime } = useDateFormat();

formatDate(new Date()); // 1/1/2024 (en), 01/01/2024 (de), 2024/1/1 (ja)
formatDateTime(new Date()); // Jan 1, 2024, 12:00 PM (en)
```

### Relative Time

```typescript
const { formatRelativeTime } = useDateFormat();

formatRelativeTime(-2, "days"); // "2 days ago" (en), "hace 2 días" (es)
```

## Best Practices

1. **Use namespaces**: Organize translations by feature or component
2. **Keep keys consistent**: Use the same key structure across all language files
3. **Provide context**: Include comments in translation files for translators
4. **Test RTL**: Always test RTL languages for layout issues
5. **Handle long text**: Some languages (like German) have longer text
6. **Use interpolation**: For dynamic values, use interpolation instead of concatenation
7. **Pluralization**: Use separate keys for singular/plural forms
8. **Date formats**: Use locale-appropriate date formats

## Translation Keys Reference

### Common Keys

- `common.search` - Search button/text
- `common.filter` - Filter button/text
- `common.save` - Save button
- `common.cancel` - Cancel button
- `common.loading` - Loading state
- `common.noResults` - Empty state message

### Search Keys

- `search.placeholder` - Search input placeholder
- `search.advancedSearch` - Advanced search label
- `search.sortBy` - Sort by label
- `search.relevance` - Relevance sort option
- `search.savedSearches` - Saved searches section
- `search.searchHistory` - Search history section

### Product Keys

- `product.title` - Product title
- `product.addProduct` - Add product button
- `product.viewProduct` - View product button
- `product.supplyChain` - Supply chain label

### Event Keys

- `event.title` - Event title
- `event.timeline` - Timeline label
- `event.shipped` - Shipped event type
- `event.received` - Received event type

## Troubleshooting

### Language Not Switching

- Check that the translation file exists and is valid JSON
- Verify the language code matches the config
- Check browser console for errors

### RTL Not Working

- Verify the language is in the `rtlLanguages` array
- Check that `document.documentElement.dir` is being set
- Ensure CSS uses logical properties or RTL-specific classes

### Missing Translations

- Missing translations fall back to English
- Check the console for missing translation warnings in development mode
- Ensure all keys exist in all language files

### Date/Number Formatting Issues

- Verify the locale code is valid
- Check browser support for the locale
- Test with different browsers for consistency

## Future Enhancements

- Server-side rendering support
- Lazy loading of translation files
- Translation management dashboard
- Crowdsourced translations
- Auto-translation integration
- Language-specific content
- Regional variations (en-US, en-GB, etc.)
