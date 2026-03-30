# Toast Notification System

A comprehensive toast notification system built with React, TypeScript, and Zustand for state management. This system provides user-friendly feedback for actions with multiple variants, auto-dismiss functionality, and accessibility features.

## Features

- ✅ **Multiple Toast Types**: Success, Error, Info, Warning variants
- ⏰ **Auto-Dismiss**: Configurable timeout with manual override
- ❌ **Manual Dismiss**: Click X button to close toasts
- 📚 **Toast Queue**: Multiple toasts stack automatically
- 📍 **Position Options**: 6 different screen positions
- 🎨 **Smooth Animations**: Slide-in and fade effects
- ♿ **Accessible**: Screen reader support with ARIA labels
- 🎯 **Action Buttons**: Optional action buttons for user interaction

## Files Created

- `/frontend/lib/toast/store.ts` - Zustand store for toast state management
- `/frontend/components/ui/Toast.tsx` - Individual toast component
- `/frontend/components/ui/ToastContainer.tsx` - Container for managing multiple toasts
- `/frontend/app/toast-demo/page.tsx` - Demo page showcasing all features

## Usage

### Basic Usage

```typescript
import { toast } from '@/lib/toast/store';

// Success toast
toast.success('Operation completed successfully!');

// Error toast
toast.error('Something went wrong. Please try again.');

// Info toast
toast.info('Here is some useful information.');

// Warning toast
toast.warning('Please review this important warning.');
```

### Advanced Usage

```typescript
// Toast with title and custom duration
toast.success('File uploaded successfully!', {
  title: 'Upload Complete',
  duration: 8000, // 8 seconds
});

// Toast with action button
toast.info('New message received', {
  title: 'Messages',
  action: {
    label: 'View Messages',
    onClick: () => {
      // Handle action click
      router.push('/messages');
    },
  },
});

// Manual dismiss
const toastId = toast.info('This can be dismissed manually');
toast.dismiss(toastId);

// Clear all toasts
toast.clear();
```

### Position Configuration

```typescript
import { useToastStore } from '@/lib/toast/store';

const { setPosition } = useToastStore();

// Available positions:
// 'top-right', 'top-left', 'top-center'
// 'bottom-right', 'bottom-left', 'bottom-center'

setPosition('bottom-center');
```

## Component Integration

The toast system is automatically integrated into your app through the layout:

```tsx
// app/layout.tsx
import { ToastContainer } from '@/components/ui/ToastContainer';

export default function RootLayout({ children }) {
  return (
    <html>
      <body>
        <AppProviders>
          {children}
          <ToastContainer />
        </AppProviders>
      </body>
    </html>
  );
}
```

## API Reference

### Toast Types

```typescript
type ToastType = 'success' | 'error' | 'info' | 'warning';
type ToastPosition = 'top-right' | 'top-left' | 'top-center' | 'bottom-right' | 'bottom-left' | 'bottom-center';

interface Toast {
  id: string;
  type: ToastType;
  title?: string;
  message: string;
  duration?: number; // Default: 5000ms
  position?: ToastPosition;
  action?: {
    label: string;
    onClick: () => void;
  };
}
```

### Store Methods

```typescript
// Add a toast
addToast(toast: Omit<Toast, 'id'>): string

// Remove a toast
removeToast(id: string): void

// Clear all toasts
clearAll(): void

// Set default position
setPosition(position: ToastPosition): void
```

### Convenience Functions

```typescript
toast.success(message: string, options?: Partial<Toast>): string
toast.error(message: string, options?: Partial<Toast>): string
toast.info(message: string, options?: Partial<Toast>): string
toast.warning(message: string, options?: Partial<Toast>): string
toast.dismiss(id: string): void
toast.clear(): void
```

## Styling

The toast components use Tailwind CSS classes for styling. Each toast type has its own color scheme:

- **Success**: Green background with green icon
- **Error**: Red background with red icon
- **Info**: Blue background with blue icon
- **Warning**: Yellow background with yellow icon

## Accessibility

- Toasts use `role="alert"` and `aria-live="polite"` for screen readers
- Dismiss buttons have proper `aria-label` attributes
- Toast container uses `role="region"` with `aria-label="Notifications"`
- Focus management for keyboard navigation

## Demo

Visit `/toast-demo` to see the toast system in action with all features demonstrated.

## Dependencies

- React 19+
- TypeScript
- Zustand (state management)
- Tailwind CSS (styling)
- Lucide React (icons)

## Browser Support

The toast system supports all modern browsers that support:
- CSS Grid and Flexbox
- CSS animations and transitions
- ES6+ JavaScript features
