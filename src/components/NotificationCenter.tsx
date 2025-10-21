import React, { useState, useEffect } from 'react';
import { errorHandler, ErrorNotification } from '../services/errorHandler';
import './NotificationCenter.css';

interface NotificationItemProps {
  notification: ErrorNotification;
  onDismiss: (id: string) => void;
}

const NotificationItem: React.FC<NotificationItemProps> = ({ notification, onDismiss }) => {
  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'success':
        return '✓';
      case 'warning':
        return '⚠';
      case 'error':
        return '✕';
      case 'info':
      default:
        return 'ℹ';
    }
  };

  const getTypeClass = (type: string) => {
    return `notification-item notification-${type}`;
  };

  return (
    <div className={getTypeClass(notification.type)}>
      <div className="notification-content">
        <div className="notification-header">
          <span className="notification-icon">{getTypeIcon(notification.type)}</span>
          <span className="notification-title">{notification.title}</span>
          <button 
            className="notification-dismiss"
            onClick={() => onDismiss(notification.id)}
            aria-label="Dismiss notification"
          >
            ×
          </button>
        </div>
        <div className="notification-message">{notification.message}</div>
        {notification.actions && notification.actions.length > 0 && (
          <div className="notification-actions">
            {notification.actions.map((action, index) => (
              <button
                key={index}
                className={`notification-action notification-action-${action.variant || 'secondary'}`}
                onClick={() => {
                  action.action();
                  onDismiss(notification.id);
                }}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}
      </div>
    </div>
  );
};

const NotificationCenter: React.FC = () => {
  const [notifications, setNotifications] = useState<ErrorNotification[]>([]);

  useEffect(() => {
    const unsubscribe = errorHandler.subscribe(setNotifications);
    
    // Set initial notifications
    setNotifications(errorHandler.getNotifications());
    
    return unsubscribe;
  }, []);

  const handleDismiss = (id: string) => {
    errorHandler.dismissNotification(id);
  };

  if (notifications.length === 0) {
    return null;
  }

  return (
    <div className="notification-center">
      {notifications.map(notification => (
        <NotificationItem
          key={notification.id}
          notification={notification}
          onDismiss={handleDismiss}
        />
      ))}
    </div>
  );
};

export default NotificationCenter;