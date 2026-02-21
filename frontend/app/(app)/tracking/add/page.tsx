import React from 'react';
import EventTrackingForm from '@/components/forms/EventTrackingForm';

export default function AddEventPage() {
    return (
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-10 w-full min-h-screen bg-gray-50">
            <div className="mb-10 max-w-5xl mx-auto">
                <h1 className="text-4xl font-extrabold text-gray-900 tracking-tight">Supply Chain Operations</h1>
                <p className="mt-3 text-lg text-gray-600">Ensure product traceability by logging operations securely on the Stellar network.</p>
            </div>

            <EventTrackingForm />
        </div>
    );
}
