import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// 导入翻译资源
import zhTranslations from './zh.json';
import enTranslations from './en.json';

const resources = {
  zh: {
    translation: zhTranslations,
  },
  en: {
    translation: enTranslations,
  },
};

i18n
  // 检测用户语言
  .use(LanguageDetector)
  // 传递 i18n 实例给 react-i18next
  .use(initReactI18next)
  // 初始化 i18next
  .init({
    resources,
    lng: 'zh', // 默认语言
    fallbackLng: 'zh', // 回退语言
    
    // 检测选项
    detection: {
      order: ['localStorage', 'navigator', 'htmlTag'],
      caches: ['localStorage'],
    },
    
    interpolation: {
      escapeValue: false, // react 已经转义
    },
    
    // 调试选项
    debug: process.env.NODE_ENV === 'development',
    
    // 命名空间
    defaultNS: 'translation',
    ns: ['translation'],
    
    // 键分隔符
    keySeparator: '.',
    nsSeparator: ':',
    
    // React options
    react: {
      useSuspense: false,
    },
  });

export default i18n;