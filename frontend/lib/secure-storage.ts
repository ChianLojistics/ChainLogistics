const SECURE_SESSION_PREFIX = "secure:";

export const secureSession = {
  setItem(key: string, value: string): void {
    try {
      sessionStorage.setItem(SECURE_SESSION_PREFIX + key, value);
    } catch (e) {
      console.error("[secure-session] failed to set item:", e);
    }
  },

  getItem(key: string): string | null {
    try {
      return sessionStorage.getItem(SECURE_SESSION_PREFIX + key);
    } catch {
      return null;
    }
  },

  removeItem(key: string): void {
    try {
      sessionStorage.removeItem(SECURE_SESSION_PREFIX + key);
    } catch (e) {
      console.error("[secure-session] failed to remove item:", e);
    }
  },

  clear(): void {
    try {
      const keys = Object.keys(sessionStorage).filter(k => k.startsWith(SECURE_SESSION_PREFIX));
      keys.forEach(k => sessionStorage.removeItem(k));
    } catch (e) {
      console.error("[secure-session] failed to clear:", e);
    }
  },
};

export const secureLocal = {
  setItem(key: string, value: string): void {
    try {
      localStorage.setItem(key, value);
    } catch (e) {
      console.error("[secure-local] failed to set item:", e);
    }
  },

  getItem(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },

  removeItem(key: string): void {
    try {
      localStorage.removeItem(key);
    } catch (e) {
      console.error("[secure-local] failed to remove item:", e);
    }
  },
};