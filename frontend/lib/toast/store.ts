import { create } from 'zustand';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export type ToastPosition = 
  | 'top-right'
  | 'top-left'
  | 'top-center'
  | 'bottom-right'
  | 'bottom-left'
  | 'bottom-center';

export interface Toast {
  id: string;
  type: ToastType;
  title?: string;
  message: string;
  duration?: number;
  position?: ToastPosition;
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface ToastStore {
  toasts: Toast[];
  position: ToastPosition;
  addToast: (toast: Omit<Toast, 'id'>) => string;
  removeToast: (id: string) => void;
  clearAll: () => void;
  setPosition: (position: ToastPosition) => void;
}

export const useToastStore = create<ToastStore>((set, get) => ({
  toasts: [],
  position: 'top-right',
  
  addToast: (toast: Omit<Toast, 'id'>) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).substring(2, 11)}`;
    const newToast: Toast = {
      id,
      duration: 5000,
      position: get().position,
      ...toast,
    };
    
    set((state) => ({
      toasts: [...state.toasts, newToast],
    }));
    
    // Auto-dismiss after duration
    if (newToast.duration && newToast.duration > 0) {
      setTimeout(() => {
        get().removeToast(id);
      }, newToast.duration);
    }
    
    return id;
  },
  
  removeToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter((toast) => toast.id !== id),
    }));
  },
  
  clearAll: () => {
    set({ toasts: [] });
  },
  
  setPosition: (position) => {
    set({ position });
  },
}));

// Convenience functions for common toast types
export const toast = {
  success: (message: string, options?: Partial<Omit<Toast, 'id' | 'type' | 'message'>>) => {
    return useToastStore.getState().addToast({ type: 'success', message, ...options });
  },
  error: (message: string, options?: Partial<Omit<Toast, 'id' | 'type' | 'message'>>) => {
    return useToastStore.getState().addToast({ type: 'error', message, ...options });
  },
  info: (message: string, options?: Partial<Omit<Toast, 'id' | 'type' | 'message'>>) => {
    return useToastStore.getState().addToast({ type: 'info', message, ...options });
  },
  warning: (message: string, options?: Partial<Omit<Toast, 'id' | 'type' | 'message'>>) => {
    return useToastStore.getState().addToast({ type: 'warning', message, ...options });
  },
  dismiss: (id: string) => {
    useToastStore.getState().removeToast(id);
  },
  clear: () => {
    useToastStore.getState().clearAll();
  },
};
