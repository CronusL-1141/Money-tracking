import React, { createContext, useContext, useState, ReactNode } from 'react';
import { Snackbar, Alert, AlertColor } from '@mui/material';
import { NotificationOptions } from '../types/app';

interface NotificationContextType {
  showNotification: (options: NotificationOptions) => void;
  hideNotification: () => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

export const useNotification = (): NotificationContextType => {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error('useNotification must be used within a NotificationProvider');
  }
  return context;
};

interface NotificationProviderProps {
  children: ReactNode;
}

interface NotificationState {
  open: boolean;
  type: AlertColor;
  title: string;
  message?: string;
  duration: number;
}

export const NotificationProvider: React.FC<NotificationProviderProps> = ({ children }) => {
  const [notification, setNotification] = useState<NotificationState>({
    open: false,
    type: 'info',
    title: '',
    message: '',
    duration: 6000,
  });

  const showNotification = (options: NotificationOptions) => {
    setNotification({
      open: true,
      type: options.type as AlertColor,
      title: options.title,
      message: options.message,
      duration: options.duration || 6000,
    });
  };

  const hideNotification = () => {
    setNotification(prev => ({ ...prev, open: false }));
  };

  const handleClose = (event?: React.SyntheticEvent | Event, reason?: string) => {
    if (reason === 'clickaway') {
      return;
    }
    hideNotification();
  };

  const value: NotificationContextType = {
    showNotification,
    hideNotification,
  };

  return (
    <NotificationContext.Provider value={value}>
      {children}
      <Snackbar
        open={notification.open}
        autoHideDuration={notification.duration}
        onClose={handleClose}
        anchorOrigin={{ vertical: 'top', horizontal: 'right' }}
      >
        <Alert 
          onClose={handleClose} 
          severity={notification.type}
          variant="filled"
          sx={{ width: '100%', minWidth: 300 }}
        >
          <strong>{notification.title}</strong>
          {notification.message && (
            <div style={{ marginTop: 4, fontSize: '0.875rem' }}>
              {notification.message}
            </div>
          )}
        </Alert>
      </Snackbar>
    </NotificationContext.Provider>
  );
};