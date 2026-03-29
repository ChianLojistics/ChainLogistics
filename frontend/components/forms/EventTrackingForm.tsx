"use client";

import React, { useState } from 'react';
import EventTypeSelector, { EventType } from './EventTypeSelector';
import { LocationInput } from "./LocationInput";
import { sanitizeInput, apiRateLimiter, eventTrackingSchema, EVENT_NOTE_MAX_LEN } from "@/lib/validation";
import { EVENT_TRACKING_SUBMIT_DELAY_MS } from "@/lib/constants";

export default function EventTrackingForm() {
    const [eventType, setEventType] = useState<EventType | ''>('');
    const [location, setLocation] = useState('');
    const [note, setNote] = useState('');
    const [productId, setProductId] = useState('');
    const [isSubmitting, setIsSubmitting] = useState(false);
    const [success, setSuccess] = useState(false);
    const [error, setError] = useState('');

    // Expected normally fetched from API/Stellar
    const products = [
        { id: 'PRD-1001-XYZ', name: 'Premium Arabica Coffee Beans' },
        { id: 'PRD-2034-ABC', name: 'Organic Cotton T-Shirt' },
        { id: 'PRD-5099-LMN', name: 'Fair Trade Chocolate' },
    ];

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();

        const result = eventTrackingSchema.safeParse({
            productId,
            eventType,
            location,
            note: note || undefined,
            timestamp: Date.now(),
        });

        if (!result.success) {
            setError(result.error.issues[0].message);
            return;
        }

        if (!apiRateLimiter.check("trackEvent")) {
            setError('Too many requests. Please wait before trying again.');
            return;
        }

        const sanitizedLocation = sanitizeInput(location);
        const sanitizedNote = sanitizeInput(note);

        if (!sanitizedLocation) {
            setError('Location is required');
            return;
        }

        setIsSubmitting(true);
        setError('');

        try {
            setLocation(sanitizedLocation);
            setNote(sanitizedNote);
            // Dummy transaction delay mirroring freighter confirm
            await new Promise((resolve) => setTimeout(resolve, EVENT_TRACKING_SUBMIT_DELAY_MS));
            setSuccess(true);
        } catch (err) {
            setError((err as Error).message || 'Failed to submit transaction');
        } finally {
            setIsSubmitting(false);
        }
    };

    if (success) {
        return (
            <div className="bg-white p-8 md:p-12 rounded-3xl shadow border border-gray-100 max-w-2xl mx-auto text-center">
                <div className="w-20 h-20 bg-green-100 text-green-600 rounded-full flex items-center justify-center mx-auto mb-6 ring-8 ring-green-50">
                    <svg className="w-10 h-10" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" /></svg>
                </div>
                <h2 className="text-3xl font-bold text-gray-900 mb-2">Event Recorded!</h2>
                <p className="text-gray-600 text-lg mb-8">The tracking event has been immutably recorded.</p>

                <div className="bg-gray-50 rounded-2xl p-6 text-left mb-8 font-mono text-sm max-w-sm mx-auto space-y-3 border border-gray-200 shadow-inner">
                    <div className="flex justify-between items-center border-b border-gray-200 pb-2">
                        <span className="text-gray-500 uppercase text-xs tracking-wider">Product ID</span>
                        <span className="text-gray-900 font-bold bg-white px-2 py-1 rounded shadow-sm">{productId}</span>
                    </div>
                    <div className="flex justify-between items-center pt-1">
                        <span className="text-gray-500 uppercase text-xs tracking-wider">Event Action</span>
                        <span className="text-indigo-700 font-bold bg-indigo-50 px-2 py-1 rounded shadow-sm">{eventType}</span>
                    </div>
                </div>

                <button
                    onClick={() => {
                        setSuccess(false);
                        setEventType('');
                        setNote('');
                        setLocation('');
                    }}
                    className="px-8 py-3 bg-indigo-600 text-white font-bold rounded-xl hover:bg-indigo-700 transition shadow-md hover:shadow-lg w-full sm:w-auto"
                >
                    Track Another Event
                </button>
            </div>
        );
    }

    return (
        <div className="bg-white p-6 md:p-10 rounded-3xl shadow border border-gray-100 w-full max-w-5xl mx-auto">
            <div className="mb-8 border-b border-gray-100 pb-6">
                <h2 className="text-2xl font-bold text-gray-900">Log Tracking Event</h2>
                <p className="text-gray-500 mt-2">Record a new step in the product&apos;s journey. Submit to ledger via wallet signature.</p>
            </div>

            {error && (
                <div role="alert" className="mb-8 p-4 bg-red-50 border border-red-200 text-red-700 rounded-xl text-sm flex gap-3 items-center">
                    <svg className="w-5 h-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                    {error}
                </div>
            )}

            <form onSubmit={handleSubmit} className="space-y-10">
                <div className="space-y-3">
                    <label htmlFor="product" className="block text-sm font-semibold text-gray-700 uppercase tracking-wide">1. Select Product *</label>
                    <select
                        id="product"
                        value={productId}
                        onChange={(e) => setProductId(e.target.value)}
                        className="w-full rounded-xl border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 bg-gray-50 border p-4 text-base"
                        required
                    >
                        <option value="">-- Choose a product acting on --</option>
                        {products.map(p => (
                            <option key={p.id} value={p.id}>{p.name} ({p.id})</option>
                        ))}
                    </select>
                </div>

                <div className="space-y-3">
                    <div className="flex items-baseline gap-2">
                        <span className="text-sm font-semibold text-gray-700 uppercase tracking-wide">2. Select Operation</span>
                    </div>
                    <EventTypeSelector
                        value={eventType}
                        onChange={setEventType}
                    />
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                    <div className="space-y-3">
                        <LocationInput
                            id="location"
                            label="3. Location Info"
                            required
                            value={location}
                            onChange={(value) => {
                                setLocation(value);
                                setError('');
                            }}
                            error={!location && error === 'Please fill in all required fields' ? 'Location is required' : undefined}
                        />
                    </div>

                    <div className="space-y-3">
                        <label htmlFor="note" className="block text-sm font-semibold text-gray-700 uppercase tracking-wide">4. Remarks (Optional)</label>
                        <input
                            type="text"
                            id="note"
                            value={note}
                            onChange={(e) => setNote(e.target.value)}
                            maxLength={EVENT_NOTE_MAX_LEN}
                            placeholder="e.g. Temperature checked at 4°C"
                            className="w-full rounded-xl border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 bg-gray-50 border p-4"
                        />
                        <p className="text-sm text-gray-500 mt-1">Additional conditions or remarks during operation. ({note.length}/{EVENT_NOTE_MAX_LEN})</p>
                    </div>
                </div>

                <div className="pt-8 border-t border-gray-200 mt-10 flex flex-col sm:flex-row items-center justify-end gap-4">
                    <button
                        type="button"
                        className="px-8 py-4 w-full sm:w-auto text-gray-600 font-semibold hover:bg-gray-100 rounded-xl transition"
                    >
                        Clear Form
                    </button>
                    <button
                        type="submit"
                        disabled={isSubmitting || !productId || !eventType || !location}
                        className="px-8 py-4 w-full sm:w-auto bg-gray-900 border border-transparent text-white font-bold rounded-xl hover:bg-black transition shadow-lg disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                    >
                        {isSubmitting ? (
                            <>
                                <span className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
                                Processing Transaction...
                            </>
                        ) : (
                            <>
                                <svg className="w-5 h-5 text-indigo-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" /></svg>
                                Sign & Submit Event
                            </>
                        )}
                    </button>
                </div>
            </form>
        </div>
    );
}
