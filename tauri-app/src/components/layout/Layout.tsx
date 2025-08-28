import React, { useState } from 'react';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  List,
  Typography,
  Divider,
  IconButton,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  useTheme,
} from '@mui/material';
import {
  Menu as MenuIcon,
  Home as HomeIcon,
  Assessment as AssessmentIcon,
  Search as SearchIcon,
  Settings as SettingsIcon,
  BugReport as TestIcon,
  Brightness4 as DarkModeIcon,
  Brightness7 as LightModeIcon,
  Language as LanguageIcon,
} from '@mui/icons-material';
import { useNavigate, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';

import { useTheme as useAppTheme } from '../../contexts/ThemeContext';
import { useLanguage } from '../../contexts/LanguageContext';
import ScatteredFluxText from '../ScatteredFluxText';
import FluxLogo from '../FluxLogo';

const drawerWidth = 240;

interface MenuItem {
  path: string;
  labelKey: string;
  icon: React.ReactElement;
}

const menuItems: MenuItem[] = [
  { path: '/', labelKey: 'navigation.home', icon: <HomeIcon /> },
  { path: '/audit', labelKey: 'navigation.analysis', icon: <AssessmentIcon /> },
  { path: '/query', labelKey: 'navigation.query', icon: <SearchIcon /> },
  // { path: '/test', labelKey: 'navigation.test', icon: <TestIcon /> },
  { path: '/settings', labelKey: 'navigation.settings', icon: <SettingsIcon /> },
];

interface LayoutProps {
  children: React.ReactNode;
}

const Layout: React.FC<LayoutProps> = ({ children }) => {
  const theme = useTheme();
  const { t } = useTranslation();
  const navigate = useNavigate();
  const location = useLocation();
  const { themeMode, toggleTheme } = useAppTheme();
  const { toggleLanguage, currentLanguage } = useLanguage();
  
  const [mobileOpen, setMobileOpen] = useState(false);

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  const handleMenuClick = (path: string) => {
    navigate(path);
    setMobileOpen(false);
  };

  const drawer = (
    <Box sx={{ 
      height: '100%', 
      width: '100%',
      maxWidth: drawerWidth,
      display: 'flex', 
      flexDirection: 'column',
      position: 'relative',
      overflow: 'hidden', // 防止整体滚动
      boxSizing: 'border-box'
    }}>
      {/* 左上角分散式FLUX文字 */}
      <Toolbar sx={{ 
        display: 'flex', 
        justifyContent: 'center',
        alignItems: 'center',
        paddingTop: 2,
        paddingBottom: 2,
        minHeight: 'auto', // 避免默认高度
        flexShrink: 0,
        height: '80px' // 给文字更多空间
      }}>
        <ScatteredFluxText size={36} />
      </Toolbar>
      
      <Divider sx={{ flexShrink: 0 }} />
      
      {/* 主要菜单区域 */}
      <Box sx={{ 
        flex: 1, 
        overflow: 'auto',
        minHeight: 0 // 重要：允许flex子项收缩
      }}>
        <List>
          {menuItems.map((item) => (
            <ListItem key={item.path} disablePadding>
              <ListItemButton
                selected={location.pathname === item.path}
                onClick={() => handleMenuClick(item.path)}
                sx={{
                  paddingLeft: 2,
                  paddingRight: 2,
                  '&.Mui-selected': {
                    backgroundColor: theme.palette.primary.main + '20',
                    '&:hover': {
                      backgroundColor: theme.palette.primary.main + '30',
                    },
                  },
                }}
              >
                <ListItemIcon
                  sx={{
                    color: location.pathname === item.path 
                      ? theme.palette.primary.main 
                      : 'inherit',
                    minWidth: 56, // 标准图标宽度
                  }}
                >
                  {item.icon}
                </ListItemIcon>
                <ListItemText 
                  primary={t(item.labelKey)}
                  sx={{
                    color: location.pathname === item.path 
                      ? theme.palette.primary.main 
                      : 'inherit',
                    marginLeft: '20px', // 文字向右移动20px
                  }}
                />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
        <Divider />
        <List>
          <ListItem disablePadding>
            <ListItemButton 
              onClick={toggleTheme}
              sx={{
                paddingLeft: 2,
                paddingRight: 2,
              }}
            >
              <ListItemIcon sx={{ minWidth: 56 }}>
                {themeMode === 'dark' ? <LightModeIcon /> : <DarkModeIcon />}
              </ListItemIcon>
              <ListItemText 
                primary={
                  themeMode === 'dark' 
                    ? t('settings.light_theme') 
                    : t('settings.dark_theme')
                }
                sx={{ marginLeft: '20px' }}
              />
            </ListItemButton>
          </ListItem>
          <ListItem disablePadding>
            <ListItemButton 
              onClick={toggleLanguage}
              sx={{
                paddingLeft: 2,
                paddingRight: 2,
              }}
            >
              <ListItemIcon sx={{ minWidth: 56 }}>
                <LanguageIcon />
              </ListItemIcon>
              <ListItemText 
                primary={
                  currentLanguage === 'zh' 
                    ? 'English' 
                    : '中文'
                }
                sx={{ marginLeft: '20px' }}
              />
            </ListItemButton>
          </ListItem>
        </List>
        
        {/* 左下角大型旋转logo */}
        <Box sx={{ 
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          padding: 2,
          marginTop: 'auto',
          height: '300px', // 增加高度到300px
          width: `${drawerWidth}px`,
          maxWidth: `${drawerWidth}px`,
          overflow: 'hidden',
          boxSizing: 'border-box',
          animation: 'continuousRotate 10s linear infinite',
          '@keyframes continuousRotate': {
            '0%': { transform: 'rotate(0deg)' },
            '100%': { transform: 'rotate(360deg)' }
          }
        }}>
          <FluxLogo size={140} showText={false} />
        </Box>
        
        {/* 最底部版本号 */}
        <Box sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          padding: 1,
          borderTop: '1px solid',
          borderColor: 'divider',
          backgroundColor: 'background.paper',
          opacity: 0.7
        }}>
          <Typography variant="caption" sx={{ 
            color: 'text.secondary',
            fontSize: '0.75rem',
            letterSpacing: '0.5px'
          }}>
            v2.0.0-Dev
          </Typography>
        </Box>
      </Box>
    </Box>
  );

  return (
    <Box sx={{ 
      display: 'flex', 
      backgroundColor: theme.palette.background.default,
      minHeight: '100vh',
      width: '100vw'
    }}>
      <AppBar
        position="fixed"
        sx={{
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          ml: { sm: `${drawerWidth}px` },
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { sm: 'none' } }}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1 }}>
            {t('app.subtitle')}
          </Typography>
        </Toolbar>
      </AppBar>
      
      <Box
        component="nav"
        sx={{ width: { sm: drawerWidth }, flexShrink: { sm: 0 } }}
      >
        <Drawer
          variant="temporary"
          open={mobileOpen}
          onClose={handleDrawerToggle}
          ModalProps={{
            keepMounted: true, // Better open performance on mobile.
          }}
          sx={{
            display: { xs: 'block', sm: 'none' },
            '& .MuiDrawer-paper': { 
              boxSizing: 'border-box', 
              width: drawerWidth,
              maxWidth: drawerWidth,
              overflow: 'hidden'
            },
          }}
        >
          {drawer}
        </Drawer>
        <Drawer
          variant="permanent"
          sx={{
            display: { xs: 'none', sm: 'block' },
            '& .MuiDrawer-paper': { 
              boxSizing: 'border-box', 
              width: drawerWidth,
              maxWidth: drawerWidth,
              overflow: 'hidden'
            },
          }}
          open
        >
          {drawer}
        </Drawer>
      </Box>
      
      <Box
        component="main"
        sx={{ 
          flexGrow: 1, 
          p: 3, 
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          height: '100vh',
          overflow: 'auto',
        }}
      >
        <Toolbar />
        {children}
      </Box>
    </Box>
  );
};

export default Layout;