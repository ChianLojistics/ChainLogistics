"use client";

import * as React from "react";

import { cn } from "@/lib/utils";

type ReverseGeocodeResult = {
  display_name?: string;
  address?: {
    road?: string;
    neighbourhood?: string;
    suburb?: string;
    city?: string;
    town?: string;
    village?: string;
    state?: string;
    country?: string;
    postcode?: string;
  };
};

async function reverseGeocode(lat: number, lon: number, signal?: AbortSignal) {
  const url = new URL("https://nominatim.openstreetmap.org/reverse");
  url.searchParams.set("format", "jsonv2");
  url.searchParams.set("lat", String(lat));
  url.searchParams.set("lon", String(lon));
  url.searchParams.set("zoom", "18");
  url.searchParams.set("addressdetails", "1");

  const res = await fetch(url.toString(), {
    method: "GET",
    signal,
    headers: {
      Accept: "application/json",
    },
  });

  if (!res.ok) {
    throw new Error("Failed to reverse geocode coordinates");
  }

  return (await res.json()) as ReverseGeocodeResult;
}

function formatAddress(result: ReverseGeocodeResult) {
  if (typeof result.display_name === "string" && result.display_name.trim().length > 0) {
    return result.display_name;
  }

  const a = result.address;
  if (!a) return null;

  const cityLike = a.city ?? a.town ?? a.village;
  const parts = [a.road, a.suburb ?? a.neighbourhood, cityLike, a.state, a.postcode, a.country].filter(
    (p): p is string => typeof p === "string" && p.trim().length > 0
  );

  return parts.length > 0 ? parts.join(", ") : null;
}

function geolocationErrorMessage(err: GeolocationPositionError) {
  switch (err.code) {
    case err.PERMISSION_DENIED:
      return "Location permission denied. You can type your location manually.";
    case err.POSITION_UNAVAILABLE:
      return "Location unavailable. Please type your location manually.";
    case err.TIMEOUT:
      return "Location request timed out. Please try again or type your location manually.";
    default:
      return "Failed to get current location. Please type your location manually.";
  }
}

export interface LocationInputProps {
  id?: string;
  label?: string;
  required?: boolean;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  error?: string;
  disabled?: boolean;
}

export function LocationInput({
  id = "location",
  label = "Location",
  required,
  value,
  onChange,
  placeholder = "e.g. Facility A or 12.345, -67.890",
  className,
  error,
  disabled,
}: Readonly<LocationInputProps>) {
  const [isLocating, setIsLocating] = React.useState(false);
  const [gpsError, setGpsError] = React.useState<string | null>(null);
  const abortRef = React.useRef<AbortController | null>(null);

  React.useEffect(() => {
    return () => {
      abortRef.current?.abort();
    };
  }, []);

  const handleUseCurrentLocation = async () => {
    if (disabled) return;

    if (!navigator.geolocation) {
      setGpsError("Geolocation is not supported by your browser.");
      return;
    }

    setIsLocating(true);
    setGpsError(null);

    abortRef.current?.abort();
    abortRef.current = new AbortController();

    navigator.geolocation.getCurrentPosition(
      async (pos) => {
        try {
          const { latitude, longitude } = pos.coords;
          const lat = Number(latitude.toFixed(6));
          const lon = Number(longitude.toFixed(6));

          try {
            const result = await reverseGeocode(lat, lon, abortRef.current?.signal);
            const formatted = formatAddress(result);
            onChange(formatted ?? `${lat}, ${lon}`);
          } catch {
            onChange(`${lat}, ${lon}`);
          }
        } catch {
          setGpsError("Failed to use current location. Please type your location manually.");
        } finally {
          setIsLocating(false);
        }
      },
      (err) => {
        setIsLocating(false);
        setGpsError(geolocationErrorMessage(err));
      },
      {
        enableHighAccuracy: true,
        timeout: 15000,
        maximumAge: 10_000,
      }
    );
  };

  return (
    <div className={cn("space-y-3", className)}>
      <label
        htmlFor={id}
        className="block text-sm font-semibold text-gray-700 uppercase tracking-wide"
      >
        {label}
        {required ? " *" : ""}
      </label>

      <div className="flex gap-2">
        <input
          type="text"
          id={id}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          aria-invalid={Boolean(error || gpsError)}
          aria-describedby={`${id}-help ${id}-error`}
          className={cn(
            "flex-1 rounded-xl border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 bg-gray-50 border p-4",
            disabled && "opacity-50 cursor-not-allowed"
          )}
        />

        <button
          type="button"
          onClick={handleUseCurrentLocation}
          disabled={disabled || isLocating}
          className={cn(
            "px-5 bg-white text-gray-700 font-medium rounded-xl hover:bg-gray-50 transition border border-gray-300 flex items-center justify-center shadow-sm whitespace-nowrap",
            (disabled || isLocating) && "opacity-50"
          )}
          aria-label="Use current location"
          title="Use current location"
        >
          {isLocating ? (
            <span className="w-5 h-5 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
          ) : (
            <span className="flex items-center gap-2">
              <svg
                className="w-5 h-5 text-indigo-600"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
              <span className="hidden sm:inline">Use current location</span>
            </span>
          )}
        </button>
      </div>

      <p id={`${id}-help`} className="text-sm text-gray-500 mt-1">
        Provide a facility name/address, or use GPS auto-fill.
      </p>

      {(error || gpsError) && (
        <p id={`${id}-error`} className="text-sm text-red-600">
          {error ?? gpsError}
        </p>
      )}
    </div>
  );
}
