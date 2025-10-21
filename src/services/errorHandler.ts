import { ApiOperationError } from './api';

export interface ErrorNotification {
  id: string;
  type: 'error' | 'warning' | 'info' | 'success';
  title: string;
  message: string;
  timestamp: Date;
  duration?: number; // Auto-dismiss after this many milliseconds
  actions?: ErrorAction[];
}

export interface ErrorAction {
  label: string;
  action: () => void;
  variant?: 'primary' | 'secondary' | 'danger';
}

/**
 * Error handler service for managing application errors and user feedback
 */
export class ErrorHandler {
  private static instance: ErrorHandler;
  private notifications: ErrorNotification[] = [];
  private listeners: ((notifications: ErrorNotification[]) => void)[] = [];

  private constructor() {}

  static getInstance(): ErrorHandler {
    if (!ErrorHandler.instance) {
      ErrorHandler.instance = new ErrorHandler();
    }
    return ErrorHandler.instance;
  }

  /**
   * Subscribe to notification updates
   */
  subscribe(listener: (notifications: ErrorNotification[]) => void): () => void {
    this.listeners.push(listener);
    
    // Return unsubscribe function
    return () => {
      const index = this.listeners.indexOf(listener);
      if (index > -1) {
        this.listeners.splice(index, 1);
      }
    };
  }

  /**
   * Notify all listeners of notification changes
   */
  private notifyListeners(): void {
    this.listeners.forEach(listener => listener([...this.notifications]));
  }

  /**
   * Add a notification
   */
  private addNotification(notification: Omit<ErrorNotification, 'id' | 'timestamp'>): string {
    const id = `notification-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    const fullNotification: ErrorNotification = {
      ...notification,
      id,
      timestamp: new Date(),
    };

    this.notifications.push(fullNotification);
    this.notifyListeners();

    // Auto-dismiss if duration is specified
    if (notification.duration) {
      setTimeout(() => {
        this.dismissNotification(id);
      }, notification.duration);
    }

    return id;
  }

  /**
   * Dismiss a notification
   */
  dismissNotification(id: string): void {
    const index = this.notifications.findIndex(n => n.id === id);
    if (index > -1) {
      this.notifications.splice(index, 1);
      this.notifyListeners();
    }
  }

  /**
   * Clear all notifications
   */
  clearAll(): void {
    this.notifications = [];
    this.notifyListeners();
  }

  /**
   * Handle API errors with user-friendly messages
   */
  handleApiError(error: unknown, context?: string): string {
    let title = 'Operation Failed';
    let message = 'An unexpected error occurred';
    let actions: ErrorAction[] = [];

    if (context) {
      title = `${context} Failed`;
    }

    if (error instanceof ApiOperationError) {
      message = error.message;
      
      // Add specific actions based on error type
      if (error.code === 'NETWORK_ERROR') {
        actions.push({
          label: 'Retry',
          action: () => {
            // This would be handled by the calling component
            console.log('Retry action triggered');
          },
          variant: 'primary',
        });
      }
    } else if (error instanceof Error) {
      message = error.message;
    } else if (typeof error === 'string') {
      message = error;
    }

    return this.addNotification({
      type: 'error',
      title,
      message,
      actions,
    });
  }

  /**
   * Handle success messages
   */
  handleSuccess(message: string, title = 'Success'): string {
    return this.addNotification({
      type: 'success',
      title,
      message,
      duration: 3000, // Auto-dismiss after 3 seconds
    });
  }

  /**
   * Handle warning messages
   */
  handleWarning(message: string, title = 'Warning'): string {
    return this.addNotification({
      type: 'warning',
      title,
      message,
      duration: 5000, // Auto-dismiss after 5 seconds
    });
  }

  /**
   * Handle info messages
   */
  handleInfo(message: string, title = 'Information'): string {
    return this.addNotification({
      type: 'info',
      title,
      message,
      duration: 4000, // Auto-dismiss after 4 seconds
    });
  }

  /**
   * Get current notifications
   */
  getNotifications(): ErrorNotification[] {
    return [...this.notifications];
  }
}

// Export singleton instance
export const errorHandler = ErrorHandler.getInstance();

/**
 * React hook for using the error handler
 */
export const useErrorHandler = () => {
  return {
    handleApiError: errorHandler.handleApiError.bind(errorHandler),
    handleSuccess: errorHandler.handleSuccess.bind(errorHandler),
    handleWarning: errorHandler.handleWarning.bind(errorHandler),
    handleInfo: errorHandler.handleInfo.bind(errorHandler),
    dismissNotification: errorHandler.dismissNotification.bind(errorHandler),
    clearAll: errorHandler.clearAll.bind(errorHandler),
  };
};