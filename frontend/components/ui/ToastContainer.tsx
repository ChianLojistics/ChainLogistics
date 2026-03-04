"use client";

import React from 'react';
import { useToastStore } from '@/lib/toast/store';
import { ToastComponent } from './Toast';
import { cn } from '@/lib/utils';

const positionClasses = {
  'top-right': 'fixed top-4 right-4 z-50 flex flex-col gap-2',
  'top-left': 'fixed top-4 left-4 z-50 flex flex-col gap-2',
  'top-center': 'fixed top-4 left-1/2 transform -translate-x-1/2 z-50 flex flex-col gap-2',
  'bottom-right': 'fixed bottom-4 right-4 z-50 flex flex-col gap-2',
  'bottom-left': 'fixed bottom-4 left-4 z-50 flex flex-col gap-2',
  'bottom-center': 'fixed bottom-4 left-1/2 transform -translate-x-1/2 z-50 flex flex-col gap-2',
};

export const ToastContainer: React.FC = () => {
  const { toasts, position, removeToast } = useToastStore();

  if (toasts.length === 0) {
    return null;
  }

  const handleDismiss = (id: string) => {
    removeToast(id);
  };

  return (
    <div 
      className={cn(
        positionClasses[position],
        'pointer-events-none'
      )}
      role="region"
      aria-label="Notifications"
      aria-live="polite"
    >
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className="pointer-events-auto animate-in slide-in-from-right-full fade-in-0 duration-300"
        >
          <ToastComponent toast={toast} onDismiss={handleDismiss} />
        </div>
      ))}
    </div>
  );
};
