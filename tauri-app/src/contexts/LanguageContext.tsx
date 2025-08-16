import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { Language } from '../types/app';

interface LanguageContextType {
  currentLanguage: Language;
  setLanguage: (language: Language) => void;
  toggleLanguage: () => void;
  availableLanguages: { code: Language; name: string; nativeName: string }[];
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export const useLanguage = (): LanguageContextType => {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within a LanguageContextProvider');
  }
  return context;
};

interface LanguageContextProviderProps {
  children: ReactNode;
}

const availableLanguages = [
  { code: 'zh' as Language, name: 'Chinese', nativeName: '中文' },
  { code: 'en' as Language, name: 'English', nativeName: 'English' },
];

export const LanguageContextProvider: React.FC<LanguageContextProviderProps> = ({ children }) => {
  const { i18n } = useTranslation();
  const [currentLanguage, setCurrentLanguage] = useState<Language>('zh');

  // 检测浏览器语言
  const getBrowserLanguage = (): Language => {
    if (typeof window !== 'undefined') {
      const browserLang = navigator.language.toLowerCase();
      if (browserLang.startsWith('zh')) {
        return 'zh';
      } else if (browserLang.startsWith('en')) {
        return 'en';
      }
    }
    return 'zh'; // 默认中文
  };

  // 从本地存储加载语言设置
  useEffect(() => {
    const savedLanguage = localStorage.getItem('app-language') as Language;
    if (savedLanguage && ['zh', 'en'].includes(savedLanguage)) {
      setCurrentLanguage(savedLanguage);
      i18n.changeLanguage(savedLanguage);
    } else {
      // 使用浏览器语言
      const browserLang = getBrowserLanguage();
      setCurrentLanguage(browserLang);
      i18n.changeLanguage(browserLang);
    }
  }, [i18n]);

  // 设置语言
  const setLanguage = (language: Language) => {
    setCurrentLanguage(language);
    i18n.changeLanguage(language);
    localStorage.setItem('app-language', language);
  };

  // 切换语言
  const toggleLanguage = () => {
    const newLanguage: Language = currentLanguage === 'zh' ? 'en' : 'zh';
    setLanguage(newLanguage);
  };

  const value: LanguageContextType = {
    currentLanguage,
    setLanguage,
    toggleLanguage,
    availableLanguages,
  };

  return (
    <LanguageContext.Provider value={value}>
      {children}
    </LanguageContext.Provider>
  );
};