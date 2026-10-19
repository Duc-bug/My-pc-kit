import { createI18n } from 'vue-i18n';
import en from './locales/en.json';
import vi from './locales/vi.json';

const savedLocale = typeof window !== 'undefined' ? localStorage.getItem('pt-locale') || 'vi' : 'vi';

export const i18n = createI18n({
  legacy: false,
  locale: savedLocale,
  fallbackLocale: 'en',
  messages: { en, vi },
});
